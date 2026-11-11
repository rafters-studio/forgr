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

/// The edge schema.json, through `schema!` as a user calls it.
mod edge {
    forgr::schema!("tests/fixtures/edge/schema.json");
}

const MACRO_TOY: &str = "tests/fixtures/macro/schema.json";

/// The schema.json as the build saw it, and the modules the tests named.
#[derive(Debug, Default, ScenarioState)]
struct Called {
    built: Slot<&'static str>,
    named: Slot<Vec<&'static str>>,
    store: Slot<rusqlite::Connection>,
    written: Slot<rusqlite::Result<usize>>,
    note: Slot<macro_toy::notes::Row>,
    object: Slot<kessel::objects::Row>,
    kind: Slot<edge::kinds::Row>,
    event: Slot<macro_toy::events::Row>,
    build: Slot<std::process::Output>,
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

#[given(
    "a crate that calls schema! over the macro toy's schema.json fixture and has no database file"
)]
fn calls_macro_toy_without_database() -> Result<(), String> {
    // The module `macro_toy` above holds the call; the crate holds no database file.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for entry in walk(dir)? {
        let ext = entry
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        if ["db", "sqlite", "sqlite3"].contains(&ext) {
            return Err(format!(
                "the crate holds a database file: {}",
                entry.display()
            ));
        }
    }
    Ok(())
}

/// Every file under `dir`.
fn walk(dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|err| format!("{}: {err}", dir.display()))? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.is_dir() {
            files.extend(walk(&path)?);
        } else {
            files.push(path);
        }
    }
    Ok(files)
}

#[when(
    "a test inserts a notes factory with id n1, owner ana, updated_at t and kind memo into the empty test store"
)]
fn inserts_note(called: &Called) -> Result<(), String> {
    let store = macro_toy::empty_store();
    let note = macro_toy::notes::factory()
        .id("n1")
        .owner("ana")
        .updated_at("t")
        .kind("memo")
        .insert(&store)
        .map_err(|err| err.to_string())?;
    called.note.set(note);
    Ok(())
}

#[then("insert returns the notes row with owner ana")]
fn note_returned(called: &Called) -> Result<(), String> {
    let note = called.note.take().ok_or("no notes row was inserted")?;
    if note.owner == "ana" && note.id == "n1" {
        Ok(())
    } else {
        Err(format!("insert returned {note:?}"))
    }
}

/// The objects factory of the scenarios, every required column set.
fn sage(store: &rusqlite::Connection) -> rusqlite::Result<kessel::objects::Row> {
    kessel::objects::factory()
        .game_id("g1")
        .stable_id("s")
        .payload_hash("p")
        .guid("x")
        .fqn("class.pc.advanced.sage")
        .kind("class")
        .json("{}")
        .insert(store)
}

#[given(
    "the empty test store for kessel's schema.json fixture, holding an objects row with game_id g1, stable_id s, payload_hash p, guid x, fqn class.pc.advanced.sage, kind class and json {{}}"
)]
fn store_with_object(called: &Called) -> Result<(), String> {
    let store = kessel::empty_store();
    sage(&store).map_err(|err| err.to_string())?;
    called.store.set(store);
    Ok(())
}

const STYLE: [(&str, &str); 6] = [
    ("combat_style_id", "g1"),
    ("fqn", "class.pc.advanced.sage"),
    ("fqn_segment", "force_wizard"),
    ("display_segment", "sage"),
    ("faction", "republic"),
    ("attack_type", "force"),
];

#[when(
    "the test inserts a combat_styles factory setting combat_style_id g1, fqn class.pc.advanced.sage, fqn_segment force_wizard, display_segment sage, faction republic and attack_type force"
)]
fn inserts_style(called: &Called) -> Result<(), String> {
    let store = called.store.take().ok_or("no store was taken")?;
    kessel::combat_styles::factory()
        .combat_style_id("g1")
        .fqn("class.pc.advanced.sage")
        .fqn_segment("force_wizard")
        .display_segment("sage")
        .faction("republic")
        .attack_type("force")
        .insert(&store)
        .map_err(|err| err.to_string())?;
    called.store.set(store);
    Ok(())
}

