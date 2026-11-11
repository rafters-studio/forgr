//! The table, column, key and foreign-key model, read by PRAGMA from the
//! replayed tables.

use rusqlite::Connection;

use crate::check_list::check_list;

/// One table of the schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub name: String,
    /// In declaration order.
    pub columns: Vec<Column>,
    /// `None` when the table declares no primary key.
    pub primary_key: Option<PrimaryKey>,
    pub foreign_keys: Vec<ForeignKey>,
}

/// One column, as SQLite reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    /// The declared type; empty when none is declared.
    pub sql_type: String,
    pub not_null: bool,
    /// The default's SQL text.
    pub default: Option<String>,
    /// The values of the column's `CHECK (col IN (...))`, from the CREATE TABLE text.
    pub check_list: Option<Vec<String>>,
}

/// A primary key: one column or several, in key order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimaryKey {
    pub columns: Vec<String>,
}

/// A foreign key: one column or several, and what they reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignKey {
    pub columns: Vec<String>,
    /// The referenced table.
    pub table: String,
    /// The referenced columns, in order; `None` where the key names none and so
    /// references the table's primary key.
    pub references: Vec<Option<String>>,
}

/// Reads table `name`, created by `sql`, from the replayed store.
pub(crate) fn read_table(conn: &Connection, name: &str, sql: &str) -> rusqlite::Result<Table> {
    let mut keyed: Vec<(i64, String)> = Vec::new();
    let columns = conn
        .prepare(
            "SELECT name, type, \"notnull\", dflt_value, pk FROM pragma_table_info(?1) ORDER BY cid",
        )?
        .query_map([name], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, bool>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, i64>(4)?,
            ))
        })?
        .map(|row| {
            let (column, sql_type, not_null, default, pk) = row?;
            if pk > 0 {
                keyed.push((pk, column.clone()));
            }
            Ok(Column {
                check_list: check_list(sql, &column),
                name: column,
                sql_type,
                not_null,
                default,
            })
        })
        .collect::<rusqlite::Result<Vec<_>>>()?;
    keyed.sort();
    let primary_key = (!keyed.is_empty()).then(|| PrimaryKey {
        columns: keyed.into_iter().map(|(_, c)| c).collect(),
    });

    let mut foreign_keys: Vec<(i64, ForeignKey)> = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT id, \"table\", \"from\", \"to\" FROM pragma_foreign_key_list(?1) ORDER BY id, seq",
    )?;
    let rows = stmt.query_map([name], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
        ))
    })?;
    for row in rows {
        let (id, table, from, to) = row?;
        match foreign_keys.last_mut() {
            Some((last, key)) if *last == id => {
                key.columns.push(from);
                key.references.push(to);
            }
            _ => foreign_keys.push((
                id,
                ForeignKey {
                    columns: vec![from],
                    table,
                    references: vec![to],
                },
            )),
        }
    }

    Ok(Table {
        name: name.to_owned(),
        columns,
        primary_key,
        foreign_keys: foreign_keys.into_iter().map(|(_, k)| k).collect(),
    })
}
