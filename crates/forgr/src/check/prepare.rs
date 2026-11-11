//! Each statement found, prepared against schema.json; DDL, `format!` SQL and
//! text that does not parse are listed as unchecked, never prepared or failed.

use rusqlite::Connection;

use super::scan::{is_ddl, split};
use super::{Found, Held, Report};

/// Prepares what `found` holds against `conn`, the replayed schema.json, and
/// counts it in `report`. A literal holding more than one statement is checked
/// a statement at a time, numbered after its line; one whose first statement
/// is DDL is listed once.
pub(super) fn check(conn: &Connection, found: Found, report: &mut Report) {
    match found.held {
        Held::Format => report.unchecked(
            found.at,
            "format! builds this SQL at runtime".to_owned(),
            &found.sql,
        ),
        Held::Unread => report.unchecked(found.at, found.sql, ""),
        Held::Statement => statement(conn, found.at, &found.sql, report),
        Held::Literal => {
            let statements = split(&found.sql);
            match statements.as_slice() {
                [] => {}
                [one] => statement(conn, found.at, one, report),
                [first, ..] if is_ddl(first) => statement(conn, found.at, &found.sql, report),
                many => {
                    for (n, sql) in many.iter().enumerate() {
                        statement(conn, format!("{} #{}", found.at, n + 1), sql, report);
                    }
                }
            }
        }
    }
}

fn statement(conn: &Connection, at: String, sql: &str, report: &mut Report) {
    if is_ddl(sql) {
        report.unchecked(at, "DDL is not checked".to_owned(), sql);
        return;
    }
    let Err(err) = conn.prepare(sql) else {
        report.passed();
        return;
    };
    let message = match err {
        rusqlite::Error::SqliteFailure(_, Some(message))
        | rusqlite::Error::SqlInputError { msg: message, .. } => message,
        other => other.to_string(),
    };
    if does_not_parse(&message) {
        report.unchecked(at, format!("does not parse ({message})"), sql);
    } else {
        report.failed(at, message, sql);
    }
}

/// Whether SQLite's error says the text is not SQL, rather than that it names
/// what the schema lacks.
fn does_not_parse(message: &str) -> bool {
    message.contains("syntax error")
        || message.contains("incomplete input")
        || message.contains("unrecognized token")
}
