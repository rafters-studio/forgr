//! A stand-in app: its SQL lives where apps keep it, as literals in code and a .sql file.
use rusqlite::{Connection, Result};

pub const NOTES_BY_OWNER: &str = "SELECT id, body FROM notes WHERE owner = ?1";

pub fn open_todos(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT n FROM a_open_todo_count", [], |r| r.get(0))
}

pub fn touch(conn: &Connection, id: &str) -> Result<usize> {
    conn.execute("UPDATE notes SET updated_at = datetime('now') WHERE id = ?1", [id])
}

pub fn count(conn: &Connection, table: &str) -> Result<i64> {
    conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
}
