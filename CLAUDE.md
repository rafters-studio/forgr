# forgr

## Acceptance scenarios

Acceptance criteria are Gherkin scenarios, one per criterion, tagged
`@criterion-<id>`.

- **Written from the requirement, never hand-edited.** The builder writes each
  `.feature` file from the requirement's `verification.scenarios`, verbatim,
  with the first line
  `# Generated from the legion requirement. Do not hand-edit this file.`
  There is no generator tool. To change a scenario, change the requirement and
  rewrite the file.
- **Where they live.** In the crate's `tests/`, mirroring `src/`: the scenario
  for behaviour whose entry point is `src/a/b.rs` lives at `tests/a/b.feature`
  in that crate.
- **Runner.** rstest-bdd. `rstest`, `rstest-bdd` and `rstest-bdd-macros` are
  workspace dev-dependencies, so scenarios run as ordinary tests under
  `cargo test` with no separate runner.
- **One runner per crate.** Each crate with scenarios has one
  `tests/scenarios.rs` calling `scenarios!("tests")`, which emits one test per
  scenario in every `.feature` file under `tests/`, and a `build.rs` that prints
  `cargo::rerun-if-changed=tests`, so adding or removing a `.feature` file
  rebuilds the runner.
