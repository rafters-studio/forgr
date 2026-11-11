//! forgr-schema: what an app's schema.json says.
//!
//! [`Schema::read`] reads a schema.json text in migratr's format, whoever wrote
//! it, replays it into in-memory SQLite tables first, and takes each table's
//! columns, keys and foreign keys from what SQLite reports, and each
//! `CHECK (col IN (...))` list from the CREATE TABLE text. It never writes the
//! file and never opens a database file.

mod check_list;
mod file;
mod model;
mod replay;

pub use model::{Column, ForeignKey, PrimaryKey, Table};

/// Why a schema.json could not be read.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("schema.json is not in migratr's format: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("could not open an in-memory SQLite: {0}")]
    Open(#[source] rusqlite::Error),
    #[error("replaying {kind} {name} failed: {source}")]
    Replay {
        kind: String,
        name: String,
        #[source]
        source: rusqlite::Error,
    },
    #[error("reading table {table} failed: {source}")]
    Read {
        table: String,
        #[source]
        source: rusqlite::Error,
    },
}

/// The schema a schema.json describes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    /// Its tables in file order, leaving out virtual tables and their shadow tables.
    pub tables: Vec<Table>,
}

impl Schema {
    /// Reads a schema.json text into the model.
    ///
    /// # Errors
    ///
    /// When the text is not in migratr's format, or an object's SQL does not
    /// replay, or a replayed table cannot be read back.
    pub fn read(text: &str) -> Result<Self, Error> {
        let file = file::parse(text)?;
        let replay = replay::replay(&file.objects)?;
        let tables = replay
            .tables
            .iter()
            .map(|name| {
                let sql = file
                    .objects
                    .iter()
                    .find(|o| o.kind == "table" && &o.name == name)
                    .map_or("", |o| o.sql.as_str());
                model::read_table(&replay.conn, name, sql).map_err(|source| Error::Read {
                    table: name.clone(),
                    source,
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { tables })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_composite_keys_and_foreign_keys() {
        let schema = Schema::read(file::tests::WRITER).unwrap();
        let names: Vec<&str> = schema.tables.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(
            names,
            ["big", "events", "note_tags", "notes", "tags", "wide"]
        );
        let note_tags = &schema.tables[2];
        assert_eq!(
            note_tags.primary_key.as_ref().unwrap().columns,
            ["note", "owner", "name"]
        );
        assert_eq!(
            note_tags.foreign_keys,
            [
                ForeignKey {
                    columns: vec!["owner".into(), "name".into()],
                    table: "tags".into(),
                    references: vec![Some("owner".into()), Some("name".into())],
                },
                ForeignKey {
                    columns: vec!["note".into()],
                    table: "notes".into(),
                    references: vec![Some("id".into())],
                },
            ]
        );
    }

    #[test]
    fn a_bad_text_is_a_parse_error() {
        assert!(matches!(Schema::read("{}"), Err(Error::Parse(_))));
    }
}
