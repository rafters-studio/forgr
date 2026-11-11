//! Behind fake: what a mock fills each column with, and its insert.

mod generate;

use rusqlite::Connection;

use crate::insert::{FromRow, Result};
use crate::value::Value;

/// A column's SQLite type affinity, which picks the kind of value fake makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Integer,
    Text,
    Blob,
    Real,
    /// Any other declared type: it takes any value, so fake makes an integer.
    Numeric,
}

/// What a mock writes to one column of every row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    /// A TEXT primary-key column: a UUIDv7.
    Uuid7,
    /// An INTEGER primary-key column: a random positive integer.
    PositiveInteger,
    /// A nullable foreign key: NULL.
    Null,
    /// A column with a schema default: left to it.
    Default,
    /// A value by type, from the CHECK (col IN (...)) list when there is one,
    /// and NULL in some rows when the column is nullable.
    Value {
        ty: Type,
        nullable: bool,
        list: Option<&'static [&'static str]>,
    },
}

/// One column of a mock's table and what it is filled with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Column {
    pub name: &'static str,
    pub fill: Fill,
}

/// Writes `n` rows of `table`, each column filled as `fill` says and then
/// passed to `parents`, which builds the row's parents and sets its foreign
/// keys to them, and returns the rows as written, keys included; `n` of 0
/// writes and returns none.
///
/// # Errors
///
/// When the store refuses a row or a row does not read back.
pub fn insert<R: FromRow>(
    conn: &Connection,
    table: &str,
    columns: &[&str],
    fill: &[Column],
    n: usize,
    mut parents: impl FnMut(Vec<(&'static str, Value)>) -> Result<Vec<(&'static str, Value)>>,
) -> Result<Vec<R>> {
    let mut rng = ::fake::rand::rng();
    (0..n)
        .map(|_| {
            let set = parents(generate::row(&mut rng, fill))?;
            crate::insert::insert(conn, table, columns, set)
        })
        .collect()
}
