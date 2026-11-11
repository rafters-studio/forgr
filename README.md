# forgr

Test data from the schema.json an app already has.

An app that owns its SQLite schema needs stores in a known state to test
against. Building them by hand costs too much, and hand-built test data drifts
from the schema without anyone being told. forgr exists so that the people and
agents who write those tests stop building test data by hand and stop finding
out about drift too late.

forgr is a Rust library. It reads a schema.json in migratr's format, whoever
wrote it (migratr writes one after every `up` and `down`), and never writes it.
One `forgr::schema!` call turns that file into a module per table, each with a
factory and, behind the `fake` feature, a mock; the empty test store; and a
test that prepares the app's SQL against the same file. A renamed column fails
the next `cargo clippy --tests`, and SQL the schema no longer matches fails the
next `cargo test`. There is no separate step.

It is for Rust developers, and the agents that write tests in their repos, who
test code against SQLite. It does not run migrations, move rows between
databases, or write event files; those belong to migratr, smugglr and the app.

## Install

In the crate that owns the store, forgr is an optional dependency, a feature
turns it on together with `forgr/fake` for mocks, and a dev-dependency covers
that crate's own tests:

```toml
[features]
testing = ["dep:forgr", "forgr/fake"]

[dependencies]
forgr = { version = "0.1", optional = true }

[dev-dependencies]
forgr = { version = "0.1", features = ["fake"] }
```

The feature name is yours; `testing` is the name the fixture below uses. Other
crates need no forgr dependency of their own.

## Setup

One `forgr::schema!` call, in the library of the crate that owns the store,
inside a module behind a cargo feature. forgr is an optional dependency that
feature turns on, and a dev-dependency for that crate's own tests. Every code
block in this section is from `crates/forgr/tests/fixtures/workspace/`, a
two-crate workspace that `w_rerun.sh` builds and tests. The fixture uses a path
dependency on the forgr in this repository in place of the version above.

The owning crate, `store/Cargo.toml`:

```toml
[package]
name = "store"
version = "0.0.0"
edition = "2024"
publish = false
[features]
testing = ["dep:forgr", "forgr/fake"]
[dependencies]
rusqlite = { version = "0.41", features = ["bundled"] }
forgr = { path = "../../../..", optional = true }
[dev-dependencies]
forgr = { path = "../../../..", features = ["fake"] }
```

`store/src/lib.rs` (the module is `forgr`; the path is relative to the crate's
manifest directory):

```rust
/// Test data from this store's schema, for this crate's tests and, through `testing`, every other crate's.
#[cfg(any(test, feature = "testing"))]
pub mod forgr {
    forgr::schema!("schema/schema.json");
}
```

Another crate's tests depend on the owning crate with the feature, and on
nothing else from forgr. `app/Cargo.toml`:

```toml
[dependencies]
store = { path = "../store" }
rusqlite = { version = "0.41", features = ["bundled"] }
[dev-dependencies]
store = { path = "../store", features = ["testing"] }
```

A normal build of `store` compiles neither forgr nor the fake generator: `cargo
tree -p store -e normal` lists only `rusqlite`, and with `--features testing`
it lists forgr and the generator. `cfg(test)` code is invisible to other
crates, which is why the call sits behind a feature that the other crate turns
on and not behind `cfg(test)` alone.

`schema!` puts its items in the scope it is called in. Called at a test file's
root beside `use forgr::fake::Override;`, it fails with `E0252`, the name
`Override` is defined multiple times. The module around the call avoids that.

Without `fake`, calling `mock` is a compile error that names the feature:
`mocks need forgr's `fake` feature`. forgr's `rusqlite` is the one the
workspace builds with; `cargo tree -i rusqlite` shows one `rusqlite v0.41.1`.

## The empty test store

`empty_store()` is an in-memory SQLite connection with the schema.json applied
and foreign keys on, holding no rows. Each call is a fresh store. It is the
first line of the test in `app/tests/uses_store.rs`, which also holds a factory,
a mock and `Fixed` and `Pick` overrides:

```rust
use store::forgr::notes;
#[test]
fn notes_from_the_owners_schema() {
    let conn = store::forgr::empty_store();
    let n = notes::factory().id("n1").owner("ana").updated_at("t").kind("memo").insert(&conn).unwrap();
    assert_eq!(n.owner, "ana");
    assert_eq!(notes::mock(4).insert(&conn).unwrap().len(), 4);
    use store::forgr::Override::{Fixed, Pick};
    let m = notes::mock(6).with(notes::Overrides { owner: Fixed("bo".into()), kind: Pick(vec!["todo".into()]), ..Default::default() }).insert(&conn).unwrap();
    assert!(m.iter().all(|r| r.owner == "bo" && r.kind == "todo"));
}
```

