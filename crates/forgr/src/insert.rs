//! Insert and read-back of rows.

use rusqlite::Connection;

use crate::value::Value;

/// What a read-back can fail with, and the row it reads from, so generated
/// code names them through forgr.
pub use rusqlite::{Result, Row as SqlRow};

/// The state of a factory's required column once a setter has set it. Before
/// that, the column's state is its marker type, `<table>::col::<column>`.
pub struct Set;

/// What a table's stand-in `mock` needs and nothing has: `schema!` emits the
/// stand-in when forgr is built without its `fake` feature, so calling it names
/// the feature.
#[diagnostic::on_unimplemented(
    message = "mocks need forgr's `fake` feature",
    label = "this mock needs forgr's `fake` feature",
    note = "add features = [\"fake\"] to forgr in [dev-dependencies]"
)]
pub trait MocksNeedFake {
    /// What the mock would be.
    type Mock;
    /// The mock.
    fn mock(self) -> Self::Mock;
}

/// A table's row, read back from a row SQLite returned.
pub trait FromRow: Sized {
    /// Reads the row, its columns in declaration order.
    ///
    /// # Errors
    ///
    /// When a column's value does not convert to its Rust type.
    fn from_row(row: &SqlRow<'_>) -> Result<Self>;
}

/// `set` with `column` given `value`, in place of any value it had.
#[must_use]
pub fn put(
    mut set: Vec<(&'static str, Value)>,
    column: &'static str,
    value: Value,
) -> Vec<(&'static str, Value)> {
    set.retain(|(c, _)| *c != column);
    set.push((column, value));
    set
}

/// Writes one row of `table`, the `set` columns given their values and every
/// other column left to the schema, and returns the row as written: every
/// column of `columns`, defaults and keys included.
///
/// # Errors
///
/// When the store refuses the row or the row does not read back.
pub fn insert<R: FromRow>(
    conn: &Connection,
    table: &str,
    columns: &[&str],
    set: Vec<(&str, Value)>,
) -> Result<R> {
    let returning = columns
        .iter()
        .map(|c| quote(c))
        .collect::<Vec<_>>()
        .join(", ");
    let (names, values): (Vec<&str>, Vec<Value>) = set.into_iter().unzip();
    let sql = if names.is_empty() {
        format!(
            "INSERT INTO {} DEFAULT VALUES RETURNING {returning}",
            quote(table)
        )
    } else {
        let names = names.iter().map(|c| quote(c)).collect::<Vec<_>>();
        let params = (1..=names.len())
            .map(|n| format!("?{n}"))
            .collect::<Vec<_>>();
        format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING {returning}",
            quote(table),
            names.join(", "),
            params.join(", ")
        )
    };
    conn.query_row(&sql, rusqlite::params_from_iter(values), |row| {
        R::from_row(row)
    })
}

/// An SQL identifier, double-quoted.
fn quote(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Pair(i64, String);

    impl FromRow for Pair {
        fn from_row(row: &SqlRow<'_>) -> Result<Self> {
            Ok(Self(row.get(0)?, row.get(1)?))
        }
    }

    fn store() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE \"t\"\"q\" (id INTEGER PRIMARY KEY, s TEXT NOT NULL DEFAULT 'd')",
        )
        .unwrap();
        conn
    }

    #[test]
    fn returns_the_row_with_the_defaults() {
        let conn = store();
        let row: Pair =
            insert(&conn, "t\"q", &["id", "s"], vec![("id", Value::Integer(7))]).unwrap();
        assert_eq!(row, Pair(7, "d".into()));
    }

    #[test]
    fn a_row_with_nothing_set_takes_every_default() {
        let conn = store();
        let row: Pair = insert(&conn, "t\"q", &["id", "s"], Vec::new()).unwrap();
        assert_eq!(row, Pair(1, "d".into()));
    }
}
