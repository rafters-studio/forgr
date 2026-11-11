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
    notes: Slot<Vec<macro_toy::notes::Row>>,
    events: Slot<Vec<macro_toy::events::Row>>,
    styles: Slot<Vec<kessel::combat_styles::Row>>,
    node: Slot<edge::nodes::Row>,
    nodes: Slot<Vec<edge::nodes::Row>>,
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
fn store_with_object_given(called: &Called) -> Result<(), String> {
    store_with_object(called)
}

#[given(
    "the empty test store for kessel's schema.json fixture, holding an objects row with game_id g1, stable_id s, payload_hash p, guid x, fqn class.pc.advanced.sage, kind class and json {{}} inserted by a factory"
)]
fn store_with_object_by_factory(called: &Called) -> Result<(), String> {
    store_with_object(called)
}

/// The kessel store holding the objects row `sage` inserts by its factory.
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
fn inserts_style_setting(called: &Called) -> Result<(), String> {
    inserts_style(called)
}

#[when(
    "the test inserts a combat_styles factory with combat_style_id g1, fqn class.pc.advanced.sage, fqn_segment force_wizard, display_segment sage, faction republic and attack_type force"
)]
fn inserts_style_with(called: &Called) -> Result<(), String> {
    inserts_style(called)
}

/// Inserts the combat_styles factory of the scenarios, every required column set.
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

/// Checks a crate named `name` that depends on forgr, with `features`, and
/// holds `lib` as its src/lib.rs and `schema` as its schema.json.
fn check_crate(
    name: &str,
    features: &str,
    lib: &str,
    schema: &str,
) -> Result<std::process::Output, String> {
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let src = root.join("src");
    std::fs::create_dir_all(&src).map_err(|err| format!("{}: {err}", src.display()))?;
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = [
        (
            root.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{name}\"\nedition = \"2024\"\n\n[workspace]\n\n\
                 [dependencies]\nforgr = {{ path = '{}', features = [{features}] }}\n",
                manifest.display()
            ),
        ),
        (root.join("src/lib.rs"), lib.to_owned()),
        (root.join("schema.json"), schema.to_owned()),
        (
            root.join("Cargo.lock"),
            include_str!("../../../Cargo.lock").to_owned(),
        ),
    ];
    for (path, text) in files {
        std::fs::write(&path, text).map_err(|err| format!("{}: {err}", path.display()))?;
    }
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    std::process::Command::new(cargo)
        .args(["check", "--offline", "--color", "never", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(root.join("target"))
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .map_err(|err| format!("could not run cargo: {err}"))
}

#[when("the test inserts a combat_styles factory with only fqn set")]
fn builds_only_fqn(called: &Called) -> Result<(), String> {
    called.build.set(check_crate(
        "only-fqn",
        "",
        ONLY_FQN,
        include_str!("fixtures/kessel/schema.json"),
    )?);
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
    called.store.set(store);
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

#[given("the empty test store for the macro toy's schema.json fixture")]
fn macro_store(called: &Called) {
    called.store.set(macro_toy::empty_store());
}

/// The macro toy's empty test store, once `sql` over it returns `want`.
fn macro_store_where(called: &Called, sql: &str, want: &str) -> Result<(), String> {
    let store = macro_toy::empty_store();
    let got: String = store
        .query_row(sql, [], |r| r.get(0))
        .map_err(|err| format!("{sql}: {err}"))?;
    if got != want {
        return Err(format!("{sql} gives {got}, not {want}"));
    }
    called.store.set(store);
    Ok(())
}

#[given(
    "the empty test store for the macro toy's schema.json fixture, where the primary key of notes is TEXT"
)]
fn notes_text_key(called: &Called) -> Result<(), String> {
    macro_store_where(
        called,
        "SELECT group_concat(name || ' ' || type) FROM pragma_table_info('notes') WHERE pk > 0",
        "id TEXT",
    )
}

#[given(
    "the empty test store for the macro toy's schema.json fixture, where the primary key of events is INTEGER"
)]
fn events_integer_key(called: &Called) -> Result<(), String> {
    macro_store_where(
        called,
        "SELECT group_concat(name || ' ' || type) FROM pragma_table_info('events') WHERE pk > 0",
        "id INTEGER",
    )
}

