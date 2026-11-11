//! The macro toy's M2 and M3 cases (RESEARCH 01a1c917-ba6c-7553-bffe-c648a5f16f44),
//! re-run against the built forgr for forgr #11. The toy's own test file (its
//! locator, toys/app/tests/forgr.rs) is not in body.inputs, so this file is
//! written from the M2 claim and its evidence, "error[E0599]: no method named
//! `owner` found for struct `notes::Factory<R0, R1, R2>` --> app/tests/forgr.rs:20:42;
//! same for notes::Mock at :45:32": one test uses notes.owner through a factory
//! setter, one through the mock. The toy's mock took chained setters; the
//! operator replaced them with the per-table Overrides struct (the macro toy's
//! caveat, "Superseded in part ... mocks take a per-table Overrides struct"), so
//! the mock's use of owner is an Overrides field here. The call is inside a
//! module, as FR-FORGR-002 says. m2_rerun.sh runs it as tests/forgr.rs of a crate
//! holding the toy's schema.json (beside this file, sha256 8025ba9f...) at
//! schema/schema.json, with forgr (features = ["fake"]) as a dev-dependency.
mod tables {
    forgr::schema!("schema/schema.json");
}
use tables::{Override::Fixed, empty_store, notes};

#[test]
fn a_note_through_the_factory() {
    let conn = empty_store();
    let note = notes::factory().id("n1").owner("ana").updated_at("t").kind("memo").insert(&conn).unwrap();
    assert_eq!(note.kind, "memo");
}

#[test]
fn notes_through_the_mock() {
    let conn = empty_store();
    let rows = notes::mock(5).with(notes::Overrides { owner: Fixed("bo".into()), ..Default::default() }).insert(&conn).unwrap();
    assert_eq!(rows.len(), 5);
}
