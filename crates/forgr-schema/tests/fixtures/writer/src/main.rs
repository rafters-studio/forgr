use std::path::Path;
use rusqlite::Connection;

fn shape(conn: &Connection) -> Vec<String> {
    let mut out = Vec::new();
    let mut names: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").unwrap()
        .query_map([], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect();
    names.sort();
    for t in names {
        let mut s = conn.prepare(&format!("SELECT cid,name,type,\"notnull\",dflt_value,pk FROM pragma_table_info('{t}')")).unwrap();
        for row in s.query_map([], |r| Ok(format!("{t} col {:?} {:?} {:?} {:?} {:?} {:?}",
            r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, i64>(3)?, r.get::<_, Option<String>>(4)?, r.get::<_, i64>(5)?))).unwrap() {
            out.push(row.unwrap());
        }
        let mut s = conn.prepare(&format!("SELECT id,seq,\"table\",\"from\",\"to\" FROM pragma_foreign_key_list('{t}')")).unwrap();
        for row in s.query_map([], |r| Ok(format!("{t} fk {:?} {:?} {:?} {:?} {:?}",
            r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?, r.get::<_, Option<String>>(4)?))).unwrap() {
            out.push(row.unwrap());
        }
    }
    out
}

fn replay(objects: &[(String, String, String)], label: &str) -> Option<Connection> {
    let c = Connection::open_in_memory().unwrap();
    for (kind, name, sql) in objects {
        if let Err(e) = c.execute_batch(sql) {
            println!("[{label}] FAIL at {kind} {name}: {e}");
            return None;
        }
    }
    println!("[{label}] ok");
    Some(c)
}

fn main() {
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("out");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).unwrap();
    let db = out.join("app.db");
    let src = Connection::open(&db).unwrap();
    src.execute_batch(include_str!("schema.sql")).unwrap();
    drop(src);

    // P1: the published writer, as a project without migratr would call it.
    let mut ex = migratr::RusqliteExecutor::new(Connection::open(&db).unwrap());
    migratr::write_schema(&mut ex, &out).expect("write_schema");
    let text = std::fs::read_to_string(out.join("schema.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    println!("P1 version = {}", v["version"]);
    let objects: Vec<(String, String, String)> = v["objects"].as_array().unwrap().iter()
        .map(|o| (o["type"].as_str().unwrap().into(), o["name"].as_str().unwrap().into(), o["sql"].as_str().unwrap().into()))
        .collect();
    for (k, n, _) in &objects { println!("  object {k} {n}"); }

    // P2: file order.
    replay(&objects, "P2 file order");

    // P3: tables first.
    let mut tf = objects.clone();
    tf.sort_by_key(|(k, _, _)| if k == "table" { 0 } else { 1 });
    replay(&tf, "P3 tables first");

    // P5: tables first, shadow tables of virtual tables skipped.
    let virtuals: Vec<String> = objects.iter()
        .filter(|(k, _, s)| k == "table" && s.to_ascii_uppercase().starts_with("CREATE VIRTUAL TABLE"))
        .map(|(_, n, _)| n.clone()).collect();
    let kept: Vec<_> = tf.iter().filter(|(k, n, _)| !(k == "table" && virtuals.iter().any(|v| n.starts_with(&format!("{v}_")) ))).cloned().collect();
    // P4 rides on this order too: views stay sorted by name, a_open_todo_count before open_todos.
    if let Some(c) = replay(&kept, "P5 tables first, shadows skipped") {
        let a = shape(&Connection::open(&db).unwrap());
        let b = shape(&c);
        println!("P5 shape rows source={} replay={} identical={}", a.len(), b.len(), a == b);
        let n: i64 = c.query_row("SELECT n FROM a_open_todo_count", [], |r| r.get(0)).unwrap();
        println!("P4 view-on-view after replay: count = {n}");
    }

    // P6: CHECK list recoverable from text, not from PRAGMA.
    let notes_sql = &objects.iter().find(|(_, n, _)| n == "notes").unwrap().2;
    println!("P6 notes sql contains CHECK = {}", notes_sql.contains("CHECK (kind IN ('memo', 'todo', 'log'))"));
}
