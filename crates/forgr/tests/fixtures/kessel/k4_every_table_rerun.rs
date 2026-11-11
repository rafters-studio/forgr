//! Adapted from kessel.rs beside it (RESEARCH 01a1c932-caa1-7561-9bc0-34466568a74d
//! body.inputs tests/kessel.rs), whole, for forgr #9: K4's combat_styles case
//! and its every-table case. What changed and why:
//! - forgr_toy::schema! is forgr::schema!, the built forgr in place of the toy;
//! - the store comes from schema!'s empty_store() in place of the toy's
//!   forgr_toy::check::schema_conn, which forgr does not have (as in #7's
//!   k4_rerun.rs and #8's k5_rerun.rs).
//! all_tables.inc beside this file is the file kessel.rs includes, byte for
//! byte (sha256 fc73de54106776c70749bbc6f9facde6b77b31f0bd326dc605972b99d6a878c0):
//! body.inputs does not carry it, so it was taken from the toy's own tree
//! (shape/toys/kessel-app/tests/all_tables.inc) and is committed unchanged; it
//! mocks each of kessel's 91 tables with mock(3) and no overrides, each in its
//! own store, and counts the inserts that fail or leave foreign_key_check rows.
//! Run it as tests/kessel.rs of a crate holding kessel's schema.json at
//! schema/schema.json and all_tables.inc at tests/all_tables.inc, with forgr
//! (features = ["fake"]) as a dev-dependency and rusqlite as a dependency:
//! cargo test -- --nocapture.
forgr::schema!("schema/schema.json");
use Override::Fixed;
use rusqlite::Connection;

fn store() -> Connection {
    empty_store()
}

/// kessel's testutil::seed_combat_styles_for, without the hand-seeded objects rows.
#[test]
fn combat_styles_without_seeding_objects() {
    let conn = store();
    let styles = combat_styles::mock(8).with(combat_styles::Overrides { attack_type: Fixed("force".into()), ..Default::default() }).insert(&conn).unwrap();
    let parents: i64 = conn.query_row("SELECT count(*) FROM combat_styles c JOIN objects o ON o.game_id = c.combat_style_id", [], |r| r.get(0)).unwrap();
    println!("K4 combat_styles {} with parents in objects {}", styles.len(), parents);
    let fk: i64 = conn.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r.get(0)).unwrap();
    assert_eq!(fk, 0);
    assert_eq!(parents, 8);
}

/// Every table that can be mocked with no overrides: count what goes in and what does not.
#[test]
fn every_table_mocks() {
    let names: Vec<String> = store().prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name").unwrap().query_map([], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect();
    println!("K4b tables {}", names.len());
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/all_tables.inc"));
}
