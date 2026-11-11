//! The schema.json format: a version and objects of type, name and sql, read with
//! forgr's own types.

use serde::Deserialize;

/// The contents of a schema.json.
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Object {
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    pub sql: String,
}

/// Parses a schema.json text.
pub(crate) fn parse(text: &str) -> Result<SchemaFile, serde_json::Error> {
    serde_json::from_str(text)
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
