//! A table's `mock`, emitted only with forgr's fake feature; without it, a
//! stand-in whose error names the feature.

use std::fmt::Write as _;

use forgr_schema::{Column, ForeignKey, Table};

use crate::ident::ident;
use crate::table::{Affinity, affinity, in_key};

/// With fake, `mock(n)`, `Mock` and its `insert`, which builds a fresh parent
/// per row for each required foreign key; without, the stand-in `mock`.
/// Decided here at expansion by forgr-macros' own fake feature, which forgr's
/// forwards to: a cfg in the generated code would read the calling crate's
/// features instead (macro toy M6).
pub(crate) fn emit(out: &mut String, table: &Table, tables: &[Table]) {
    if !cfg!(feature = "fake") {
        out.push_str(
            "/// Needs forgr's `fake` feature.\n\
             pub fn mock<N: ::forgr::insert::MocksNeedFake>(n: N) -> N::Mock {\n\
             ::forgr::insert::MocksNeedFake::mock(n)\n\
             }",
        );
        return;
    }
    let _ = write!(
        out,
        "/// `n` rows of `{name}` filled by fake.\n\
         pub fn mock(n: usize) -> Mock {{\n\
         Mock {{ n }}\n\
         }}\n\
         /// Rows of `{name}` filled by fake, not yet written.\n\
         #[must_use]\n\
         pub struct Mock {{\n\
         n: usize,\n\
         }}\n\
         impl Mock {{\n\
         /// Writes the rows and returns them as written, keys included.\n\
         ///\n\
         /// # Errors\n\
         ///\n\
         /// When the store refuses a row.\n\
         pub fn insert(self, store: &::forgr::store::Connection) -> ::forgr::insert::Result<::std::vec::Vec<Row>> {{\n\
         ::forgr::fake::insert(store, {name:?}, COLUMNS, FILL, self.n, {parents})\n\
         }}\n\
         }}\n\
         const FILL: &[::forgr::fake::Column] = &[",
        name = table.name,
        parents = parents(table, tables),
    );
    for column in &table.columns {
        let _ = write!(
            out,
            "::forgr::fake::Column {{ name: {:?}, fill: {} }},",
            column.name,
            fill(table, column)
        );
    }
    out.push_str("];");
}

/// The per-row hook of a mock's insert: for each required foreign key to
/// another table, one fresh parent mocked and the key set from its referenced
/// columns. A foreign key to the table's own table gets no parent: the test
/// sets it with Fixed.
fn parents(table: &Table, tables: &[Table]) -> String {
    let mut body = String::new();
    for fk in &table.foreign_keys {
        if fk.table.eq_ignore_ascii_case(&table.name) {
            continue;
        }
        let Some(parent) = tables
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(&fk.table))
        else {
            continue;
        };
        let Some(pairs) = pairs(table, parent, fk) else {
            continue;
        };
        if !pairs
            .iter()
            .any(|(child, _)| child.not_null || in_key(table, child))
        {
            continue;
        }
        let _ = write!(
            body,
            "for parent in super::{}::mock(1).insert(store)? {{",
            parent.name
        );
        for (child, referenced) in pairs {
            let _ = write!(
                body,
                "set = ::forgr::insert::put(set, {:?}, ::forgr::value::Value::from(parent.{}));",
                child.name,
                ident(&referenced.name)
            );
        }
        body.push('}');
    }
    if body.is_empty() {
        "::core::result::Result::Ok".to_owned()
    } else {
        format!("|mut set| {{ {body} ::core::result::Result::Ok(set) }}")
    }
}

/// Each column of `fk` in `table` and the column of `parent` it references, a
/// key that names no column referencing the parent's primary key; names
/// matched without regard to case. `None` when one does not resolve.
fn pairs<'a>(
    table: &'a Table,
    parent: &'a Table,
    fk: &ForeignKey,
) -> Option<Vec<(&'a Column, &'a Column)>> {
    let find =
        |t: &'a Table, name: &str| t.columns.iter().find(|c| c.name.eq_ignore_ascii_case(name));
    fk.columns
        .iter()
        .zip(&fk.references)
        .enumerate()
        .map(|(at, (from, to))| {
            let to = match to {
                Some(to) => to.as_str(),
                None => parent.primary_key.as_ref()?.columns.get(at)?.as_str(),
            };
            Some((find(table, from)?, find(parent, to)?))
        })
        .collect()
}

/// What a mock writes to `column`: a nullable foreign key NULL, a TEXT key a
/// UUIDv7, an INTEGER key a positive integer, a column with a default its
/// default, any other a value by type, from its CHECK list when it has one.
fn fill(table: &Table, column: &Column) -> String {
    let affinity = affinity(column);
    let key = in_key(table, column);
    let nullable = !column.not_null && !key;
    let foreign = table
        .foreign_keys
        .iter()
        .any(|fk| fk.columns.contains(&column.name));
    if nullable && foreign {
        "::forgr::fake::Fill::Null".to_owned()
    } else if key && affinity == Affinity::Text {
        "::forgr::fake::Fill::Uuid7".to_owned()
    } else if key && affinity == Affinity::Integer {
        "::forgr::fake::Fill::PositiveInteger".to_owned()
    } else if column.default.is_some() && !key {
        "::forgr::fake::Fill::Default".to_owned()
    } else {
        let list = column.check_list.as_ref().map_or_else(
            || "::core::option::Option::None".to_owned(),
            |values| format!("::core::option::Option::Some(&{values:?})"),
        );
        let ty = match affinity {
            Affinity::Integer => "Integer",
            Affinity::Text => "Text",
            Affinity::Blob => "Blob",
            Affinity::Real => "Real",
            Affinity::Numeric => "Numeric",
        };
        format!(
            "::forgr::fake::Fill::Value {{ ty: ::forgr::fake::Type::{ty}, nullable: {nullable}, list: {list} }}"
        )
    }
}