#[given(
    "the empty test store for the macro toy's schema.json fixture, where notes.body is nullable and not a foreign key"
)]
fn notes_nullable_body(called: &Called) -> Result<(), String> {
    macro_store_where(
        called,
        "SELECT (SELECT \"notnull\" FROM pragma_table_info('notes') WHERE name = 'body') || ' ' || \
         (SELECT count(*) FROM pragma_foreign_key_list('notes') WHERE \"from\" = 'body')",
        "0 0",
    )
}

#[given(
    "the empty test store for the macro toy's schema.json fixture, where events.source is a nullable foreign key"
)]
fn events_nullable_source(called: &Called) -> Result<(), String> {
    macro_store_where(
        called,
        "SELECT (SELECT \"notnull\" FROM pragma_table_info('events') WHERE name = 'source') || ' ' || \
         (SELECT \"table\" FROM pragma_foreign_key_list('events') WHERE \"from\" = 'source')",
        "0 events",
    )
}

/// Inserts `notes::mock(n)` into the scenario's store.
fn mock_notes(called: &Called, n: usize) -> Result<(), String> {
    let store = called.store.take().ok_or("no store was taken")?;
    let rows = macro_toy::notes::mock(n)
        .insert(&store)
        .map_err(|err| err.to_string())?;
    called.notes.set(rows);
    called.store.set(store);
    Ok(())
}

#[when("the test inserts notes::mock(0)")]
fn mock_no_notes(called: &Called) -> Result<(), String> {
    mock_notes(called, 0)
}

#[when("the test inserts notes::mock(8)")]
fn mock_eight_notes(called: &Called) -> Result<(), String> {
    mock_notes(called, 8)
}

#[when("the test inserts notes::mock(50)")]
fn mock_fifty_notes(called: &Called) -> Result<(), String> {
    mock_notes(called, 50)
}

#[when("the test inserts events::mock(8)")]
fn mock_eight_events(called: &Called) -> Result<(), String> {
    let store = called.store.take().ok_or("no store was taken")?;
    let rows = macro_toy::events::mock(8)
        .insert(&store)
        .map_err(|err| err.to_string())?;
    called.events.set(rows);
    called.store.set(store);
    Ok(())
}

#[when("the test inserts events::mock(8) with no override for source")]
fn mock_events_without_source(called: &Called) -> Result<(), String> {
    mock_eight_events(called)
}

#[then("insert returns no rows")]
fn returns_no_rows(called: &Called) -> Result<(), String> {
    let rows = called.notes.take().ok_or("no mock was inserted")?;
    let store = called.store.take().ok_or("no store was taken")?;
    let held: i64 = store
        .query_row("SELECT count(*) FROM notes", [], |r| r.get(0))
        .map_err(|err| err.to_string())?;
    if rows.is_empty() && held == 0 {
        Ok(())
    } else {
        Err(format!("insert returned {rows:?}; the store holds {held}"))
    }
}

