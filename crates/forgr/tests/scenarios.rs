//! Acceptance-scenario runner for the `forgr` crate.
//!
//! `scenarios!` discovers every `.feature` file under `tests/` and emits one
//! test per scenario. Step definitions must compile into this binary.

use rstest_bdd_macros::scenarios;

scenarios!("tests");
