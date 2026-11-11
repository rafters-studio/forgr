//! The schema.json format: a version and objects of type, name and sql, read with
//! forgr's own types.

use serde_json::Value;

/// The contents of a schema.json.
#[derive(Debug)]
pub(crate) struct SchemaFile {
    /// The newest applied migration's version; `None` when the file records none.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "part of the format; only the tests read it yet")
    )]
    pub version: Option<u64>,
    pub objects: Vec<Object>,
}

/// One table, index, trigger or view, with the SQL SQLite stored for it.
#[derive(Debug, Clone)]
pub(crate) struct Object {
    pub kind: String,
    pub name: String,
    pub sql: String,
}

/// Parses a schema.json text; the error says what is not in the format.
pub(crate) fn parse(text: &str) -> Result<SchemaFile, String> {
    let value: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let version = match value.get("version") {
        None | Some(Value::Null) => None,
        Some(v) => Some(v.as_u64().ok_or("version is not a whole number or null")?),
    };
    let objects = value
        .get("objects")
        .and_then(Value::as_array)
        .ok_or("objects is missing or not an array")?
        .iter()
        .enumerate()
        .map(|(at, object)| {
            let field = |key: &str| {
                object
                    .get(key)
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .ok_or(format!("objects[{at}].{key} is missing or not a string"))
            };
            Ok(Object {
                kind: field("type")?,
                name: field("name")?,
                sql: field("sql")?,
            })
        })
        .collect::<Result<_, String>>()?;
    Ok(SchemaFile { version, objects })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// The writer toy's schema.json: what migratr wrote from the toy's DDL.
    pub(crate) const WRITER: &str = include_str!("../tests/fixtures/writer/schema.json");

    #[test]
    fn reads_the_writer_fixture() {
        let file = parse(WRITER).expect("parse");
        assert_eq!(file.version, None);
        assert_eq!(file.objects.len(), 15);
        let first = &file.objects[0];
        assert_eq!(
            (first.kind.as_str(), first.name.as_str()),
            ("index", "a_notes_owner")
        );
    }
}
