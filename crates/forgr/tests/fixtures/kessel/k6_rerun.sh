#!/bin/sh
# The kessel toy's K6 case (RESEARCH 01a1c932-caa1-7561-9bc0-34466568a74d),
# re-run against the built forgr-schema for forgr #11. k6_rerun.rs beside this
# script (adapted from the toy's writer/examples/kessel_schema.rs; its header
# names the change) is built as src/main.rs of a crate assembled under the
# target dir and run on kessel's src: it writes kessel's schema.json by hand
# and by migratr's write_schema from the same store, compares the bytes as K6
# did, then reads both with the built forgr-schema. The hand-written file is
# then compared with the committed schema.json beside this script.
# Usage: KESSEL_DIR=<huttspawn/kessel checkout at 5d75f98> sh k6_rerun.sh <target dir>
set -u
here=$(cd "$(dirname "$0")" && pwd)
schema_crate=$(cd "$here/../../../../forgr-schema" && pwd)
kessel=${KESSEL_DIR:?set KESSEL_DIR to a huttspawn/kessel checkout at 5d75f98}
target=${1:?usage: KESSEL_DIR=<kessel> sh k6_rerun.sh <target dir>}
mkdir -p "$target"
target=$(cd "$target" && pwd)
tool="$target/k6"
rm -rf "$tool"
mkdir -p "$tool/src"
cp "$here/k6_rerun.rs" "$tool/src/main.rs"
cat > "$tool/Cargo.toml" <<EOT
[package]
name = "kessel-schema"
version = "0.0.0"
edition = "2024"
publish = false
[workspace]
[dependencies]
forgr-schema = { path = "$schema_crate" }
migratr = "0.3"
rusqlite = { version = "0.41", features = ["bundled"] }
serde_json = "1"
syn = { version = "3", features = ["full", "parsing", "visit"] }
EOT
echo "== K6 kessel at $(git -C "$kessel" rev-parse --short HEAD), $(git -C "$kessel" status --porcelain | wc -l | tr -d ' ') changed files"
out="$target/k6-out"
rm -rf "$out"
cargo run --offline --quiet --manifest-path "$tool/Cargo.toml" --target-dir "$target/k6-target" -- "$kessel/kessel/src" "$out"
echo "== K6 the two files"
shasum -a 256 "$out/schema.json" "$out/by-migratr/schema.json" "$here/schema.json"
cmp -s "$out/schema.json" "$here/schema.json" && echo "the hand-written file is the committed schema.json, byte for byte"
