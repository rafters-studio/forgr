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
