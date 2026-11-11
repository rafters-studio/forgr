//! The workspace toy's W7 case (RESEARCH 01a1c94b-8c34-7363-bafb-c60062cf77b1),
//! written for forgr #11. The toy's own file (its locator,
//! toys/app/tests/forgr.rs) is not in body.inputs, so this file is written from
//! the W7 claim, "schema!'s items go into the scope it is called in; at a test
//! file's root beside `use forgr::fake::Override` they collide ... a module
//! around the call avoids it": the call is inside a module, beside
//! `use forgr::fake::Override` at the file's root, over the store's schema.json
//! (the macro toy's). w_rerun.sh runs it as tests/forgr.rs of a crate holding
//! that schema.json at schema/schema.json, with forgr (features = ["fake"]) as a
//! dev-dependency, and runs the same file with the call at the root for the
//! E0252 the toy measured.
use forgr::fake::Override;

mod tables {
    forgr::schema!("schema/schema.json");
}

#[test]
fn the_call_inside_a_module_compiles_beside_override() {
    let conn = tables::empty_store();
    let rows = tables::notes::mock(3)
        .with(tables::notes::Overrides { owner: Override::Fixed("bo".into()), ..Default::default() })
        .insert(&conn)
        .unwrap();
    println!("W7 notes {} with owner bo {}", rows.len(), rows.iter().filter(|r| r.owner == "bo").count());
    assert!(rows.iter().all(|r| r.owner == "bo"));
}
