# forgr

Test data from the schema.json an app already has.

## What the SQL check does not see

`schema!` emits a test, `forgr_sql_matches_schema`, that prepares the SQL in
the non-test `src/` of every crate in the workspace against schema.json. It
does not see everything:

- SQL built with `format!` is not supported in 0.1. The check lists it as
  unchecked and does not prepare it.
- DDL (CREATE, ALTER, DROP), in a string literal or a .sql file, is listed as
  unchecked, never prepared and never failed.
- A string that looks like SQL but does not parse is listed as unchecked and
  never fails the test.
- `#[cfg(test)]` code and a crate's `tests/` are not scanned.