/// Whether `id` is a UUIDv7: 8-4-4-4-12 lowercase hex, version 7, variant 10.
fn is_uuid7(id: &str) -> bool {
    let parts: Vec<&str> = id.split('-').collect();
    parts.iter().map(|p| p.len()).eq([8, 4, 4, 4, 12])
        && parts.iter().all(|p| {
            p.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
        && parts[2].starts_with('7')
        && parts[3].starts_with(['8', '9', 'a', 'b'])
}

#[then("every notes row's id is a UUIDv7")]
fn notes_ids_are_uuid7(called: &Called) -> Result<(), String> {
    let rows = called.notes.take().ok_or("no mock was inserted")?;
    let bad: Vec<&str> = rows
        .iter()
        .map(|r| r.id.as_str())
        .filter(|id| !is_uuid7(id))
        .collect();
    if rows.len() == 8 && bad.is_empty() {
        Ok(())
    } else {
        Err(format!("{} rows; not UUIDv7: {bad:?}", rows.len()))
    }
}

#[then("every events row's id is a positive integer")]
fn events_ids_are_positive(called: &Called) -> Result<(), String> {
    let rows = called.events.take().ok_or("no mock was inserted")?;
    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    if ids.len() == 8 && ids.iter().all(|&id| id > 0) {
        Ok(())
    } else {
        Err(format!("the events ids are {ids:?}"))
    }
}

#[then("some notes rows have body null")]
fn some_bodies_null(called: &Called) -> Result<(), String> {
    let rows = called.notes.take().ok_or("no mock was inserted")?;
    let nulls = rows.iter().filter(|r| r.body.is_none()).count();
    if rows.len() == 50 && nulls > 0 {
        Ok(())
    } else {
        Err(format!(
            "{nulls} of {} notes rows have body null",
            rows.len()
        ))
    }
}

#[then("every events row's source is NULL")]
fn sources_null(called: &Called) -> Result<(), String> {
    let rows = called.events.take().ok_or("no mock was inserted")?;
    let sources: Vec<Option<i64>> = rows.iter().map(|r| r.source).collect();
    if sources.len() == 8 && sources.iter().all(Option::is_none) {
        Ok(())
    } else {
        Err(format!("the events sources are {sources:?}"))
    }
}

#[when("the test inserts combat_styles::mock(8)")]
fn mock_eight_styles(called: &Called) -> Result<(), String> {
    let store = called.store.take().ok_or("no store was taken")?;
    let rows = kessel::combat_styles::mock(8)
        .insert(&store)
        .map_err(|err| err.to_string())?;
    called.styles.set(rows);
    called.store.set(store);
    Ok(())
}

/// The value of `sql`, a count, over the scenario's store.
fn count(called: &Called, sql: &str) -> Result<i64, String> {
    let store = called.store.take().ok_or("no store was taken")?;
    let n = store
        .query_row(sql, [], |r| r.get(0))
        .map_err(|err| format!("{sql}: {err}"));
    called.store.set(store);
    n
}

#[then("the store holds 8 combat_styles rows")]
fn holds_eight_styles(called: &Called) -> Result<(), String> {
    match count(called, "SELECT count(*) FROM combat_styles")? {
        8 => Ok(()),
        n => Err(format!("the store holds {n} combat_styles rows")),
    }
}

#[then("insert returns 8 combat_styles rows, each with its combat_style_id")]
fn returns_eight_styles(called: &Called) -> Result<(), String> {
    let rows = called.styles.take().ok_or("no mock was inserted")?;
    let store = called.store.take().ok_or("no store was taken")?;
    let mut held: Vec<String> = store
        .prepare("SELECT combat_style_id FROM combat_styles")
        .and_then(|mut stmt| stmt.query_map([], |r| r.get(0))?.collect())
        .map_err(|err| err.to_string())?;
    let mut returned: Vec<String> = rows.iter().map(|r| r.combat_style_id.clone()).collect();
    held.sort();
    returned.sort();
    if rows.len() == 8 && returned == held {
        Ok(())
    } else {
        Err(format!(
            "insert returned the ids {returned:?}; the store holds {held:?}"
        ))
    }
}

#[given(
    "the empty test store for the macro toy's schema.json fixture, where note_tags has the required composite foreign key (owner, name) to tags"
)]
fn note_tags_composite_key(called: &Called) -> Result<(), String> {
    macro_store_where(
        called,
        "SELECT group_concat(k.\"from\" || '>' || k.\"to\" || ' ' || c.\"notnull\", ', ') \
         FROM pragma_foreign_key_list('note_tags') k \
         JOIN pragma_table_info('note_tags') c ON c.name = k.\"from\" \
         WHERE k.\"table\" = 'tags'",
        "owner>owner 1, name>name 1",
    )
}

#[when("the test inserts note_tags::mock(8)")]
fn mock_eight_note_tags(called: &Called) -> Result<(), String> {
    let store = called.store.take().ok_or("no store was taken")?;
    macro_toy::note_tags::mock(8)
        .insert(&store)
        .map_err(|err| err.to_string())?;
    called.store.set(store);
    Ok(())
}

#[then("the store holds 8 tags rows and no foreign-key violations")]
fn eight_tags_no_violations(called: &Called) -> Result<(), String> {
    let tags = count(called, "SELECT count(*) FROM tags")?;
    let violations = count(called, "SELECT count(*) FROM pragma_foreign_key_check")?;
    if tags == 8 && violations == 0 {
        Ok(())
    } else {
        Err(format!(
            "the store holds {tags} tags rows and {violations} foreign-key violations"
        ))
    }
}