## A factory

`notes::factory()` starts empty. Setters are named for the columns (`r#type`
for a column named `type`). `insert(&store)` writes the row and returns it as
the module's `Row`, defaults and keys included. From `crates/forgr/tests/scenarios.rs`,
the scenario that inserts a note:

```rust
    let store = macro_toy::empty_store();
    let note = macro_toy::notes::factory()
        .id("n1")
        .owner("ana")
        .updated_at("t")
        .kind("memo")
        .insert(&store)
        .map_err(|err| err.to_string())?;
```

A NOT NULL column with no default, and every primary-key column, left unset is
a compile error. The next section shows it.

## A mock

Behind the `fake` feature, `notes::mock(n)` is the factory filled by fake: `n`
rows, written by `insert(&store)`, returned as written, keys included. A TEXT
primary key gets a UUIDv7 and an INTEGER one a random positive integer. A
column with a default is left to it. A `CHECK (col IN (...))` column takes a
listed value. A nullable column is NULL in some rows. `mock(0)` writes nothing.
Tests do not assert on mock values; they assert on the properties the test
sets.

```rust
    let rows = macro_toy::notes::mock(n)
        .insert(&store)
        .map_err(|err| err.to_string())?;
```

A Rust range gives a random number of rows. `mock(3..32)` is 3 to 31 rows,
`mock(3..=32)` is 3 to 32:

```rust
    macro_toy::events::mock(3..32)
        .insert(&store)
        .map_err(|err| err.to_string())?;
```

Both are from `crates/forgr/tests/scenarios.rs`.

## Overrides

Each table has an `Overrides` struct with one field per column. A field is an
`Override` of `Fake` (the default), `Fixed(value)`, `Pick(list)` or
`Each(closure)`. `..Default::default()` leaves the rest Fake. `schema!`
re-exports `Override` beside the table modules, so another crate's tests need no
forgr dependency to use it.

`Fixed` and `Pick`, from `uses_store.rs` above, and from `scenarios.rs`:

```rust
    use macro_toy::Override::Pick;
    mock_events_with(
        called,
        macro_toy::events::Overrides {
            status: Pick(vec![-1, 0, 1, 2]),
            ..Default::default()
        },
    )
```

`Each` makes the value per row from a closure; `r.row` is the row's number and
`r.pick(list)` picks from a list:

```rust
    use macro_toy::Override;
    mock_events_with(
        called,
        macro_toy::events::Overrides {
            body: Override::each(|r| format!("kid {}", r.row)),
            ..Default::default()
        },
    )
```

Closures in an `Overrides` field infer with no type annotation. For a nullable
column the closure returns the `Option`: `Override::Each(Box::new(|r| Some(r.pick([1, 2]))))`
is in `crates/forgr/tests/fixtures/kessel/k5_rerun.rs`.

## A child attached to a parent

A child is a table with a foreign key. Insert the parent with a factory, then
point the child's key at it. From `scenarios.rs`, over a `nodes` table whose
`parent_id` references `nodes(id)`:

```rust
    let store = edge::empty_store();
    let root = edge::nodes::factory()
        .id("root")
        .parent_id("root")
        .insert(&store)
        .map_err(|err| err.to_string())?;
```

```rust
    let rows = edge::nodes::mock(8)
        .with(edge::nodes::Overrides {
            parent_id: Fixed(root.id.clone()),
            ..Default::default()
        })
        .insert(&store)
        .map_err(|err| err.to_string())?;
```

Given no parent, a mock builds a fresh parent per row for each required foreign
key, mocked the same way, so `combat_styles::mock(8)` over kessel's schema.json
writes 8 `combat_styles` rows and the 8 `objects` rows they need, with no
foreign-key violations.

## When the schema moves

Three things tell you, each in a command the project already runs.

### An unset required column

```rust
mod store {
    forgr::schema!("schema.json");
}

pub fn only_fqn() -> forgr::insert::Result<store::combat_styles::Row> {
    let store = store::empty_store();
    store::combat_styles::factory().fqn("x").insert(&store)
}
```

`cargo check` over that crate, with kessel's schema.json
(`crates/forgr/tests/fixtures/kessel/schema.json`) beside it. It reports one
error per unset required column; this is the first of five:

```text
error[E0277]: a required column was never set: combat_styles::col::combat_style_id
 --> src/lib.rs:7:46
  |
7 |     store::combat_styles::factory().fqn("x").insert(&store)
  |                                              ^^^^^^ this factory never set combat_styles::col::combat_style_id
  |
help: the trait `combat_styles::required::combat_style_id` is not implemented for `combat_styles::col::combat_style_id`
 --> src/lib.rs:2:5
  |
2 |     forgr::schema!("schema.json");
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  = note: the schema says this column is NOT NULL with no default, or it is part of the primary key
note: required by a bound in `combat_styles::Factory::<R0, R1, R2, R3, R4, R5>::insert`
 --> src/lib.rs:2:5
  |
2 |     forgr::schema!("schema.json");
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Factory::<R0, R1, R2, R3, R4, R5>::insert`
  = note: this error originates in the macro `forgr::schema` (in Nightly builds, run with -Z macro-backtrace for more info)
