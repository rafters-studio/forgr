//! A table's module: its column markers and its `Row`.

use std::fmt::Write as _;

use forgr_schema::{Column, Table};

use crate::ident::ident;

/// A column's SQLite type affinity, by SQLite's rules for its declared type:
/// INTEGER, TEXT, BLOB (or no type) and REAL; any other type has NUMERIC
/// affinity and takes any SQLite value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Affinity {
    Integer,
    Text,
    Blob,
    Real,
    Numeric,
}

/// The column's affinity.
pub(crate) fn affinity(column: &Column) -> Affinity {
    let declared = column.sql_type.to_ascii_uppercase();
    if declared.contains("INT") {
        Affinity::Integer
    } else if ["CHAR", "CLOB", "TEXT"]
        .iter()
        .any(|t| declared.contains(t))
    {
        Affinity::Text
    } else if declared.is_empty() || declared.contains("BLOB") {
        Affinity::Blob
    } else if ["REAL", "FLOA", "DOUB"]
        .iter()
        .any(|t| declared.contains(t))
    {
        Affinity::Real
    } else {
        Affinity::Numeric
    }
}

/// The Rust type of a column's values, by its affinity; a NUMERIC column
/// takes any SQLite value.
pub(crate) fn rust_type(column: &Column) -> &'static str {
    match affinity(column) {
        Affinity::Integer => "i64",
        Affinity::Text => "::std::string::String",
        Affinity::Blob => "::std::vec::Vec<u8>",
        Affinity::Real => "f64",
        Affinity::Numeric => "::forgr::value::Value",
    }
}

/// The Rust type of the column in `Row`: a primary-key or NOT NULL column its
/// value's type, any other column an `Option` of it.
pub(crate) fn row_type(table: &Table, column: &Column) -> String {
    let ty = rust_type(column);
    if column.not_null || in_key(table, column) {
        ty.to_owned()
    } else {
        format!("::core::option::Option<{ty}>")
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
        let _ = write!(
            out,
            "pub {}: {},",
            ident(&column.name),
            row_type(table, column)
        );
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
