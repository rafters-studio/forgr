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
    store: Slot<rusqlite::Connection>,
    written: Slot<rusqlite::Result<usize>>,
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
    if now != built {
        return Err(format!("{MACRO_TOY} changed after the tests were built"));
    }
    // A write during the build would be in what the build saw, so also hold
    // the file to the committed bytes.
    let diff = std::process::Command::new("git")
        .args(["diff", "--quiet", "--exit-code", "--", MACRO_TOY])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .map_err(|err| format!("could not run git to compare {MACRO_TOY}: {err}"))?;
    if diff.success() {
        Ok(())
    } else {
        Err(format!("{MACRO_TOY} differs from the committed file"))
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

const KESSEL_TABLES: [&str; 2] = ["combat_styles", "objects"];

#[given("kessel's schema.json fixture")]
fn kessel_fixture() {
    // The module `kessel` above holds the call over it.
}

#[when("a test takes the empty test store")]
fn takes_store(called: &Called) {
    called.store.set(kessel::empty_store());
}

#[then("the store has the tables combat_styles and objects")]
fn has_tables(called: &Called) -> Result<(), String> {
    let conn = called.store.take().ok_or("no store was taken")?;
    for table in KESSEL_TABLES {
        let n: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |r| r.get(0),
            )
            .map_err(|err| format!("{table}: {err}"))?;
        if n != 1 {
            return Err(format!("the store has no table {table}"));
        }
    }
    Ok(())
}

#[then("the combat_styles and objects tables have no rows")]
fn no_rows(called: &Called) -> Result<(), String> {
    let conn = called.store.take().ok_or("no store was taken")?;
    for table in KESSEL_TABLES {
        let n: i64 = conn
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .map_err(|err| format!("{table}: {err}"))?;
        if n != 0 {
            return Err(format!("{table} has {n} rows"));
        }
    }
    Ok(())
}

#[given(
    "the empty test store for kessel's schema.json fixture, where combat_styles has a foreign key to objects"
)]
fn store_with_foreign_key(called: &Called) -> Result<(), String> {
    let conn = kessel::empty_store();
    let n: i64 = conn
        .query_row(
            "SELECT count(*) FROM pragma_foreign_key_list('combat_styles') \
             WHERE \"table\" = 'objects' AND \"from\" = 'combat_style_id'",
            [],
            |r| r.get(0),
        )
        .map_err(|err| err.to_string())?;
    if n != 1 {
        return Err("combat_styles has no foreign key to objects".to_owned());
    }
    called.store.set(conn);
    Ok(())
}

#[when("the test writes a combat_styles row whose foreign key names no objects row")]
fn writes_orphan(called: &Called) -> Result<(), String> {
    let conn = called.store.take().ok_or("no store was taken")?;
    // Every NOT NULL column is set, so the foreign key is the only constraint the row breaks.
    called.written.set(conn.execute(
        "INSERT INTO combat_styles \
         (combat_style_id, fqn, fqn_segment, display_segment, faction, attack_type) \
         VALUES ('no-such-object', 'class.pc.advanced.sage', 'force_wizard', 'sage', 'republic', 'force')",
        [],
    ));
    Ok(())
}

#[then("the store refuses the row")]
fn refuses(called: &Called) -> Result<(), String> {
    match called.written.take().ok_or("no row was written")? {
        Err(rusqlite::Error::SqliteFailure(err, _))
            if err.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY =>
        {
            Ok(())
        }
        Err(err) => Err(format!(
            "the row was refused, but not for its foreign key: {err}"
        )),
        Ok(n) => Err(format!("the store took the row ({n} written)")),
    }
}

scenarios!("tests", fixtures = [called: Called]);
