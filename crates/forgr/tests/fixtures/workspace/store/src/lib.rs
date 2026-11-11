//! The crate that owns the store and its schema. (Adapted for forgr #10, on this line so the toy's line numbers hold: forgr_toy::schema! is forgr::schema!, the built forgr, whose generated code names ::forgr:: paths.)
pub const CREATE_NOTES: &str = "CREATE TABLE IF NOT EXISTS notes (id TEXT PRIMARY KEY)";
pub fn owner_of(conn: &rusqlite::Connection, id: &str) -> rusqlite::Result<String> {
    conn.query_row("SELECT owner FROM notes WHERE id = ?1", [id], |r| r.get(0))
}

/// Test data from this store's schema, for this crate's tests and, through `testing`, every other crate's.
#[cfg(any(test, feature = "testing"))]
pub mod forgr {
    forgr::schema!("schema/schema.json");
}
