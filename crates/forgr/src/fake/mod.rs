//! Behind fake: what a mock fills each column with, a mock's overrides and
//! count, and its insert.

mod generate;

use std::ops::{Range, RangeInclusive};

use fake::rand::RngExt;
use fake::rand::rngs::ThreadRng;
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

/// What a mock writes to one column, one field of a table's `Overrides`.
pub enum Override<T> {
    /// A value by fake, as the column's type, CHECK list and keys say.
    Fake,
    /// This value in every row.
    Fixed(T),
    /// A value from this list in every row.
    Pick(Vec<T>),
    /// The closure's value, made per row.
    Each(Box<dyn FnMut(&mut Pick) -> T>),
}

// By hand: a derive would bound T by Default, which a NUMERIC column's Value is not.
#[allow(clippy::derivable_impls)]
impl<T> Default for Override<T> {
    fn default() -> Self {
        Self::Fake
    }
}

impl<T> Override<T> {
    /// The closure's value, made per row.
    pub fn each(f: impl FnMut(&mut Pick) -> T + 'static) -> Self {
        Self::Each(Box::new(f))
    }

    /// This row's value; `None` for Fake.
    ///
    /// # Panics
    ///
    /// When a `Pick` list is empty.
    #[doc(hidden)]
    pub fn value(&mut self, r: &mut Pick) -> Option<T>
    where
        T: Clone,
    {
        match self {
            Self::Fake => None,
            Self::Fixed(value) => Some(value.clone()),
            Self::Pick(list) => Some(r.pick(list.iter().cloned())),
            Self::Each(f) => Some(f(r)),
        }
    }
}

/// The row an `Each` closure makes a value for.
pub struct Pick {
    /// The row's place in the mock, from 0.
    pub row: usize,
    rng: ThreadRng,
}

impl Pick {
    /// A value from `list`, at random.
    ///
    /// # Panics
    ///
    /// When `list` is empty.
    pub fn pick<T>(&mut self, list: impl IntoIterator<Item = T>) -> T {
        let mut list: Vec<T> = list.into_iter().collect();
        let at = self.rng.random_range(..list.len());
        list.swap_remove(at)
    }
}

/// How many rows a mock writes: `n`, or a random number in a Rust range
/// (`3..32` is 3 to 31, `3..=32` is 3 to 32).
pub struct Count(RangeInclusive<usize>);

impl From<usize> for Count {
    fn from(n: usize) -> Self {
        Self(n..=n)
    }
}

impl From<Range<usize>> for Count {
    fn from(range: Range<usize>) -> Self {
        Self(range.start..=range.end.saturating_sub(1))
    }
}

impl From<RangeInclusive<usize>> for Count {
    fn from(range: RangeInclusive<usize>) -> Self {
        Self(range)
    }
}

/// Writes `count` rows of `table`, each column filled as `fill` says and then
/// passed to `each` with the row, which builds the row's parents, sets its
/// foreign keys to them and applies the test's overrides, and returns the
/// rows as written, keys included; a count of 0 writes and returns none.
///
/// # Errors
///
/// When the store refuses a row or a row does not read back.
///
/// # Panics
///
/// When `count` is an empty range.
pub fn insert<R: FromRow>(
    conn: &Connection,
    table: &str,
    columns: &[&str],
    fill: &[Column],
    count: Count,
    mut each: impl FnMut(&mut Pick, Vec<(&'static str, Value)>) -> Result<Vec<(&'static str, Value)>>,
) -> Result<Vec<R>> {
    let mut r = Pick {
        row: 0,
        rng: ::fake::rand::rng(),
    };
    let n = r.rng.random_range(count.0);
    (0..n)
        .map(|row| {
            r.row = row;
            let set = generate::row(&mut r.rng, fill);
            let set = each(&mut r, set)?;
            crate::insert::insert(conn, table, columns, set)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pick() -> Pick {
        Pick {
            row: 0,
            rng: ::fake::rand::rng(),
        }
    }

    #[test]
    fn a_range_counts_as_rust_does() {
        let mut rng = ::fake::rand::rng();
        for _ in 0..200 {
            let n = rng.random_range(Count::from(3..32).0);
            assert!((3..32).contains(&n));
            let n = rng.random_range(Count::from(3..=32).0);
            assert!((3..=32).contains(&n));
        }
        assert_eq!(Count::from(8).0, 8..=8);
    }

    #[test]
    fn each_variant_gives_its_value() {
        let mut r = pick();
        assert_eq!(Override::<i64>::Fake.value(&mut r), None);
        assert_eq!(Override::Fixed(4).value(&mut r), Some(4));
        let picked = Override::Pick(vec![1, 2]).value(&mut r);
        assert!(matches!(picked, Some(1 | 2)));
        r.row = 5;
        assert_eq!(
            Override::each(|r| format!("kid {}", r.row)).value(&mut r),
            Some("kid 5".to_owned())
        );
    }
}
