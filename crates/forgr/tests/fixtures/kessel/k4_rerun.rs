//! Adapted from kessel.rs beside it (RESEARCH 01a1c932-caa1-7561-9bc0-34466568a74d
//! body.inputs tests/kessel.rs), its K4 combat_styles case only, for forgr #7.
//! What changed and why:
//! - the store comes from schema!'s empty_store() in place of the toy's
//!   forgr_toy::check::schema_conn, which forgr does not have;
//! - the attack_type Fixed override is dropped: Overrides is #8, and
//!   attack_type has no CHECK and no foreign key, so it cannot change the
//!   parent count or the foreign_key_check rows K4 measures;
//! - the every-table case (all_tables.inc) is left out: it is #9.
//! Run it as tests/kessel.rs of a crate holding kessel's schema.json at
//! schema/schema.json, with forgr (features = ["fake"]) and rusqlite as
//! dev-dependencies: cargo test -- --nocapture.
mod store {
    forgr::schema!("schema/schema.json");
}
use store::*;

/// kessel's testutil::seed_combat_styles_for, without the hand-seeded objects rows.
#[test]
fn combat_styles_without_seeding_objects() {
    let conn = empty_store();
    let styles = combat_styles::mock(8).insert(&conn).unwrap();
    let parents: i64 = conn.query_row("SELECT count(*) FROM combat_styles c JOIN objects o ON o.game_id = c.combat_style_id", [], |r| r.get(0)).unwrap();
    println!("K4 combat_styles {} with parents in objects {}", styles.len(), parents);
    let fk: i64 = conn.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r.get(0)).unwrap();
    println!("K4 foreign_key_check rows {fk}");
    assert_eq!(fk, 0);
    assert_eq!(parents, 8);
}
