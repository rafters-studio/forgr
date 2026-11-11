//! The macro toy's M8 cases (RESEARCH 01a1c917-ba6c-7553-bffe-c648a5f16f44),
//! re-run against the built forgr for #8. The toy's own M8 test is not in its
//! body.inputs (its locator is toys/app/RESULTS.md), so this file is written
//! from the M8 claim and its evidence, "events.status outside -1..2: 15 of 15;
//! notes.kind only memo/todo/log over 50 rows", over the toy's schema.json as
//! body.inputs holds it (beside this file, sha256 8025ba9f...):
//! - notes.kind, TEXT NOT NULL CHECK (kind IN ('memo', 'todo', 'log')): 50 rows
//!   of notes::mock, counting the rows whose kind is listed;
//! - events.status, INTEGER NOT NULL with no CHECK, an int enum meant as -1..2:
//!   15 rows of events::mock with no override, counting the rows outside it.
//! Run it as tests/m8.rs of a crate holding the toy's schema.json at
//! schema/schema.json, with forgr (features = ["fake"]) and rusqlite as
//! dev-dependencies: cargo test -- --nocapture.
forgr::schema!("schema/schema.json");

#[test]
fn notes_kind_only_listed_values() {
    let conn = empty_store();
    let notes = notes::mock(50).insert(&conn).unwrap();
    let listed = notes.iter().filter(|n| ["memo", "todo", "log"].contains(&n.kind.as_str())).count();
    println!("M8 notes.kind only memo/todo/log: {listed} of {} rows", notes.len());
    assert_eq!((notes.len(), listed), (50, 50));
}

#[test]
fn events_status_any_integer() {
    let conn = empty_store();
    let events = events::mock(15).insert(&conn).unwrap();
    let outside = events.iter().filter(|e| !(-1..=2).contains(&e.status)).count();
    println!("M8 events.status outside -1..2: {outside} of {}", events.len());
    assert_eq!(events.len(), 15);
}
