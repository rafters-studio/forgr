//! Adapted from kessel.rs beside it (RESEARCH 01a1c932-caa1-7561-9bc0-34466568a74d
//! body.inputs tests/kessel.rs), its combat_styles case with the Overrides
//! struct, for forgr #8 (K5). What changed and why:
//! - forgr_toy::schema! is forgr::schema!, the built forgr in place of the toy;
//! - the store comes from schema!'s empty_store() in place of the toy's
//!   forgr_toy::check::schema_conn, which forgr does not have (as in #7's
//!   k4_rerun.rs);
//! - the Overrides literal also sets display_segment with Override::each(|r| ...)
//!   and string_id with Override::Each(Box::new(|r| ...)): K5's two Each forms,
//!   quoted in its claim's evidence (locator toys/app/tests/forgr.rs, which is
//!   not in body.inputs), so the re-run shows they infer with no annotation;
//!   string_id is a nullable INTEGER, so its closure wraps r.pick([1, 2]) in Some;
//! - the checks that each override took (every attack_type force,
//!   display_segment per row, string_id 1 or 2) are added after K4's;
//! - the every-table case (all_tables.inc) is left out: it is #9.
//! Run it as tests/kessel.rs of a crate holding kessel's schema.json at
//! schema/schema.json, with forgr (features = ["fake"]) and rusqlite as
//! dev-dependencies: cargo test -- --nocapture.
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
    let styles = combat_styles::mock(8).with(combat_styles::Overrides {
        attack_type: Fixed("force".into()),
        display_segment: Override::each(|r| format!("style {}", r.row)),
        string_id: Override::Each(Box::new(|r| Some(r.pick([1, 2])))),
        ..Default::default()
    }).insert(&conn).unwrap();
    let parents: i64 = conn.query_row("SELECT count(*) FROM combat_styles c JOIN objects o ON o.game_id = c.combat_style_id", [], |r| r.get(0)).unwrap();
    println!("K4 combat_styles {} with parents in objects {}", styles.len(), parents);
    let fk: i64 = conn.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r.get(0)).unwrap();
    assert_eq!(fk, 0);
    assert_eq!(parents, 8);
    let force = styles.iter().filter(|s| s.attack_type == "force").count();
    let per_row = styles.iter().enumerate().filter(|(n, s)| s.display_segment == format!("style {n}")).count();
    let picked = styles.iter().filter(|s| matches!(s.string_id, Some(1 | 2))).count();
    println!("K5 attack_type force {force} of {}; display_segment per row {per_row}; string_id 1 or 2 {picked}", styles.len());
    assert_eq!((force, per_row, picked), (8, 8, 8));
}
