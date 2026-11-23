//! The empty test store from schema.json.

/// The store's type, so generated code names it through forgr.
pub use rusqlite::Connection;

/// An in-memory store with `schema_json` applied by forgr-schema's replay and
/// foreign keys on, holding no rows. `schema!` emits `empty_store()`, which
/// calls this with the schema.json it read.
///
/// # Panics
///
/// When the text does not replay or foreign keys cannot be turned on; the
/// message says why. `schema!` read the same text at build time, so its
/// `empty_store()` does not meet the first.
#[must_use]
pub fn empty_store(schema_json: &str) -> Connection {
    let conn = match forgr_schema::replay(schema_json) {
        Ok(conn) => conn,
        Err(err) => panic!("forgr could not build the empty test store: {err}"),
    };
    if let Err(err) = conn.pragma_update(None, "foreign_keys", true) {
        panic!("forgr could not turn foreign keys on in the empty test store: {err}");
    }
    conn
}

#[cfg(test)]
mod tests {
    use super::*;

    /// forgr#31: the empty store from a schema.json with an FTS5 `_search`, its
    /// shadow tables and an ordinary `_search_map` holds `_search_map`.
    #[test]
    fn the_empty_store_holds_a_table_named_after_a_virtual_table() {
        let source = Connection::open_in_memory().unwrap();
        source
            .execute_batch(
                "CREATE VIRTUAL TABLE _search USING fts5(body);
                 CREATE TABLE _search_map (rid INTEGER PRIMARY KEY, node TEXT NOT NULL) WITHOUT ROWID;",
            )
            .unwrap();
        let objects: Vec<serde_json::Value> = source
            .prepare(
                "SELECT type, name, sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY name",
            )
            .unwrap()
            .query_map([], |r| {
                Ok(serde_json::json!({
                    "type": r.get::<_, String>(0)?,
                    "name": r.get::<_, String>(1)?,
                    "sql": r.get::<_, String>(2)?,
                }))
            })
            .unwrap()
            .map(Result::unwrap)
            .collect();
        let schema_json = serde_json::json!({ "version": null, "objects": objects }).to_string();

        let store = empty_store(&schema_json);
        let kind: String = store
            .query_row(
                "SELECT type FROM pragma_table_list WHERE name = '_search_map'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(kind, "table");
        store
            .execute("INSERT INTO _search_map (rid, node) VALUES (1, 'n')", [])
            .unwrap();
    }
}