#[then("the store holds a combat_styles row with those values")]
fn holds_style(called: &Called) -> Result<(), String> {
    let store = called.store.take().ok_or("no store was taken")?;
    let names: Vec<&str> = STYLE.iter().map(|(c, _)| *c).collect();
    let held: Vec<String> = store
        .query_row(
            &format!("SELECT {} FROM combat_styles", names.join(", ")),
            [],
            |r| (0..STYLE.len()).map(|at| r.get(at)).collect(),
        )
        .map_err(|err| err.to_string())?;
    let want: Vec<&str> = STYLE.iter().map(|(_, v)| *v).collect();
    if held == want {
        Ok(())
    } else {
        Err(format!("the store holds {held:?}"))
    }
}

#[given(
    "the empty test store for the edge schema.json fixture, where kinds has a TEXT NOT NULL column named type"
)]
fn edge_store(called: &Called) -> Result<(), String> {
    let store = edge::empty_store();
    let (ty, not_null): (String, bool) = store
        .query_row(
            "SELECT type, \"notnull\" FROM pragma_table_info('kinds') WHERE name = 'type'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|err| format!("kinds has no column named type: {err}"))?;
    if ty != "TEXT" || !not_null {
        return Err(format!("kinds.type is {ty}, not null {not_null}"));
    }
    called.store.set(store);
    Ok(())
}

#[when("the test inserts a kinds factory setting id and setting type with r#type")]
fn inserts_kind(called: &Called) -> Result<(), String> {
    let store = called.store.take().ok_or("no store was taken")?;
    let kind = edge::kinds::factory()
        .id("k1")
        .r#type("leaf")
        .insert(&store)
        .map_err(|err| err.to_string())?;
    called.kind.set(kind);
    Ok(())
}

#[then("the test compiles and the inserted kinds row carries the type it was given")]
fn kind_carries_type(called: &Called) -> Result<(), String> {
    let kind = called.kind.take().ok_or("no kinds row was inserted")?;
    if kind.r#type == "leaf" {
        Ok(())
    } else {
        Err(format!("insert returned {kind:?}"))
    }
}

#[given("kessel's schema.json fixture, where the primary key of combat_styles is combat_style_id")]
fn kessel_key() -> Result<(), String> {
    let store = kessel::empty_store();
    let key: Vec<String> = store
        .prepare("SELECT name FROM pragma_table_info('combat_styles') WHERE pk > 0 ORDER BY pk")
        .and_then(|mut stmt| stmt.query_map([], |r| r.get(0))?.collect())
        .map_err(|err| err.to_string())?;
    if key == ["combat_style_id"] {
        Ok(())
    } else {
        Err(format!("the primary key of combat_styles is {key:?}"))
    }
}

/// A crate holding the test that sets only fqn, over kessel's schema.json.
const ONLY_FQN: &str = r#"mod store {
    forgr::schema!("schema.json");
}

pub fn only_fqn() -> forgr::insert::Result<store::combat_styles::Row> {
    let store = store::empty_store();
    store::combat_styles::factory().fqn("x").insert(&store)
}
"#;

#[when("the test inserts a combat_styles factory with only fqn set")]
fn builds_only_fqn(called: &Called) -> Result<(), String> {
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("only-fqn");
    let src = root.join("src");
    std::fs::create_dir_all(&src).map_err(|err| format!("{}: {err}", src.display()))?;
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = [
        (
            root.join("Cargo.toml"),
            format!(
                "[package]\nname = \"only-fqn\"\nedition = \"2024\"\n\n[workspace]\n\n\
                 [dependencies]\nforgr = {{ path = '{}' }}\n",
                manifest.display()
            ),
        ),
        (root.join("src/lib.rs"), ONLY_FQN.to_owned()),
        (
            root.join("schema.json"),
            include_str!("fixtures/kessel/schema.json").to_owned(),
        ),
        (
            root.join("Cargo.lock"),
            include_str!("../../../Cargo.lock").to_owned(),
        ),
    ];
    for (path, text) in files {
        std::fs::write(&path, text).map_err(|err| format!("{}: {err}", path.display()))?;
    }
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let output = std::process::Command::new(cargo)
        .args(["check", "--offline", "--color", "never", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(root.join("target"))
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .map_err(|err| format!("could not run cargo: {err}"))?;
    called.build.set(output);
    Ok(())
}

#[then("the build fails with an error naming combat_styles::col::combat_style_id")]
fn build_names_key(called: &Called) -> Result<(), String> {
    let output = called.build.take().ok_or("nothing was built")?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let named =
        "error[E0277]: a required column was never set: combat_styles::col::combat_style_id";
    if output.status.success() {
        Err("the build succeeded".to_owned())
    } else if stderr.contains(named) {
        Ok(())
    } else {
        Err(format!(
            "the build failed without naming the column:\n{stderr}"
        ))
    }
}

#[given(
    "the empty test store for the macro toy's schema.json fixture, where events.status is INTEGER and events.body is TEXT"
)]
fn events_store(called: &Called) -> Result<(), String> {
    let store = macro_toy::empty_store();
    let types: Vec<String> = store
        .prepare(
            "SELECT type FROM pragma_table_info('events') WHERE name IN ('status', 'body') ORDER BY name",
        )
        .and_then(|mut stmt| stmt.query_map([], |r| r.get(0))?.collect())
        .map_err(|err| err.to_string())?;
    if types != ["TEXT", "INTEGER"] {
        return Err(format!("events.body and events.status are {types:?}"));
    }
    called.store.set(store);
    Ok(())
}