#[when("the test inserts events::mock(3..32)")]
fn mock_ranged_events(called: &Called) -> Result<(), String> {
    let store = called.store.take().ok_or("no store was taken")?;
    macro_toy::events::mock(3..32)
        .insert(&store)
        .map_err(|err| err.to_string())?;
    called.store.set(store);
    Ok(())
}

#[then("the store holds at least 3 and at most 31 events rows")]
fn holds_ranged_events(called: &Called) -> Result<(), String> {
    match count(called, "SELECT count(*) FROM events")? {
        3..=31 => Ok(()),
        n => Err(format!("the store holds {n} events rows")),
    }
}

#[when("the test inserts combat_styles::mock(8) with attack_type Fixed to force")]
fn mock_styles_fixed(called: &Called) -> Result<(), String> {
    use kessel::Override::Fixed;
    let store = called.store.take().ok_or("no store was taken")?;
    kessel::combat_styles::mock(8)
        .with(kessel::combat_styles::Overrides {
            attack_type: Fixed("force".into()),
            ..Default::default()
        })
        .insert(&store)
        .map_err(|err| err.to_string())?;
    called.store.set(store);
    Ok(())
}

#[then("every combat_styles row has attack_type force")]
fn styles_all_force(called: &Called) -> Result<(), String> {
    let held = count(called, "SELECT count(*) FROM combat_styles")?;
    let force = count(
        called,
        "SELECT count(*) FROM combat_styles WHERE attack_type = 'force'",
    )?;
    if held == 8 && force == 8 {
        Ok(())
    } else {
        Err(format!(
            "{force} of {held} combat_styles rows have attack_type force"
        ))
    }
}

/// Inserts `events::mock(12)` with `overrides` into the scenario's store.
fn mock_events_with(
    called: &Called,
    overrides: macro_toy::events::Overrides,
) -> Result<(), String> {
    let store = called.store.take().ok_or("no store was taken")?;
    let rows = macro_toy::events::mock(12)
        .with(overrides)
        .insert(&store)
        .map_err(|err| err.to_string())?;
    called.events.set(rows);
    called.store.set(store);
    Ok(())
}

#[when("the test inserts events::mock(12) with status Pick from -1, 0, 1 and 2")]
fn mock_events_picked(called: &Called) -> Result<(), String> {
    use macro_toy::Override::Pick;
    mock_events_with(
        called,
        macro_toy::events::Overrides {
            status: Pick(vec![-1, 0, 1, 2]),
            ..Default::default()
        },
    )
}

#[then("every events row has a status of -1, 0, 1 or 2")]
fn statuses_picked(called: &Called) -> Result<(), String> {
    let rows = called.events.take().ok_or("no mock was inserted")?;
    let held = count(called, "SELECT count(*) FROM events")?;
    let listed = count(
        called,
        "SELECT count(*) FROM events WHERE status IN (-1, 0, 1, 2)",
    )?;
    if rows.len() == 12 && held == 12 && listed == 12 {
        Ok(())
    } else {
        Err(format!(
            "{listed} of {held} events rows have a listed status"
        ))
    }
}

#[when("the test inserts events::mock(12) with body made per row as kid and the row's number")]
fn mock_events_per_row(called: &Called) -> Result<(), String> {
    use macro_toy::Override;
    mock_events_with(
        called,
        macro_toy::events::Overrides {
            body: Override::each(|r| format!("kid {}", r.row)),
            ..Default::default()
        },
    )
}

#[then("each events row's body is kid and that row's number")]
fn bodies_per_row(called: &Called) -> Result<(), String> {
    let rows = called.events.take().ok_or("no mock was inserted")?;
    let store = called.store.take().ok_or("no store was taken")?;
    let held: Vec<String> = store
        .prepare("SELECT body FROM events ORDER BY rowid")
        .and_then(|mut stmt| stmt.query_map([], |r| r.get(0))?.collect())
        .map_err(|err| err.to_string())?;
    let want: Vec<String> = (0..12).map(|n| format!("kid {n}")).collect();
    let returned: Vec<String> = rows.iter().map(|r| r.body.clone()).collect();
    // events.id is a random rowid, so the store's order is by id; compare as sets there.
    let mut sorted = held.clone();
    sorted.sort();
    let mut sorted_want = want.clone();
    sorted_want.sort();
    if returned == want && sorted == sorted_want {
        Ok(())
    } else {
        Err(format!(
            "insert returned {returned:?}; the store holds {held:?}"
        ))
    }
}

