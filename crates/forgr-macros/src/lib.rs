//! forgr-macros: code generation for `schema!`, re-exported by forgr.

use std::fmt::Write as _;

use proc_macro::{TokenStream, TokenTree};

mod factory;
mod ident;
mod mock;
mod table;

/// `schema!("<path>/schema.json")`, the path relative to the calling crate's
/// manifest directory: one module per table of the schema.json, each with its
/// column markers, `Row`, `factory()` and `mock(n)`
/// (a stand-in naming forgr's fake feature when it is off), and
/// `empty_store()`, the empty test store. The file is read at expansion by
/// forgr-schema and tracked by the `include_str!` `empty_store()` replays, so an
/// edit to the file alone expands the call again.
#[proc_macro]
pub fn schema(input: TokenStream) -> TokenStream {
    expand(input).unwrap_or_else(|message| {
        format!("compile_error!({message:?});")
            .parse()
            .unwrap_or_default()
    })
}

fn expand(input: TokenStream) -> Result<TokenStream, String> {
    let path = path_literal(input)?;
    let dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "schema! needs CARGO_MANIFEST_DIR, which cargo sets".to_owned())?;
    let file = std::path::Path::new(&dir).join(&path);
    let text = std::fs::read_to_string(&file)
        .map_err(|err| format!("schema! could not read {}: {err}", file.display()))?;
    let schema = forgr_schema::Schema::read(&text).map_err(|err| format!("{path}: {err}"))?;

    let mut out = format!(
        "/// The empty test store: an in-memory store with this schema.json applied \
         and foreign keys on, holding no rows.\n\
         #[must_use]\n\
         pub fn empty_store() -> ::forgr::store::Connection {{\n\
         ::forgr::store::empty_store(::core::include_str!(::core::concat!(::core::env!(\"CARGO_MANIFEST_DIR\"), \"/\", {path:?})))\n\
         }}"
    );
    for table in &schema.tables {
        let _ = write!(out, "pub mod {} {{", table.name);
        table::emit(&mut out, table);
        factory::emit(&mut out, table);
        mock::emit(&mut out, table, &schema.tables);
        out.push('}');
    }
    out.parse()
        .map_err(|err| format!("schema! generated code that does not parse: {err}"))
}

/// The one string literal `schema!` takes, unquoted.
fn path_literal(input: TokenStream) -> Result<String, String> {
    const USAGE: &str = "schema! takes one string literal: schema!(\"<path>/schema.json\")";
    let mut tokens = input.into_iter();
    let (Some(TokenTree::Literal(literal)), None) = (tokens.next(), tokens.next()) else {
        return Err(USAGE.to_owned());
    };
    let text = literal.to_string();
    text.strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .filter(|path| !path.contains('\\'))
        .map(str::to_owned)
        .ok_or_else(|| USAGE.to_owned())
}
