//! Acceptance-scenario runner for the `forgr` crate.
//!
//! `scenarios!` discovers every `.feature` file under `tests/` and emits one
//! test per scenario. Step definitions must compile into this binary.

use rstest::fixture;
use rstest_bdd::Slot;
use rstest_bdd_macros::{ScenarioState, given, scenarios, then, when};

/// The macro toy's schema.json, through `schema!` as a user calls it.
mod macro_toy {
    forgr::schema!("tests/fixtures/macro/schema.json");
}

/// Kessel's schema.json, through `schema!` as a user calls it.
mod kessel {
    forgr::schema!("tests/fixtures/kessel/schema.json");
}

const MACRO_TOY: &str = "tests/fixtures/macro/schema.json";

/// The schema.json as the build saw it, and the modules the tests named.
#[derive(Debug, Default, ScenarioState)]
struct Called {
    built: Slot<&'static str>,
    named: Slot<Vec<&'static str>>,
}

#[fixture]
fn called() -> Called {
    Called::default()
}

/// Names a generated module: it compiles only when the module exists.
macro_rules! name {
    ($schema:ident :: $table:ident) => {{
        #[allow(unused_imports)]
        use $schema::$table as _;
        stringify!($table)
    }};
}

#[given("a crate that calls schema! over the macro toy's schema.json fixture")]
fn calls_macro_toy(called: &Called) {
    called.built.set(include_str!("fixtures/macro/schema.json"));
}

#[when("the developer builds and runs the crate's tests")]
fn builds_and_runs() {
    // This test binary is the crate's tests, built with the call above and now running.
}

#[then("the crate's schema.json is unchanged")]
fn unchanged(called: &Called) -> Result<(), String> {
    let built = called.built.take().ok_or("no schema.json was built")?;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(MACRO_TOY);
    let now = std::fs::read_to_string(&path).map_err(|err| format!("{MACRO_TOY}: {err}"))?;
    if now == built {
        Ok(())
    } else {
        Err(format!("{MACRO_TOY} changed after the tests were built"))
    }
}

#[given("an owning crate that calls schema! over kessel's schema.json fixture")]
fn calls_kessel() {
    // The module `kessel` above holds the call.
}

#[when("its tests name the generated modules")]
fn name_modules(called: &Called) {
    called
        .named
        .set(vec![name!(kessel::combat_styles), name!(kessel::objects)]);
}

#[then("they can name a combat_styles module and an objects module")]
fn named(called: &Called) -> Result<(), String> {
    let named = called.named.take().ok_or("no module was named")?;
    if named == ["combat_styles", "objects"] {
        Ok(())
    } else {
        Err(format!("the tests named {named:?}"))
    }
}

scenarios!("tests", fixtures = [called: Called]);
