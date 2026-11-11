//! The test `schema!` emits: forgr's SQL check, run by the owning crate's tests.

use std::fmt::Write as _;

/// Emits `forgr_sql_matches_schema`, a test that prepares the SQL in the
/// non-test src of every member of the calling crate's workspace against the
/// schema.json at `path`, relative to the calling crate's manifest directory.
pub(crate) fn emit(out: &mut String, path: &str) {
    let _ = write!(
        out,
        "/// forgr's SQL check: the SQL in the non-test src of every member of \
         this crate's workspace, prepared against this schema.json.\n\
         #[::core::prelude::v1::test]\n\
         fn forgr_sql_matches_schema() {{\n\
         ::forgr::check::run(::core::env!(\"CARGO_MANIFEST_DIR\"), \
         ::core::include_str!(::core::concat!(::core::env!(\"CARGO_MANIFEST_DIR\"), \"/\", {path:?})));\n\
         }}"
    );
}
