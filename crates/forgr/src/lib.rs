//! forgr.

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
/// rows filled by fake, written by `insert(&store)`, which returns them as
/// written, keys included. A TEXT primary key gets a UUIDv7 and an INTEGER one a
/// random positive integer; a column with a default is left to it, a CHECK
/// (col IN (...)) column takes a listed value, a nullable column is NULL in
/// some rows and a nullable foreign key is NULL. A required foreign key to
/// another table gets a fresh parent per row, mocked the same way. Without the
/// feature, calling `mock` is a compile error naming it:
///
/// ```text
/// error[E0277]: mocks need forgr's `fake` feature
/// ```
pub use forgr_macros::schema;

#[cfg(feature = "fake")]
#[doc(hidden)]
pub mod fake;

#[doc(hidden)]
pub mod insert;
#[doc(hidden)]
pub mod store;
#[doc(hidden)]
pub mod value;
