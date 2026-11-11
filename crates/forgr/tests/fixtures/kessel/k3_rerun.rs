//! Adapted from the kessel toy's tests/kessel_sql.rs (RESEARCH
//! 01a1c932-caa1-7561-9bc0-34466568a74d body.inputs), its K3 case, for forgr #10.
//! What changed and why:
//! - kessel is read from the checkout at $KESSEL_DIR (huttspawn/kessel at
//!   5d75f98) in place of the toy's ../../src/kessel/kessel, so the re-run does
//!   not depend on where the toy kept kessel; with KESSEL_DIR unset the test says
//!   so and is skipped;
//! - forgr::check::run, the entry schema!'s emitted test calls, replaces the
//!   toy's forgr_toy::check::check(dir, schema, &["src"]): it reads the members
//!   of kessel's workspace from its manifest and scans each member's non-test
//!   src (FR-FORGR-007), where the toy scanned the kessel crate's src alone; it
//!   prints the report the toy's println! lines summarised, and panics when a
//!   statement fails;
//! - schema.json is kessel's, beside this file (sha256 e72408fe...).
//! Run it as tests/kessel_sql.rs of a crate holding kessel's schema.json at
//! schema/schema.json, with forgr as a dev-dependency:
//! KESSEL_DIR=<kessel checkout> cargo test -- --nocapture.
#[test]
fn kessel_src_against_its_schema() {
    let Some(kessel) = std::env::var_os("KESSEL_DIR") else {
        println!("K3 skipped: KESSEL_DIR is unset");
        return;
    };
    let crate_dir = std::path::Path::new(&kessel).join("kessel");
    let schema = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/schema/schema.json"));
    forgr::check::run(&crate_dir.to_string_lossy(), schema);
}
