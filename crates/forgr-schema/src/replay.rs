//! Replay of schema.json into in-memory SQLite: tables first, each virtual table
//! created from its own entry and the entries of the shadow tables SQLite made
//! for it skipped.

use rusqlite::Connection;

use crate::Error;
use crate::file::Object;

/// The replayed store and the entries of its tables that are neither virtual
/// tables nor their shadow tables, in file order.
pub(crate) struct Replay<'a> {
    pub conn: Connection,
    pub tables: Vec<&'a Object>,
}

fn is_virtual(object: &Object) -> bool {
    object.kind == "table"
        && object
            .sql
            .trim_start()
            .get(..20)
            .is_some_and(|head| head.eq_ignore_ascii_case("CREATE VIRTUAL TABLE"))
}

/// The tables SQLite created as shadow tables of the virtual tables replayed so
/// far: `PRAGMA table_list` reports them as type `shadow`. Which tables a virtual
/// table owns differs by module, so SQLite is asked rather than the names guessed.
fn shadow_tables(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM pragma_table_list WHERE type = 'shadow'")?;
    stmt.query_map([], |r| r.get(0))?.collect()
}

/// Replays `objects` into a fresh in-memory SQLite: every virtual table first,
/// then every other table, then every other object, each group in file order. A
/// table SQLite created as a virtual table's shadow table is recreated with it,
/// so its own entry is skipped; any other table is replayed, whatever its name.
pub(crate) fn replay(objects: &[Object]) -> Result<Replay<'_>, Error> {
    let conn = Connection::open_in_memory().map_err(Error::Open)?;
    let run = |object: &Object| {
        conn.execute_batch(&object.sql)
            .map_err(|source| Error::Replay {
                kind: object.kind.clone(),
                name: object.name.clone(),
                source,
            })
    };

    for object in objects.iter().filter(|o| is_virtual(o)) {
        run(object)?;
    }
    let shadows = shadow_tables(&conn).map_err(|source| Error::Read {
        table: "pragma_table_list".to_owned(),
        source,
    })?;
    let is_shadow = |o: &Object| o.kind == "table" && shadows.contains(&o.name);

    let mut ordered: Vec<&Object> = objects
        .iter()
        .filter(|o| !is_virtual(o) && !is_shadow(o))
        .collect();
    // A stable sort: views keep their name order, which SQLite resolves lazily.
    ordered.sort_by_key(|o| o.kind != "table");
    for object in ordered {
        run(object)?;
    }

    let tables = objects
        .iter()
        .filter(|o| o.kind == "table" && !is_virtual(o) && !is_shadow(o))
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
        let objects = parse(WRITER).unwrap().objects;
        let replay = replay(&objects).unwrap();
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
        assert!(
            !replay
                .tables
                .iter()
                .any(|t| t.name.starts_with("notes_fts"))
        );
    }

    /// The objects of a store built from `ddl`, as schema.json lists them: every
    /// `sqlite_master` row with SQL, shadow tables included, in name order.
    fn objects_of(ddl: &str) -> Vec<Object> {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(ddl).unwrap();
        c.prepare("SELECT type, name, sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY name")
            .unwrap()
            .query_map([], |r| {
                Ok(Object {
                    kind: r.get(0)?,
                    name: r.get(1)?,
                    sql: r.get(2)?,
                })
            })
            .unwrap()
            .map(Result::unwrap)
            .collect()
    }

    /// rannoch #67's shape: an FTS5 table `_search`, an ordinary `_search_map`
    /// beside it, and an R*Tree `r`.
    const SEARCH: &str = "
        CREATE VIRTUAL TABLE _search USING fts5(body);
        CREATE TABLE _search_map (rid INTEGER PRIMARY KEY, node TEXT NOT NULL) WITHOUT ROWID;
        CREATE VIRTUAL TABLE r USING rtree(id, x0, x1);
    ";

    /// forgr#31: a table named `<virtual table>_<suffix>` that SQLite did not
    /// create as a shadow table is replayed, and SQL naming it prepares.
    #[test]
    fn an_ordinary_table_with_a_virtual_tables_prefix_is_replayed() {
        let objects = objects_of(SEARCH);
        assert!(objects.iter().any(|o| o.name == "_search_config"));
        assert!(objects.iter().any(|o| o.name == "r_node"));
        let replay = replay(&objects).unwrap();
        replay
            .conn
            .prepare("SELECT DISTINCT m.node FROM _search s JOIN _search_map m ON m.rid = s.rowid WHERE _search MATCH ?1")
            .unwrap();
        let names: Vec<&str> = replay.tables.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, ["_search_map"]);
    }

    /// forgr#31: an R*Tree's `_node`, `_parent` and `_rowid` tables and the
    /// FTS5 shadow tables are still skipped and recreated by SQLite.
    #[test]
    fn rtree_and_fts5_shadow_tables_are_still_skipped() {
        let objects = objects_of(SEARCH);
        let replay = replay(&objects).unwrap();
        let mut shadows = shadow_tables(&replay.conn).unwrap();
        shadows.sort();
        assert_eq!(
            shadows,
            [
                "_search_config",
                "_search_content",
                "_search_data",
                "_search_docsize",
                "_search_idx",
                "r_node",
                "r_parent",
                "r_rowid",
            ]
        );
        assert!(replay.tables.iter().all(|t| !shadows.contains(&t.name)));
    }
}
