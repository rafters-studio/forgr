//! forgr-schema: what an app's schema.json says.
//!
//! [`Schema::read`] reads a schema.json text in migratr's format, whoever wrote
//! it, replays it into in-memory SQLite tables first, and takes each table's
//! columns, keys and foreign keys from what SQLite reports, and each
//! `CHECK (col IN (...))` list from the CREATE TABLE text. It never writes the
//! file and never opens a database file.

use std::fmt;

mod check_list;
mod file;
mod model;
mod replay;

pub use model::{Column, ForeignKey, PrimaryKey, Table};

/// Why a schema.json could not be read.
#[derive(Debug)]
pub enum Error {
    /// The text is not in migratr's format; the message says where.
    Parse(String),
    /// No in-memory SQLite could be opened.
    Open(rusqlite::Error),
    /// An object's SQL did not replay.
    Replay {
        kind: String,
        name: String,
        source: rusqlite::Error,
    },
    /// A replayed table could not be read back.
    Read {
        table: String,
        source: rusqlite::Error,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(message) => write!(f, "schema.json is not in migratr's format: {message}"),
            Self::Open(source) => write!(f, "could not open an in-memory SQLite: {source}"),
            Self::Replay { kind, name, source } => {
                write!(f, "replaying {kind} {name} failed: {source}")
            }
            Self::Read { table, source } => write!(f, "reading table {table} failed: {source}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(_) => None,
            Self::Open(source) | Self::Replay { source, .. } | Self::Read { source, .. } => {
                Some(source)
            }
        }
    }
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
        let file = file::parse(text).map_err(Error::Parse)?;
        let replay = replay::replay(&file.objects)?;
        let tables = replay
            .tables
            .iter()
            .map(|table| {
                model::read_table(&replay.conn, &table.name, &table.sql).map_err(|source| {
                    Error::Read {
                        table: table.name.clone(),
                        source,
                    }
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { tables })
    }
}

/// Replays a schema.json text into a fresh in-memory SQLite, by the same
/// tables-first replay [`Schema::read`] reads, and returns that store.
///
/// # Errors
///
/// When the text is not in migratr's format, or an object's SQL does not replay.
pub fn replay(text: &str) -> Result<rusqlite::Connection, Error> {
    let file = file::parse(text).map_err(Error::Parse)?;
    Ok(replay::replay(&file.objects)?.conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_gives_the_store_with_the_tables() {
        let conn = replay(file::tests::WRITER).unwrap();
        let n: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='notes'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 1);
        assert!(matches!(replay("{}"), Err(Error::Parse(_))));
    }

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
