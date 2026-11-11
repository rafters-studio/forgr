#!/bin/sh
# The kessel toy's K2 case (RESEARCH 01a1c932-caa1-7561-9bc0-34466568a74d),
# re-run against the built forgr for forgr #11: clippy green, a column renamed in
# schema.json alone, clippy again, with K2's timings and token size.
# The crate is assembled under the target dir as the toy's kessel-app was:
# kessel's schema.json beside this script (sha256 e72408fe...) at
# schema/schema.json, k4_every_table_rerun.rs (the toy's tests/kessel.rs, whole,
# as #9 adapted it) at tests/kessel.rs with all_tables.inc beside it, and
# k3_rerun.rs (the toy's tests/kessel_sql.rs, as #10 adapted it) at
# tests/kessel_sql.rs; forgr (features = ["fake"]) is a dev-dependency and
# rusqlite a dependency.
# The rename: K2's evidence names combat_styles.attack_type but not its new
# name, so it is renamed attack_kind, in the CREATE TABLE and in the index on it
# (idx_combat_styles_attack), in the assembled copy alone.
# Timings, by the shell's clock around each run:
# - warm: dependencies built, the crate's own output cleaned (cargo clean -p),
#   clippy --tests;
# - re-check: clippy --tests again with nothing changed;
# - after the rename: clippy --tests with only schema.json changed.
# Tokens: the expanded tests/kessel.rs, by nightly rustc -Zunpretty=expanded, in
# bytes, and in bytes with runs of whitespace taken as one space.
# clippy runs with clippy::doc_lazy_continuation allowed: the header comments of
# k3_rerun.rs and k4_every_table_rerun.rs trip it, and its warnings would bury
# the output; it does not change what errors.
# Usage: sh k2_rerun.sh <target dir>
set -u
here=$(cd "$(dirname "$0")" && pwd)
forgr=$(cd "$here/../../.." && pwd)
target=${1:?usage: sh k2_rerun.sh <target dir>}
mkdir -p "$target"
target=$(cd "$target" && pwd)
app="$target/k2"
rm -rf "$app"
mkdir -p "$app/src" "$app/tests" "$app/schema"
cp "$here/schema.json" "$app/schema/schema.json"
cp "$here/k4_every_table_rerun.rs" "$app/tests/kessel.rs"
cp "$here/all_tables.inc" "$app/tests/all_tables.inc"
cp "$here/k3_rerun.rs" "$app/tests/kessel_sql.rs"
: > "$app/src/lib.rs"
cat > "$app/Cargo.toml" <<EOT
[package]
name = "kessel-app"
version = "0.0.0"
edition = "2024"
publish = false
[workspace]
[dependencies]
rusqlite = { version = "0.41", features = ["bundled"] }
[dev-dependencies]
forgr = { path = "$forgr", features = ["fake"] }
EOT
clippy() {
    cargo clippy --offline --manifest-path "$app/Cargo.toml" --target-dir "$target/k2-target" --tests -- -A clippy::doc_lazy_continuation
}
timed() {
    start=$(perl -MTime::HiRes=time -e 'printf "%.3f", time')
    clippy
    end=$(perl -MTime::HiRes=time -e 'printf "%.3f", time')
    perl -e "printf \"K2 $1: %.1f s\n\", $end - $start"
}
echo "== K2 clippy --tests, building dependencies"
clippy
cargo clean --offline --manifest-path "$app/Cargo.toml" --target-dir "$target/k2-target" -p kessel-app
echo "== K2 clippy --tests, warm"
timed warm
echo "== K2 clippy --tests, re-check"
timed re-check
echo "== K2 tokens"
cargo +nightly rustc --offline --manifest-path "$app/Cargo.toml" --target-dir "$target/k2-nightly" \
    --profile=check --test kessel -- -Zunpretty=expanded > "$target/k2-expanded.rs" 2> "$target/k2-expanded.err"
echo "K2 expanded tests/kessel.rs: $(wc -c < "$target/k2-expanded.rs" | tr -d ' ') bytes; whitespace runs as one: $(tr -s ' \n\t' ' ' < "$target/k2-expanded.rs" | wc -c | tr -d ' ') bytes"
cp "$app/schema/schema.json" "$target/k2-schema.json.before"
sed -e 's/attack_type     TEXT NOT NULL/attack_kind     TEXT NOT NULL/' \
    -e 's/ON combat_styles(attack_type)/ON combat_styles(attack_kind)/' "$target/k2-schema.json.before" > "$app/schema/schema.json"
echo "== K2 the rename"
diff "$target/k2-schema.json.before" "$app/schema/schema.json" | grep -o '^[<>].*$' | grep -o '.\{0,24\}attack_[a-z]*.\{0,16\}' 
echo "== K2 files changed since the re-check"
find "$app" -newer "$target/k2-schema.json.before" -type f
echo "== K2 clippy --tests after the rename in schema.json alone"
timed "after the rename"
