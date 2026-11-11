#!/bin/sh
# The workspace toy's W1, W2, W3, W6 and W7 cases (RESEARCH
# 01a1c94b-8c34-7363-bafb-c60062cf77b1) and FR-FORGR-003's dependency tree,
# re-run against the built forgr for forgr #11, over the workspace beside this
# script (the toy's store and app, as #10 committed them).
# - W1 and W6: app2's cargo test runs app/tests/uses_store.rs, which uses
#   store::forgr's factory, mock, Override and empty_store through the
#   dev-dependency on store with testing; app2 has no forgr dependency.
# - W2: cargo tree of store's normal build, then with testing for contrast.
# - W3: cargo clippy --tests over the workspace, the macro toy's rename (RESEARCH
#   01a1c917-ba6c-7553-bffe-c648a5f16f44 body.inputs: notes: 'owner TEXT NOT
#   NULL' -> 'author TEXT NOT NULL'; index a_notes_owner: notes(owner) ->
#   notes(author)) applied to store/schema/schema.json alone, clippy again, the
#   file restored. No other file is touched between the two clippy runs.
# - W7: w7_rerun.rs beside this script, as tests/forgr.rs of a crate assembled
#   under the target dir, and the same call at a test file's root.
# - FR-FORGR-003: forgr's rusqlite, with and without fake, and the rusqlite the
#   workspace's crates build with.
# Usage: sh w_rerun.sh <target dir>
set -u
here=$(cd "$(dirname "$0")" && pwd)
forgr=$(cd "$here/../../.." && pwd)
repo=$(cd "$forgr/../.." && pwd)
target=${1:?usage: sh w_rerun.sh <target dir>}
mkdir -p "$target"
target=$(cd "$target" && pwd)
schema="$here/store/schema/schema.json"
ws() {
    sub=$1
    shift
    cargo "$sub" --offline --manifest-path "$here/Cargo.toml" --target-dir "$target/ws" "$@"
}
ws_tree() {
    cargo tree --offline --manifest-path "$here/Cargo.toml" "$@"
}

echo "== W1 and W6: cargo test -p app2"
ws test -p app2 -- --nocapture
echo "== W1 and W6: app2's own dependencies (app/Cargo.toml)"
cat "$here/app/Cargo.toml"

echo "== W2: cargo tree -p store -e normal"
ws_tree -p store -e normal
echo "== W2 contrast: cargo tree -p store -e normal --features testing"
ws_tree -p store -e normal --features testing

echo "== W3: clippy --tests before the rename"
ws clippy --workspace --tests
cp "$schema" "$target/store-schema.json.before"
sed -e 's/notes (id TEXT PRIMARY KEY, owner TEXT NOT NULL/notes (id TEXT PRIMARY KEY, author TEXT NOT NULL/' \
    -e 's/ON notes(owner)/ON notes(author)/' "$target/store-schema.json.before" > "$schema"
echo "== W3: clippy --tests after the rename in store/schema/schema.json alone"
ws clippy --workspace --tests
cp "$target/store-schema.json.before" "$schema"

w7="$target/w7"
rm -rf "$w7"
mkdir -p "$w7/src" "$w7/tests" "$w7/schema"
cp "$schema" "$w7/schema/schema.json"
cp "$here/w7_rerun.rs" "$w7/tests/forgr.rs"
: > "$w7/src/lib.rs"
cat > "$w7/Cargo.toml" <<EOF
[package]
name = "w7"
version = "0.0.0"
edition = "2024"
publish = false
[workspace]
[dev-dependencies]
forgr = { path = "$forgr", features = ["fake"] }
EOF
w7() {
    sub=$1
    shift
    cargo "$sub" --offline --manifest-path "$w7/Cargo.toml" --target-dir "$target/w7-target" "$@"
}
echo "== W7: the call inside a module, beside use forgr::fake::Override"
w7 test --test forgr -- --nocapture
cat > "$w7/tests/root.rs" <<'EOF'
use forgr::fake::Override;
forgr::schema!("schema/schema.json");
#[test]
fn the_call_at_the_root() {
    let _: Override<String> = Override::Fake;
}
EOF
echo "== W7 contrast: the call at a test file's root, beside use forgr::fake::Override"
w7 test --test root --no-run

echo "== FR-FORGR-003: forgr's rusqlite, without fake"
cargo tree --offline --manifest-path "$repo/Cargo.toml" -p forgr -e normal -i rusqlite
echo "== FR-FORGR-003: forgr's rusqlite, with fake"
cargo tree --offline --manifest-path "$repo/Cargo.toml" -p forgr -e normal --features fake -i rusqlite
echo "== FR-FORGR-003: the rusqlite the workspace's crates build with"
ws_tree --workspace -e normal,dev -i rusqlite

rm -f "$here/Cargo.lock"
