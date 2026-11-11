forgr_toy::schema!("schema/schema.json");
use Override::Fixed;
use rusqlite::Connection;

fn store() -> Connection {
    let c = forgr_toy::check::schema_conn(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/schema/schema.json")).unwrap()).unwrap();
    c.execute_batch("PRAGMA foreign_keys = ON").unwrap();
    c
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