#[given(
    "the empty test store for the macro toy's schema.json fixture, holding an events row inserted by a factory with id 1, ty 3, status 0 and body root"
)]
fn store_with_root_event(called: &Called) -> Result<(), String> {
    let store = macro_toy::empty_store();
    let root = macro_toy::events::factory()
        .id(1)
        .ty(3)
        .status(0)
        .body("root")
        .insert(&store)
        .map_err(|err| err.to_string())?;
    called.event.set(root);
    called.store.set(store);
    Ok(())
}

#[when("the test inserts events::mock(12) with source Fixed to 1")]
fn mock_events_to_root(called: &Called) -> Result<(), String> {
    use macro_toy::Override::Fixed;
    mock_events_with(
        called,
        macro_toy::events::Overrides {
            source: Fixed(Some(1)),
            ..Default::default()
        },
    )
}

#[then("every new events row's source is 1")]
fn sources_root(called: &Called) -> Result<(), String> {
    let root = called.event.take().ok_or("no events row was inserted")?;
    let rows = called.events.take().ok_or("no mock was inserted")?;
    let new = count(called, "SELECT count(*) FROM events WHERE id <> 1")?;
    let attached = count(
        called,
        "SELECT count(*) FROM events WHERE id <> 1 AND source = 1",
    )?;
    let sources: Vec<Option<i64>> = rows.iter().map(|r| r.source).collect();
    if root.id == 1
        && rows.len() == 12
        && sources.iter().all(|s| *s == Some(1))
        && new == 12
        && attached == 12
    {
        Ok(())
    } else {
        Err(format!(
            "{attached} of {new} new events rows have source 1; insert returned {sources:?}"
        ))
    }
}

#[given(
    "the empty test store for the edge schema.json fixture, holding one nodes row inserted by a factory whose parent_id is its own id"
)]
fn store_with_root_node(called: &Called) -> Result<(), String> {
    let store = edge::empty_store();
    let root = edge::nodes::factory()
        .id("root")
        .parent_id("root")
        .insert(&store)
        .map_err(|err| err.to_string())?;
    if root.parent_id != root.id {
        return Err(format!("the nodes row is {root:?}"));
    }
    called.node.set(root);
    called.store.set(store);
    Ok(())
}

#[when("the test inserts nodes::mock(8) with parent_id Fixed to that row's id")]
fn mock_nodes_to_root(called: &Called) -> Result<(), String> {
    use edge::Override::Fixed;
    let root = called.node.take().ok_or("no nodes row was inserted")?;
    let store = called.store.take().ok_or("no store was taken")?;
    let rows = edge::nodes::mock(8)
        .with(edge::nodes::Overrides {
            parent_id: Fixed(root.id.clone()),
            ..Default::default()
        })
        .insert(&store)
        .map_err(|err| err.to_string())?;
    called.nodes.set(rows);
    called.node.set(root);
    called.store.set(store);
    Ok(())
}

#[then("every new nodes row's parent_id is that row's id")]
fn parents_root(called: &Called) -> Result<(), String> {
    let root = called.node.take().ok_or("no nodes row was inserted")?;
    let rows = called.nodes.take().ok_or("no mock was inserted")?;
    let new = count(called, "SELECT count(*) FROM nodes WHERE id <> 'root'")?;
    let attached = count(
        called,
        "SELECT count(*) FROM nodes WHERE id <> 'root' AND parent_id = 'root'",
    )?;
    if rows.len() == 8 && rows.iter().all(|r| r.parent_id == root.id) && new == 8 && attached == 8 {
        Ok(())
    } else {
        Err(format!(
            "{attached} of {new} new nodes rows have parent_id {}",
            root.id
        ))
    }
}

scenarios!("tests", fixtures = [called: Called]);