#[when("the test inserts an events factory with id 1, ty 3, status 0 and body root")]
fn inserts_event(called: &Called) -> Result<(), String> {
    let store = called.store.take().ok_or("no store was taken")?;
    // status takes an integer and body takes text: the setters' types follow the columns'.
    let status: i64 = 0;
    let body: &str = "root";
    let event = macro_toy::events::factory()
        .id(1)
        .ty(3)
        .status(status)
        .body(body)
        .insert(&store)
        .map_err(|err| err.to_string())?;
    called.event.set(event);
    Ok(())
}

#[then("the test compiles and the inserted row carries status 0 and body root")]
fn event_carries(called: &Called) -> Result<(), String> {
    let event = called.event.take().ok_or("no events row was inserted")?;
    let (status, body): (i64, String) = (event.status, event.body.clone());
    if status == 0 && body == "root" {
        Ok(())
    } else {
        Err(format!("insert returned {event:?}"))
    }
}

#[given(
    "the empty test store for kessel's schema.json fixture, where objects.template_guid defaults to empty text"
)]
fn store_with_default(called: &Called) -> Result<(), String> {
    let store = kessel::empty_store();
    let default: Option<String> = store
        .query_row(
            "SELECT dflt_value FROM pragma_table_info('objects') WHERE name = 'template_guid'",
            [],
            |r| r.get(0),
        )
        .map_err(|err| err.to_string())?;
    if default.as_deref() != Some("''") {
        return Err(format!("objects.template_guid defaults to {default:?}"));
    }
    called.store.set(store);
    Ok(())
}

#[when(
    "the test inserts an objects factory with game_id g1, stable_id s, payload_hash p, guid x, fqn class.pc.advanced.sage, kind class and json {{}} and no template_guid"
)]
fn inserts_object_without_default(called: &Called) -> Result<(), String> {
    inserts_object(called)
}

#[then("the inserted objects row has an empty template_guid")]
fn empty_template_guid(called: &Called) -> Result<(), String> {
    let object = called.object.take().ok_or("no objects row was inserted")?;
    if object.template_guid.is_empty() {
        Ok(())
    } else {
        Err(format!("template_guid is {:?}", object.template_guid))
    }
}

#[given("the empty test store for kessel's schema.json fixture")]
fn kessel_store(called: &Called) {
    called.store.set(kessel::empty_store());
}

#[when(
    "the test inserts an objects factory with game_id g1, stable_id s, payload_hash p, guid x, fqn class.pc.advanced.sage, kind class and json {{}}"
)]
fn inserts_object(called: &Called) -> Result<(), String> {
    let store = called.store.take().ok_or("no store was taken")?;
    called
        .object
        .set(sage(&store).map_err(|err| err.to_string())?);
    called.store.set(store);
    Ok(())
}

#[then("insert returns the objects row as written, with game_id g1")]
fn object_returned(called: &Called) -> Result<(), String> {
    let store = called.store.take().ok_or("no store was taken")?;
    let object = called.object.take().ok_or("no objects row was inserted")?;
    let stored: (String, String, String, i64, i64) = store
        .query_row(
            "SELECT game_id, fqn, json, for_export, created_at FROM objects",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .map_err(|err| err.to_string())?;
    let returned = (
        object.game_id.clone(),
        object.fqn.clone(),
        object.json.clone(),
        object.for_export,
        object.created_at,
    );
    if object.game_id == "g1" && returned == stored {
        Ok(())
    } else {
        Err(format!(
            "insert returned {object:?}; the store holds {stored:?}"
        ))
    }
}

scenarios!("tests", fixtures = [called: Called]);
