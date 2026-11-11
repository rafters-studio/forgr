#!/bin/sh
# The workspace toy's W4 and W5 cases (RESEARCH 01a1c94b-8c34-7363-bafb-c60062cf77b1),
# re-run against the built forgr for forgr #10: store's cargo test runs the one
# check schema! emits, over the members Cargo.toml beside this script names.
# Run before and after the macro toy's rename (RESEARCH
# 01a1c917-ba6c-7553-bffe-c648a5f16f44 body.inputs: notes: 'owner TEXT NOT NULL'
# -> 'author TEXT NOT NULL'; index a_notes_owner: notes(owner) -> notes(author)),
# applied to store/schema/schema.json and undone after.
# Usage: sh w4_rerun.sh <target dir>
set -u
here=$(cd "$(dirname "$0")" && pwd)
target=${1:?usage: sh w4_rerun.sh <target dir>}
schema="$here/store/schema/schema.json"
run() {
    cargo test --offline --manifest-path "$here/Cargo.toml" -p store --target-dir "$target" -- --nocapture
}
echo "== W4 before the rename"
run
cp "$schema" "$target/store-schema.json.before"
sed -e 's/notes (id TEXT PRIMARY KEY, owner TEXT NOT NULL/notes (id TEXT PRIMARY KEY, author TEXT NOT NULL/' \
    -e 's/ON notes(owner)/ON notes(author)/' "$target/store-schema.json.before" > "$schema"
echo "== W4 after the rename"
run
cp "$target/store-schema.json.before" "$schema"
rm -f "$here/Cargo.lock"
