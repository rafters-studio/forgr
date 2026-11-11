// Adapted from the kessel toy's tool writer/examples/kessel_schema.rs (RESEARCH
// 01a1c932-caa1-7561-9bc0-34466568a74d body.inputs), its K6 case, for forgr #11.
// What changed and why:
// - the block at the end of main, after the toy's byte comparison, is added: it
//   reads both files, the hand-written one and migratr's, with the built
//   forgr-schema (forgr_schema::Schema::read) and compares what it reads, since
//   the criterion re-runs K6 "against the built forgr-schema: both files read";
//   it also compares the two files as JSON values, the toy's "identical JSON";
// - nothing above that block is changed. k6_rerun.sh beside this file builds it
//   as src/main.rs of a crate on migratr 0.3 (the release on rusqlite 0.41; the
//   toy's migratr version is not in body.inputs), syn as forgr's lockfile has it,
//   rusqlite 0.41, serde_json, and the built forgr-schema, and runs it on
//   kessel's src (huttspawn/kessel at 5d75f98).
//! The "something else" that writes kessel's schema.json: kessel's own DDL literals (non-test src)
//! run into an empty store, then written out by hand in migratr's format. K1 and K6.
use rusqlite::Connection;
use syn::visit::Visit;
use std::path::{Path, PathBuf};

struct V(Vec<String>);
impl<'a> Visit<'a> for V {
    fn visit_attribute(&mut self, _: &'a syn::Attribute) {}
    fn visit_item_mod(&mut self, m: &'a syn::ItemMod) {
        if m.attrs.iter().any(|a| a.path().is_ident("cfg")) { return; }
        syn::visit::visit_item_mod(self, m)
    }
    fn visit_item_fn(&mut self, f: &'a syn::ItemFn) {
        if f.attrs.iter().any(|a| a.path().is_ident("cfg") || a.path().is_ident("test")) { return; }
        syn::visit::visit_item_fn(self, f)
    }
    fn visit_lit_str(&mut self, l: &'a syn::LitStr) {
        let s = l.value();
        if s.to_ascii_uppercase().contains("CREATE ") { self.0.push(s) }
    }
    fn visit_macro(&mut self, m: &'a syn::Macro) {
        for t in m.tokens.clone() { if let Ok(l) = syn::parse2::<syn::LitStr>(t.into()) { self.visit_lit_str(&l) } }
    }
}
fn walk(d: &Path, out: &mut Vec<PathBuf>) { let mut v: Vec<_> = std::fs::read_dir(d).unwrap().flatten().map(|e| e.path()).collect(); v.sort(); for p in v { if p.is_dir() { walk(&p, out) } else if p.extension().is_some_and(|e| e == "rs") { out.push(p) } } }

fn main() {
    let src = Path::new(&std::env::args().nth(1).unwrap()).to_path_buf();
    let out = PathBuf::from(std::env::args().nth(2).unwrap());
    let mut files = Vec::new(); walk(&src, &mut files);
    let mut v = V(Vec::new());
    for f in files { if let Ok(file) = syn::parse_file(&std::fs::read_to_string(&f).unwrap()) { v.visit_file(&file) } }
    let conn = Connection::open_in_memory().unwrap();
    // several passes: a statement may name a table a later literal creates
    let mut pending = v.0.clone(); let mut failed = Vec::new();
    for _ in 0..5 { failed.clear(); for s in &pending { if let Err(e) = conn.execute_batch(s) { failed.push((s.clone(), e.to_string())) } } pending = failed.iter().map(|(s, _)| s.clone()).collect(); }
    println!("K1 DDL literals {}, failed after retries {}", v.0.len(), failed.len());
    for (s, e) in failed.iter().take(8) { println!("  fail: {e} :: {}", s.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(100).collect::<String>()); }
    // by hand, in migratr's format
    let mut objs: Vec<(String, String, String)> = conn.prepare("SELECT type, name, sql FROM sqlite_master WHERE sql IS NOT NULL AND name NOT LIKE 'sqlite_%'").unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap().map(|r| r.unwrap()).collect();
    objs.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    let tables = objs.iter().filter(|o| o.0 == "table").count();
    let json = serde_json::json!({"version": null, "objects": objs.iter().map(|(t, n, s)| serde_json::json!({"type": t, "name": n, "sql": s})).collect::<Vec<_>>()});
    std::fs::create_dir_all(&out).unwrap();
    let hand = serde_json::to_string_pretty(&json).unwrap() + "\n";
    std::fs::write(out.join("schema.json"), &hand).unwrap();
    println!("K1 tables {tables}, objects {}", objs.len());
    // K6: migratr's writer on the same store
    let dir = out.join("by-migratr"); std::fs::create_dir_all(&dir).unwrap();
    let mut ex = migratr::RusqliteExecutor::new(conn);
    migratr::write_schema(&mut ex, &dir).unwrap();
    let m = std::fs::read_to_string(dir.join("schema.json")).unwrap();
    println!("K6 hand-written bytes == migratr bytes: {}", m == hand);
    // forgr #11: both files read by the built forgr-schema.
    let by_hand = forgr_schema::Schema::read(&hand).unwrap();
    let by_migratr = forgr_schema::Schema::read(&m).unwrap();
    println!("K6 forgr-schema tables: hand-written {}, migratr {}", by_hand.tables.len(), by_migratr.tables.len());
    println!("K6 forgr-schema reads the two the same: {}", by_hand == by_migratr);
    let hand_json: serde_json::Value = serde_json::from_str(&hand).unwrap();
    let m_json: serde_json::Value = serde_json::from_str(&m).unwrap();
    println!("K6 objects: hand-written {}, migratr {}", hand_json["objects"].as_array().map_or(0, Vec::len), m_json["objects"].as_array().map_or(0, Vec::len));
    println!("K6 the same JSON: {}", hand_json == m_json);
}
