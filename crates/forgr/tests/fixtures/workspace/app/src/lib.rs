pub fn by_owner(conn: &rusqlite::Connection, who: &str) -> rusqlite::Result<i64> {
    conn.query_row("SELECT count(*) FROM notes WHERE owner = ?1", [who], |r| r.get(0))
}
#[cfg(test)]
mod tests { const _FIXTURE: &str = "SELECT nothing FROM nowhere"; }
