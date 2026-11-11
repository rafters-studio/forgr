//! forgr.

/// `schema!("<path>/schema.json")`, the path relative to the calling crate's
/// manifest directory, gives one module per table of that schema.json. Call it
/// inside a module, since its items go into the scope it is called in.
pub use forgr_macros::schema;
