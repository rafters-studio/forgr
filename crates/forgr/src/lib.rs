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
pub use forgr_macros::schema;

#[doc(hidden)]
pub mod insert;
#[doc(hidden)]
pub mod store;
#[doc(hidden)]
pub mod value;
