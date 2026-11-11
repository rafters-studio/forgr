# forgr-macros

The `schema!` macro for forgr.

forgr-macros holds the code generation behind `forgr::schema!`. You do not
depend on it directly: the [forgr](https://github.com/rafters-studio/forgr#readme)
crate re-exports the macro, and forgr's README shows the setup and what the
macro generates.

The call takes one string literal, a path to a schema.json relative to the
calling crate's manifest directory:

```rust
forgr::schema!("schema/schema.json");
```

It reads the file with forgr-schema when the macro expands. For each table it
emits a module with `Row`, `factory()`, `mock(n)` and `Overrides`, plus
`empty_store()` and the test `forgr_sql_matches_schema`. Without the `fake`
feature, `mock` is a stand-in whose error names the feature; forgr turns the
feature on through its own `fake`.
