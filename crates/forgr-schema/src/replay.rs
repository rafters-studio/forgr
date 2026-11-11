//! Replay of schema.json into in-memory SQLite: tables first, each virtual table
//! created from its own entry and the entries of its shadow tables skipped.

use rusqlite::Connection;

use crate::Error;
use crate::file::Object;

/// The replayed store and the names of its tables that are neither virtual
/// tables nor their shadow tables, in file order.
pub(crate) struct Replay {
    pub conn: Connection,
    pub tables: Vec<String>,
}

fn is_virtual(object: &Object) -> bool {
    object.kind == "table"
        && object
            .sql
            .trim_start()
            .get(..20)
            .is_some_and(|head| head.eq_ignore_ascii_case("CREATE VIRTUAL TABLE"))
}

/// Replays `objects` into a fresh in-memory SQLite: every table before every
/// other object, otherwise in file order. A table named `<virtual table>_*` is a
/// shadow table SQLite recreates with its virtual table, so its entry is skipped.
pub(crate) fn replay(objects: &[Object]) -> Result<Replay, Error> {
    let virtuals: Vec<&str> = objects
        .iter()
        .filter(|o| is_virtual(o))
        .map(|o| o.name.as_str())
        .collect();
    let is_shadow = |o: &Object| {
        o.kind == "table"
            && virtuals.iter().any(|v| {
                o.name
                    .strip_prefix(v)
                    .is_some_and(|rest| rest.starts_with('_'))
            })
    };

    let mut ordered: Vec<&Object> = objects.iter().filter(|o| !is_shadow(o)).collect();
    // A stable sort: views keep their name order, which SQLite resolves lazily.
    ordered.sort_by_key(|o| o.kind != "table");

    let conn = Connection::open_in_memory().map_err(Error::Open)?;
    for object in ordered {
        conn.execute_batch(&object.sql)
            .map_err(|source| Error::Replay {
                kind: object.kind.clone(),
                name: object.name.clone(),
                source,
            })?;
    }
    let tables = objects
        .iter()
        .filter(|o| o.kind == "table" && !is_virtual(o) && !is_shadow(o))
        .map(|o| o.name.clone())
        .collect();
    Ok(Replay { conn, tables })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::{parse, tests::WRITER};

    /// The writer toy's DDL, from which migratr wrote the fixture.
    const DDL: &str = include_str!("../tests/fixtures/writer/src/schema.sql");

    /// The writer toy's `shape`: every table's `table_info` and
    /// `foreign_key_list` rows, tables in name order.
    fn shape(conn: &Connection) -> Vec<String> {
        let mut names: Vec<String> = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
            )
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        names.sort();
        let mut out = Vec::new();
        for t in names {
            let mut s = conn
                .prepare(&format!(
                    "SELECT cid,name,type,\"notnull\",dflt_value,pk FROM pragma_table_info('{t}')"
                ))
                .unwrap();
            for row in s
                .query_map([], |r| {
                    Ok(format!(
                        "{t} col {:?} {:?} {:?} {:?} {:?} {:?}",
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, i64>(3)?,
                        r.get::<_, Option<String>>(4)?,
                        r.get::<_, i64>(5)?
                    ))
                })
                .unwrap()
            {
                out.push(row.unwrap());
            }
            let mut s = conn
                .prepare(&format!(
                    "SELECT id,seq,\"table\",\"from\",\"to\" FROM pragma_foreign_key_list('{t}')"
                ))
                .unwrap();
            for row in s
                .query_map([], |r| {
                    Ok(format!(
                        "{t} fk {:?} {:?} {:?} {:?} {:?}",
                        r.get::<_, i64>(0)?,
                        r.get::<_, i64>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, Option<String>>(4)?
                    ))
                })
                .unwrap()
            {
                out.push(row.unwrap());
            }
        }
        out
    }

    /// Writer toy P5: the replay's `table_info` and `foreign_key_list` match a
    /// store built from the DDL itself.
    #[test]
    fn p5_replay_matches_the_source() {
        let source = Connection::open_in_memory().unwrap();
        source.execute_batch(DDL).unwrap();
        let replay = replay(&parse(WRITER).unwrap().objects).unwrap();
        let (a, b) = (shape(&source), shape(&replay.conn));
        println!(
            "P5 shape rows source={} replay={} identical={}",
            a.len(),
            b.len(),
            a == b
        );
        assert_eq!(a.len(), 67);
        assert_eq!(a, b);
        let n: i64 = replay
            .conn
            .query_row("SELECT n FROM a_open_todo_count", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0);
    }

    /// Writer toy P2: in file order the fixture fails at its index; the replay reads it.
    #[test]
    fn p2_file_order_fails_and_the_replay_does_not() {
        let objects = parse(WRITER).unwrap().objects;
        let c = Connection::open_in_memory().unwrap();
        let failed = objects
            .iter()
            .find_map(|o| c.execute_batch(&o.sql).err().map(|e| (o.name.clone(), e)));
        let (name, err) = failed.expect("file order fails");
        println!("P2 file order: FAIL at {name}: {err}");
        assert_eq!(name, "a_notes_owner");
        assert!(err.to_string().starts_with("no such table: main.notes"));
        assert!(replay(&objects).is_ok());
        println!("P2 forgr-schema replay: ok");
    }

    /// Writer toy P3: tables first with the shadow tables' entries replayed fails;
    /// the replay creates `notes_fts` and SQLite recreates its shadow tables.
    #[test]
    fn p3_shadow_entries_fail_and_the_replay_does_not() {
        let objects = parse(WRITER).unwrap().objects;
        let c = Connection::open_in_memory().unwrap();
        let mut tables_first = objects.clone();
        tables_first.sort_by_key(|o| o.kind != "table");
        let failed = tables_first
            .iter()
            .find_map(|o| c.execute_batch(&o.sql).err().map(|e| (o.name.clone(), e)));
        let (name, err) = failed.expect("unskipped shadow tables fail");
        println!("P3 tables first: FAIL at {name}: {err}");
        assert_eq!(name, "notes_fts_config");
        assert!(
            err.to_string()
                .starts_with("table 'notes_fts_config' already exists")
        );

        let replay = replay(&objects).unwrap();
        let recreated: i64 = replay
            .conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name LIKE 'notes\\_fts%' ESCAPE '\\'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        println!("P3 forgr-schema replay: ok, notes_fts and its shadow tables: {recreated}");
        assert_eq!(recreated, 5);
        assert!(!replay.tables.iter().any(|t| t.starts_with("notes_fts")));
    }
}