#[then("the combat_styles row is written and refers to the objects row g1")]
fn style_refers_to_object(called: &Called) -> Result<(), String> {
    let store = called.store.take().ok_or("no store was taken")?;
    let referring = count_in(
        &store,
        "SELECT count(*) FROM combat_styles c JOIN objects o ON o.game_id = c.combat_style_id \
         WHERE c.combat_style_id = 'g1'",
    )?;
    let violations = count_in(&store, "SELECT count(*) FROM pragma_foreign_key_check")?;
    if referring == 1 && violations == 0 {
        Ok(())
    } else {
        Err(format!(
            "{referring} combat_styles rows refer to the objects row g1, \
             with {violations} foreign-key violations"
        ))
    }
}

/// The one integer `sql` over `store` returns.
fn count_in(store: &rusqlite::Connection, sql: &str) -> Result<i64, String> {
    store
        .query_row(sql, [], |r| r.get(0))
        .map_err(|err| format!("{sql}: {err}"))
}

#[then("the store holds that events row")]
fn holds_event(called: &Called) -> Result<(), String> {
    let store = called.store.take().ok_or("no store was taken")?;
    let event = called.event.take().ok_or("no events row was inserted")?;
    let held: Vec<(i64, i64, i64, String)> = store
        .prepare("SELECT id, ty, status, body FROM events")
        .and_then(|mut stmt| {
            stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
                .collect()
        })
        .map_err(|err| err.to_string())?;
    let want = (1, 3, 0, "root".to_owned());
    let returned = (event.id, event.ty, event.status, event.body.clone());
    if held == [want.clone()] && returned == want {
        Ok(())
    } else {
        Err(format!(
            "the store holds {held:?}; insert returned {event:?}"
        ))
    }
}

/// The macro toy's store, once it holds notes_fts as an FTS5 virtual table.
fn holds_notes_fts() -> Result<(), String> {
    let sql: String = macro_toy::empty_store()
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'notes_fts'",
            [],
            |r| r.get(0),
        )
        .map_err(|err| format!("the macro toy's schema.json has no notes_fts: {err}"))?;
    if sql.to_ascii_lowercase().contains("using fts5") {
        Ok(())
    } else {
        Err(format!("notes_fts is not an FTS5 table: {sql}"))
    }
}

#[given("the macro toy's schema.json fixture, which holds the FTS5 table notes_fts")]
fn macro_toy_with_fts() -> Result<(), String> {
    holds_notes_fts()
}

#[given(
    "the macro toy's schema.json fixture, which holds the FTS5 table notes_fts, and forgr's fake feature on"
)]
fn macro_toy_with_fts_and_fake() -> Result<(), String> {
    // The crate the When builds depends on forgr with its fake feature.
    holds_notes_fts()
}

/// A crate over the macro toy's schema.json whose code calls `call` on notes_fts.
fn calls_notes_fts(name: &str, features: &str, call: &str) -> Result<std::process::Output, String> {
    let lib = format!(
        "mod store {{\n    forgr::schema!(\"schema.json\");\n}}\n\n\
         pub fn calls_notes_fts() {{\n    let _ = store::notes_fts::{call};\n}}\n"
    );
    check_crate(
        name,
        features,
        &lib,
        include_str!("fixtures/macro/schema.json"),
    )
}

#[when("a test calls notes_fts::factory()")]
fn calls_fts_factory(called: &Called) -> Result<(), String> {
    called
        .build
        .set(calls_notes_fts("fts-factory", "", "factory()")?);
    Ok(())
}

#[when("a test calls notes_fts::mock(8)")]
fn calls_fts_mock(called: &Called) -> Result<(), String> {
    called
        .build
        .set(calls_notes_fts("fts-mock", "\"fake\"", "mock(8)")?);
    Ok(())
}

#[then("the test fails to compile")]
fn fails_to_compile(called: &Called) -> Result<(), String> {
    let output = called.build.take().ok_or("nothing was built")?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Failing for notes_fts, not for some other reason the crate could fail.
    let for_fts = stderr
        .lines()
        .any(|line| line.starts_with("error[") && line.contains("`notes_fts`"));
    if output.status.success() {
        Err("the test compiled".to_owned())
    } else if for_fts {
        Ok(())
    } else {
        Err(format!(
            "the test failed to compile, but not for notes_fts:\n{stderr}"
        ))
    }
}
