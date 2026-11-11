//! Acceptance-scenario runner for the `forgr-schema` crate.
//!
//! `scenarios!` discovers every `.feature` file under `tests/` and emits one
//! test per scenario. Step definitions must compile into this binary.

use forgr_schema::Schema;
use rstest::fixture;
use rstest_bdd::Slot;
use rstest_bdd_macros::{ScenarioState, given, scenarios, then, when};

/// The schema.json text read and the model it read into.
#[derive(Debug, Default, ScenarioState)]
struct Read {
    text: Slot<String>,
    schema: Slot<Schema>,
}

#[fixture]
fn read() -> Read {
    Read::default()
}

#[given(
    "the writer schema.json fixture, which holds the FTS5 table notes_fts and its shadow tables"
)]
fn writer_fixture(read: &Read) -> Result<(), String> {
    let text = include_str!("fixtures/writer/schema.json");
    for table in ["notes_fts", "notes_fts_config", "notes_fts_data"] {
        if !text.contains(&format!("\"name\": \"{table}\"")) {
            return Err(format!("the writer fixture holds no {table}"));
        }
    }
    read.text.set(text.to_owned());
    Ok(())
}

#[when("forgr-schema reads it into the schema model")]
fn reads_it(read: &Read) -> Result<(), String> {
    let text = read.text.get().ok_or("no schema.json was given")?;
    read.schema
        .set(Schema::read(&text).map_err(|err| err.to_string())?);
    Ok(())
}

#[then(
    "the model's tables include no notes_fts and no table named notes_fts_ followed by anything"
)]
fn no_virtual_or_shadow(read: &Read) -> Result<(), String> {
    let schema = read.schema.take().ok_or("nothing was read")?;
    let left_in: Vec<&str> = schema
        .tables
        .iter()
        .map(|t| t.name.as_str())
        .filter(|name| *name == "notes_fts" || name.starts_with("notes_fts_"))
        .collect();
    if schema.tables.is_empty() {
        return Err("the model holds no tables at all".to_owned());
    }
    if left_in.is_empty() {
        Ok(())
    } else {
        Err(format!("the model's tables include {left_in:?}"))
    }
}

scenarios!("tests", fixtures = [read: Read]);
