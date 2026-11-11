//! A table's `mock`, emitted only with forgr's fake feature; without it, a
//! stand-in whose error names the feature.

use std::fmt::Write as _;

use forgr_schema::{Column, Table};

use crate::table::{in_key, rust_type};

/// With fake, `mock(n)`, `Mock` and its `insert`; without, the stand-in `mock`.
/// Decided here at expansion by forgr-macros' own fake feature, which forgr's
/// forwards to: a cfg in the generated code would read the calling crate's
/// features instead (macro toy M6).
pub(crate) fn emit(out: &mut String, table: &Table) {
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
         ::forgr::fake::insert(store, {name:?}, COLUMNS, FILL, self.n)\n\
         }}\n\
         }}\n\
         const FILL: &[::forgr::fake::Column] = &[",
        name = table.name,
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

/// What a mock writes to `column`: a nullable foreign key NULL, a TEXT key a
/// UUIDv7, an INTEGER key a positive integer, a column with a default its
/// default, any other a value by type, from its CHECK list when it has one.
fn fill(table: &Table, column: &Column) -> String {
    let ty = match rust_type(column) {
        "i64" => "Integer",
        "::std::string::String" => "Text",
        "::std::vec::Vec<u8>" => "Blob",
        "f64" => "Real",
        _ => "Numeric",
    };
    let key = in_key(table, column);
    let nullable = !column.not_null && !key;
    let foreign = table
        .foreign_keys
        .iter()
        .any(|fk| fk.columns.contains(&column.name));
    if nullable && foreign {
        "::forgr::fake::Fill::Null".to_owned()
    } else if key && ty == "Text" {
        "::forgr::fake::Fill::Uuid7".to_owned()
    } else if key && ty == "Integer" {
        "::forgr::fake::Fill::PositiveInteger".to_owned()
    } else if column.default.is_some() && !key {
        "::forgr::fake::Fill::Default".to_owned()
    } else {
        let list = column.check_list.as_ref().map_or_else(
            || "::core::option::Option::None".to_owned(),
            |values| format!("::core::option::Option::Some(&{values:?})"),
        );
        format!(
            "::forgr::fake::Fill::Value {{ ty: ::forgr::fake::Type::{ty}, nullable: {nullable}, list: {list} }}"
        )
    }
}
