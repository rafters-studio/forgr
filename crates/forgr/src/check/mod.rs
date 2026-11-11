//! The SQL check `schema!` emits as a test of the owning crate: the SQL in the
//! non-test src of every workspace member, prepared against schema.json.

use std::fmt;
use std::path::Path;

mod prepare;
mod scan;
mod workspace;

/// SQL text found in a member's src, where it was found, and how it is held.
pub(crate) struct Found {
    /// `<member path>/src/<file>:<line>` for a literal, `<file> #<n>` for a
    /// statement of a .sql file, relative to the workspace root.
    pub at: String,
    pub sql: String,
    pub held: Held,
}

/// How SQL text is held in src.
pub(crate) enum Held {
    /// A string literal.
    Literal,
    /// The format string of a `format!`.
    Format,
    /// One statement of a .sql file.
    Statement,
    /// A .rs file syn could not read; `sql` holds why.
    Unread,
}

/// What the check found: checked, failed and unchecked, counted apart.
#[derive(Default)]
pub(crate) struct Report {
    /// Statements prepared, failed ones included.
    checked: usize,
    /// Where, SQLite's error, and the statement.
    failed: Vec<(String, String, String)>,
    /// Where, why it was not prepared, and the text.
    unchecked: Vec<(String, String, String)>,
}

impl Report {
    fn passed(&mut self) {
        self.checked += 1;
    }

    fn failed(&mut self, at: String, error: String, sql: &str) {
        self.checked += 1;
        self.failed.push((at, error, short(sql)));
    }

    fn unchecked(&mut self, at: String, why: String, sql: &str) {
        self.unchecked.push((at, why, short(sql)));
    }
}

/// `sql` past its leading comments, on one line, cut at 60 characters.
fn short(sql: &str) -> String {
    let line = scan::skip_comments(sql)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    match line.char_indices().nth(60) {
        Some((cut, _)) => format!("{} ...", &line[..cut]),
        None => line,
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (at, why, sql) in &self.unchecked {
            if sql.is_empty() {
                writeln!(f, "unchecked {at}: {why}")?;
            } else {
                writeln!(f, "unchecked {at}: {why}: {sql}")?;
            }
        }
        for (at, error, sql) in &self.failed {
            writeln!(f, "{at}: {error}: {sql}")?;
        }
        write!(
            f,
            "{} of {} checked SQL statements do not match schema.json ({} unchecked, listed above)",
            self.failed.len(),
            self.checked,
            self.unchecked.len()
        )
    }
}

/// The SQL check `schema!` emits: reads the workspace that holds
/// `manifest_dir` from its manifest, prepares the SQL in the non-test src of
/// every member against `schema_json`, and prints the report.
///
/// # Panics
///
/// When a statement names what the schema lacks, with the report; or when the
/// check cannot run, saying why.
pub fn run(manifest_dir: &str, schema_json: &str) {
    match check(Path::new(manifest_dir), schema_json) {
        Ok(report) if report.failed.is_empty() => println!("forgr SQL check:\n{report}"),
        Ok(report) => panic!("forgr SQL check:\n{report}"),
        Err(err) => panic!("forgr's SQL check could not run: {err}"),
    }
}

fn check(manifest_dir: &Path, schema_json: &str) -> Result<Report, String> {
    let conn = forgr_schema::replay(schema_json).map_err(|err| err.to_string())?;
    let workspace = workspace::read(manifest_dir)?;
    let mut report = Report::default();
    for member in &workspace.members {
        for found in scan::member(&workspace.root, member)? {
            prepare::check(&conn, found, &mut report);
        }
    }
    Ok(report)
}
