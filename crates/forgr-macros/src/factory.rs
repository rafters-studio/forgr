//! The type-state factory: required columns as where-clause bounds on insert.

use std::fmt::Write as _;

use forgr_schema::Table;

use crate::ident::ident;
use crate::table::{in_key, rust_type};

/// `Factory`, `factory()`, a chained setter per column and `insert`.
///
/// A required column (NOT NULL with no default, or part of the primary key)
/// has a type parameter of its own: the column's marker type while it is
/// unset, `::forgr::insert::Set` once its setter has run. `insert` bounds each
/// by the column's trait in `required`, which only `Set` implements, in its own
/// where clause, so an unset one is E0277 with the trait's message naming the
/// column; on the impl block the bounds would give E0599 and lose the message
/// (macro toy M4).
pub(crate) fn emit(out: &mut String, table: &Table) {
    let required: Vec<usize> = table
        .columns
        .iter()
        .enumerate()
        .filter(|(_, c)| (c.not_null && c.default.is_none()) || in_key(table, c))
        .map(|(at, _)| at)
        .collect();
    let params: Vec<String> = (0..required.len()).map(|n| format!("R{n}")).collect();
    let params = params.join(", ");
    let state = if required.is_empty() {
        "()".to_owned()
    } else {
        format!("({params},)")
    };
    let unset = required
        .iter()
        .map(|&at| format!("col::{}", ident(&table.columns[at].name)))
        .collect::<Vec<_>>()
        .join(", ");

    out.push_str(
        "/// One trait per required column, holding only once it is set.\n\
         #[doc(hidden)]\n#[allow(non_camel_case_types)]\npub mod required {",
    );
    for &at in &required {
        let name = ident(&table.columns[at].name);
        let _ = write!(
            out,
            "#[diagnostic::on_unimplemented(\n\
             message = \"a required column was never set: {table}::col::{name}\",\n\
             label = \"this factory never set {table}::col::{name}\",\n\
             note = \"the schema says this column is NOT NULL with no default, \
             or it is part of the primary key\"\n\
             )]\n\
             pub trait {name} {{}}\n\
             impl {name} for ::forgr::insert::Set {{}}",
            table = table.name,
        );
    }
    out.push('}');
    let _ = write!(
        out,
        "/// A row of `{table}` the test fills by chained setters, one per column; \
         every column it leaves unset is the schema's.\n\
         #[must_use]\n\
         pub struct Factory<{params}> {{\n\
         set: ::std::vec::Vec<(&'static str, ::forgr::value::Value)>,\n\
         state: ::core::marker::PhantomData<{state}>,\n\
         }}\n\
         /// An empty factory: no column set.\n\
         pub fn factory() -> Factory<{unset}> {{\n\
         Factory {{ set: ::std::vec::Vec::new(), state: ::core::marker::PhantomData }}\n\
         }}\n\
         impl<{params}> Factory<{params}> {{",
        table = table.name,
    );
    for (at, column) in table.columns.iter().enumerate() {
        let name = ident(&column.name);
        let ty = rust_type(column);
        let returns = match required.iter().position(|&r| r == at) {
            Some(slot) => {
                let state: Vec<String> = (0..required.len())
                    .map(|n| {
                        if n == slot {
                            "::forgr::insert::Set".to_owned()
                        } else {
                            format!("R{n}")
                        }
                    })
                    .collect();
                format!("Factory<{}>", state.join(", "))
            }
            None => "Self".to_owned(),
        };
        let _ = write!(
            out,
            "/// Sets `{column}`.\n\
             pub fn {name}(self, value: impl ::forgr::value::IntoCell<{ty}>) -> {returns} {{\n\
             Factory {{\n\
             set: ::forgr::insert::put(self.set, {column:?}, ::forgr::value::IntoCell::<{ty}>::into_cell(value)),\n\
             state: ::core::marker::PhantomData,\n\
             }}\n\
             }}",
            column = column.name,
        );
    }
    let bounds: String = required
        .iter()
        .enumerate()
        .map(|(n, &at)| format!("R{n}: required::{},", ident(&table.columns[at].name)))
        .collect();
    let _ = write!(
        out,
        "/// Writes the row and returns it as written, defaults and keys included.\n\
         ///\n\
         /// # Errors\n\
         ///\n\
         /// When the store refuses the row.\n\
         pub fn insert(self, store: &::forgr::store::Connection) -> ::forgr::insert::Result<Row>\n\
         where {bounds}\n\
         {{\n\
         ::forgr::insert::insert(store, {name:?}, COLUMNS, self.set)\n\
         }}\n\
         }}",
        name = table.name,
    );
}
