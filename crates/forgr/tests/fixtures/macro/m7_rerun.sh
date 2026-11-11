#!/bin/sh
# The macro toy's M7 sequence (RESEARCH 01a1c917-ba6c-7553-bffe-c648a5f16f44),
# re-run against the built forgr for forgr #10; C2 and C3 are read from the same
# two runs. The app is app/ beside this script: src/lib.rs and src/sql/report.sql
# from the toy's body.inputs, tests/check.rs holding only schema!().
# The rename, from the toy's body.inputs: notes: 'owner TEXT NOT NULL' ->
# 'author TEXT NOT NULL'; index a_notes_owner: notes(owner) -> notes(author).
# It is applied to schema.json beside this script and undone after, so the
# committed fixture is left as it was.
# Usage: sh m7_rerun.sh <target dir>
set -u
here=$(cd "$(dirname "$0")" && pwd)
target=${1:?usage: sh m7_rerun.sh <target dir>}
run() {
    cargo test --offline --manifest-path "$here/app/Cargo.toml" --target-dir "$target" -- --nocapture
}
echo "== M7 before the rename"
run
cp "$here/schema.json" "$target/schema.json.before"
sed -e 's/notes (id TEXT PRIMARY KEY, owner TEXT NOT NULL/notes (id TEXT PRIMARY KEY, author TEXT NOT NULL/' \
    -e 's/ON notes(owner)/ON notes(author)/' "$target/schema.json.before" > "$here/schema.json"
echo "== M7 after the rename"
run
cp "$target/schema.json.before" "$here/schema.json"
rm -f "$here/app/Cargo.lock"