```

### A renamed column

Rename `notes.owner` to `author` in the workspace fixture's
`store/schema/schema.json` and touch nothing else. `schema!` tracks the file, so
`cargo clippy --workspace --tests` fails in the other crate's tests. The three
errors, less two `note:` lines that name a temporary file:

```text
error[E0599]: no method named `owner` found for struct `store::forgr::notes::Factory<R0, R1, R2, R3>` in the current scope
 --> app/tests/uses_store.rs:5:39
  |
5 |     let n = notes::factory().id("n1").owner("ana").updated_at("t").kind("memo").insert(&conn).unwrap();
  |                                       ^^^^^ method not found in `Factory<Set, author, updated_at, kind>`
  |

error[E0560]: struct `store::forgr::notes::Overrides` has no field named `owner`
 --> app/tests/uses_store.rs:9:52
  |
9 |     let m = notes::mock(6).with(notes::Overrides { owner: Fixed("bo".into()), kind: Pick(vec!["todo".into()]), ..Default::default() }...
  |                                                    ^^^^^ `store::forgr::notes::Overrides` does not have this field
  |
  = note: available fields are: `id`, `author`, `body`, `updated_at`

error[E0609]: no field `owner` on type `&store::forgr::notes::Row`
  --> app/tests/uses_store.rs:10:32
   |
10 |     assert!(m.iter().all(|r| r.owner == "bo" && r.kind == "todo"));
   |                                ^^^^^ unknown field
   |
   = note: available fields are: `id`, `author`, `body`, `updated_at`, `kind`

Some errors have detailed explanations: E0560, E0599, E0609.
For more information about an error, try `rustc --explain E0560`.
error: could not compile `app2` (test "uses_store") due to 3 previous errors
```

The error is rustc's own.

### SQL the schema no longer matches

`schema!` also emits `forgr_sql_matches_schema`, a test of the owning crate. It
prepares the string literals and `.sql` statements in the non-test `src/` of
every crate in the workspace against the schema.json, in memory. Before the
rename, in `store`:

```text
forgr SQL check:
unchecked store/src/lib.rs:2: DDL is not checked: CREATE TABLE IF NOT EXISTS notes (id TEXT PRIMARY KEY)
unchecked store/src/sql/report.sql #1: DDL is not checked: CREATE INDEX IF NOT EXISTS notes_owner ON notes(owner)
0 of 3 checked SQL statements do not match schema.json (2 unchecked, listed above)
```

After it, `cargo test -p store` fails, with the file and line of each statement
the schema does not match (the panic location line is left out):

```text
forgr SQL check:
unchecked store/src/lib.rs:2: DDL is not checked: CREATE TABLE IF NOT EXISTS notes (id TEXT PRIMARY KEY)
unchecked store/src/sql/report.sql #1: DDL is not checked: CREATE INDEX IF NOT EXISTS notes_owner ON notes(owner)
app/src/lib.rs:2: no such column: owner: SELECT count(*) FROM notes WHERE owner = ?1
store/src/lib.rs:4: no such column: owner: SELECT owner FROM notes WHERE id = ?1
store/src/sql/report.sql #2: no such column: owner: SELECT owner, count(*) FROM notes GROUP BY owner
3 of 3 checked SQL statements do not match schema.json (2 unchecked, listed above)
```

The report counts checked, failed and unchecked apart. The runs above are
`crates/forgr/tests/fixtures/workspace/w4_rerun.sh` and `w_rerun.sh`.

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

## Limits in 0.1

- SQL built with `format!` is unchecked. The check lists it.
- DDL is unchecked, whether it is in a string literal or a `.sql` file.
- An INTEGER column whose values mean an enum, with no CHECK, gets any integer
  from a mock. Set it with `Pick` or `Fixed`. forgr does not look for your Rust
  enums.
- forgr does not read a TEXT column as JSON. It is a TEXT column: a mock fills
  it like any other, and the test sets a JSON value with `Fixed`, `Pick` or
  `Each`.
- An FTS5 virtual table gets no factory and no mock, and its shadow tables are
  skipped. `notes_fts::factory()` and `notes_fts::mock(8)` do not compile.
- One schema per workspace: `schema!` is called once, in the crate that owns the
  store.
- An `events` table is a table like any other. forgr writes no event files.
