//! CHECK (col IN (...)) lists, read from the stored CREATE TABLE text, since
//! PRAGMA does not expose them.

/// The values of the first `CHECK (column IN ('a', 'b', ...))` in `sql`, or
/// `None` when it holds no such list of single-quoted values for `column`.
pub(crate) fn check_list(sql: &str, column: &str) -> Option<Vec<String>> {
    let upper = sql.to_ascii_uppercase();
    upper
        .match_indices("CHECK")
        .find_map(|(at, _)| list_after(&sql[at + 5..], column))
}

/// Reads `(column IN ('a', ...))` at the start of `rest`, after whitespace.
fn list_after(rest: &str, column: &str) -> Option<Vec<String>> {
    let rest = rest.trim_start().strip_prefix('(')?.trim_start();
    let rest = strip_ident(rest, column)?.trim_start();
    let rest = rest
        .get(..2)
        .filter(|kw| kw.eq_ignore_ascii_case("IN"))
        .map(|_| &rest[2..])?;
    let mut rest = rest.trim_start().strip_prefix('(')?;
    let mut values = Vec::new();
    loop {
        let body = rest.trim_start().strip_prefix('\'')?;
        let end = body.find('\'')?;
        values.push(body[..end].to_owned());
        rest = body[end + 1..].trim_start();
        if let Some(after) = rest.strip_prefix(',') {
            rest = after;
        } else {
            rest.strip_prefix(')')?.trim_start().strip_prefix(')')?;
            return Some(values);
        }
    }
}

/// Strips `column`, bare or double-quoted, from the start of `rest`.
fn strip_ident<'a>(rest: &'a str, column: &str) -> Option<&'a str> {
    let quoted = rest
        .strip_prefix('"')
        .and_then(|r| r.strip_prefix(column))
        .and_then(|r| r.strip_prefix('"'));
    quoted.or_else(|| {
        let after = rest.strip_prefix(column)?;
        let next = after.chars().next();
        (!next.is_some_and(|c| c.is_alphanumeric() || c == '_')).then_some(after)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Writer toy P6: notes.kind's list, from the notes CREATE TABLE text.
    #[test]
    fn p6_reads_notes_kind() {
        let schema = crate::Schema::read(crate::file::tests::WRITER).unwrap();
        let notes = schema.tables.iter().find(|t| t.name == "notes").unwrap();
        let kind = notes.columns.iter().find(|c| c.name == "kind").unwrap();
        println!("P6 notes.kind check list: {:?}", kind.check_list);
        assert_eq!(
            kind.check_list.as_deref(),
            Some(&["memo".to_owned(), "todo".to_owned(), "log".to_owned()][..])
        );
        let owner = notes.columns.iter().find(|c| c.name == "owner").unwrap();
        assert_eq!(owner.check_list, None);
    }

    #[test]
    fn another_columns_list_is_not_read() {
        let sql = "CREATE TABLE t (kind TEXT CHECK (kind IN ('a', 'b')), kinds TEXT)";
        assert_eq!(check_list(sql, "kinds"), None);
        assert_eq!(check_list(sql, "kind"), Some(vec!["a".into(), "b".into()]));
    }
}
