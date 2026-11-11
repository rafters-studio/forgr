#!/bin/sh
# The macro toy's M2 and M3 sequence (RESEARCH 01a1c917-ba6c-7553-bffe-c648a5f16f44),
# re-run against the built forgr for forgr #11: clippy green, edit schema.json
# only, clippy again. The crate is assembled under the target dir: m2_rerun.rs
# beside this script as tests/forgr.rs, and a copy of schema.json beside this
# script at schema/schema.json. The rename, from the toy's body.inputs: notes:
# 'owner TEXT NOT NULL' -> 'author TEXT NOT NULL'; index a_notes_owner:
# notes(owner) -> notes(author). It is made to that copy alone; no other file is
# touched between the two clippy runs (M3). The committed schema.json is left as
# it is. The dep-info rustc wrote for the test is printed, to show schema.json
# among the files it tracks.
# Usage: sh m2_rerun.sh <target dir>
set -u
here=$(cd "$(dirname "$0")" && pwd)
forgr=$(cd "$here/../../.." && pwd)
target=${1:?usage: sh m2_rerun.sh <target dir>}
mkdir -p "$target"
target=$(cd "$target" && pwd)
app="$target/m2"
rm -rf "$app"
mkdir -p "$app/src" "$app/tests" "$app/schema"
cp "$here/schema.json" "$app/schema/schema.json"
cp "$here/m2_rerun.rs" "$app/tests/forgr.rs"
: > "$app/src/lib.rs"
cat > "$app/Cargo.toml" <<EOT
[package]
name = "m2"
version = "0.0.0"
edition = "2024"
publish = false
[workspace]
[dev-dependencies]
forgr = { path = "$forgr", features = ["fake"] }
EOT
clippy() {
    cargo clippy --offline --manifest-path "$app/Cargo.toml" --target-dir "$target/m2-target" --tests
}
echo "== M2 clippy --tests before the rename"
clippy
echo "== M3 the files rustc tracks for tests/forgr.rs"
tr ' ' '\n' < "$(ls "$target"/m2-target/debug/deps/forgr-*.d | head -n 1)" | grep -E 'schema\.json|tests/forgr\.rs' | sed 's/:$//' | sort -u
cp "$app/schema/schema.json" "$target/m2-schema.json.before"
sed -e 's/notes (id TEXT PRIMARY KEY, owner TEXT NOT NULL/notes (id TEXT PRIMARY KEY, author TEXT NOT NULL/' \
    -e 's/ON notes(owner)/ON notes(author)/' "$target/m2-schema.json.before" > "$app/schema/schema.json"
echo "== M3 files changed since the first clippy"
find "$app" -newer "$target/m2-schema.json.before" -type f
echo "== M2 clippy --tests after the rename in schema.json alone"
clippy
