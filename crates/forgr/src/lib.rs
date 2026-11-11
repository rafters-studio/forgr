//! forgr: test data from the schema.json an app already has.
//!
//! A Rust library for developers and agents who test code against SQLite. One
//! [`schema!`] call, in the crate that owns the store, reads a schema.json in
//! migratr's format and gives every crate's tests a module per table with a
//! factory and, behind the `fake` feature, a mock; the empty test store; and a
//! test that prepares the app's SQL against the same file. A renamed column
//! fails `cargo clippy --tests`; SQL the schema no longer matches fails
//! `cargo test`.
//!
//! The README has the rest, with output pasted from runs:
//!
//! - [Install](https://github.com/rafters-studio/forgr#install)
//! - [Setup](https://github.com/rafters-studio/forgr#setup): the owning crate,
//!   the feature, and the crates that test through it.
//! - [The empty test store](https://github.com/rafters-studio/forgr#the-empty-test-store)
//! - [A factory](https://github.com/rafters-studio/forgr#a-factory)
//! - [A mock](https://github.com/rafters-studio/forgr#a-mock)
//! - [Overrides](https://github.com/rafters-studio/forgr#overrides)
//! - [A child attached to a parent](https://github.com/rafters-studio/forgr#a-child-attached-to-a-parent)
//! - [When the schema moves](https://github.com/rafters-studio/forgr#when-the-schema-moves):
//!   the compile error, the clippy error and the SQL check report.
//! - [What the SQL check does not see](https://github.com/rafters-studio/forgr#what-the-sql-check-does-not-see)
//! - [Limits in 0.1](https://github.com/rafters-studio/forgr#limits-in-01)

/// `schema!("<path>/schema.json")`, the path relative to the calling crate's
/// manifest directory, gives one module per table of that schema.json and
/// `empty_store()`, the empty test store, a `rusqlite::Connection` named
/// through forgr. Call it inside a module, since its items go into the scope it
/// is called in.
///
/// Each table's module has `factory()`, which starts empty and sets columns
/// with chained setters named for them (`r#type` for a column named `type`);
/// its `insert(&store)` writes the row and returns it as the module's `Row`,
/// defaults and keys included. A NOT NULL column with no default, and every
/// primary-key column, left unset is a compile error naming the column:
///
/// ```text
/// error[E0277]: a required column was never set: combat_styles::col::combat_style_id
/// ```
///
/// With forgr's `fake` feature, each table's module also has `mock(n)`: `n`
/// rows filled by fake, or a random number in a Rust range (`mock(3..32)` is 3
/// to 31 rows), written by `insert(&store)`, which returns them as written,
/// keys included. `mock(n).with(Overrides { .. })` sets what the mock writes to
/// a column: each field of the table's `Overrides` is an `Override`, Fake by
/// default, `Fixed(value)`, `Pick(list)` or `Each(closure)`, made per row from
/// the closure's argument (its `row`, and `pick(list)`); `..Default::default()`
/// leaves the rest Fake. `schema!` re-exports `Override` beside the table
/// modules:
///
/// ```text
/// events::mock(12).with(events::Overrides {
///     status: Pick(vec![-1, 0, 1, 2]),
///     body: Override::each(|r| format!("kid {}", r.row)),
///     ..Default::default()
/// })
/// ```
/// A TEXT primary key gets a UUIDv7 and an INTEGER one a
/// random positive integer; a column with a default is left to it, a CHECK
/// (col IN (...)) column takes a listed value, a nullable column is NULL in
/// some rows and a nullable foreign key is NULL. A required foreign key to
/// another table that the test leaves Fake gets a fresh parent per row, mocked
/// the same way. Without the
/// feature, calling `mock` is a compile error naming it:
///
/// ```text
/// error[E0277]: mocks need forgr's `fake` feature
/// ```
///
/// `schema!` also emits `forgr_sql_matches_schema`, a test of the crate that
/// calls it: forgr's SQL check reads the members of the workspace from its
/// manifest, takes the string literals in each member's non-test `src/`
/// (`#[cfg(test)]` code and `tests/` are not scanned) and each statement of its
/// .sql files, and prepares them against schema.json in memory. A statement
/// naming what the schema lacks fails the test with its file and line; the
/// report counts checked, failed and unchecked apart. SQL built with `format!`
/// is not supported in 0.1: it is listed as unchecked. DDL (CREATE, ALTER,
/// DROP) and SQL-looking text that does not parse are listed as unchecked,
/// never failed.
pub use forgr_macros::schema;

#[doc(hidden)]
pub mod check;

#[cfg(feature = "fake")]
#[doc(hidden)]
pub mod fake;

#[doc(hidden)]
pub mod insert;
#[doc(hidden)]
pub mod store;
#[doc(hidden)]
pub mod value;
