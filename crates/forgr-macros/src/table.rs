//! A table's module: its column markers and its `Row`.

use std::fmt::Write as _;

use forgr_schema::{Column, Table};

use crate::ident::ident;

/// The Rust type of a column's values, by SQLite's affinity rules for its
/// declared type: INTEGER, TEXT, BLOB (or no type) and REAL; any other type has
/// NUMERIC affinity and takes any SQLite value.
pub(crate) fn rust_type(column: &Column) -> &'static str {
    let declared = column.sql_type.to_ascii_uppercase();
    if declared.contains("INT") {
        "i64"
    } else if ["CHAR", "CLOB", "TEXT"]
        .iter()
        .any(|t| declared.contains(t))
    {
        "::std::string::String"
    } else if declared.is_empty() || declared.contains("BLOB") {
        "::std::vec::Vec<u8>"
    } else if ["REAL", "FLOA", "DOUB"]
        .iter()
        .any(|t| declared.contains(t))
    {
        "f64"
    } else {
        "::forgr::value::Value"
    }
}

/// Whether the column is part of the table's primary key.
pub(crate) fn in_key(table: &Table, column: &Column) -> bool {
    table
        .primary_key
        .as_ref()
        .is_some_and(|key| key.columns.contains(&column.name))
}

/// The `col` module, one marker type per column, and `Row`, the table's row as
/// written: a primary-key or NOT NULL column holds its value, any other column
/// an `Option` of it.
pub(crate) fn emit(out: &mut String, table: &Table) {
    out.push_str(
        "/// One marker type per column, named for it.\n\
         #[allow(non_camel_case_types)]\npub mod col {",
    );
    for column in &table.columns {
        let _ = write!(
            out,
            "/// The column `{}`.\npub struct {};",
            column.name,
            ident(&column.name)
        );
    }
    out.push_str(
        "}\n/// A row as written, its columns in declaration order.\n\
         #[derive(Debug, Clone, PartialEq)]\npub struct Row {",
    );
    for column in &table.columns {
        let ty = rust_type(column);
        let name = ident(&column.name);
        if column.not_null || in_key(table, column) {
            let _ = write!(out, "pub {name}: {ty},");
        } else {
            let _ = write!(out, "pub {name}: ::core::option::Option<{ty}>,");
        }
    }
    out.push_str(
        "}\nimpl ::forgr::insert::FromRow for Row {\n\
         fn from_row(row: &::forgr::insert::SqlRow<'_>) -> ::forgr::insert::Result<Self> {\n\
         ::core::result::Result::Ok(Self {",
    );
    for (at, column) in table.columns.iter().enumerate() {
        let _ = write!(out, "{}: row.get({at})?,", ident(&column.name));
    }
    out.push_str("})}}\nconst COLUMNS: &[&str] = &[");
    for column in &table.columns {
        let _ = write!(out, "{:?},", column.name);
    }
    out.push_str("];");
}
