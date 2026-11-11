//! forgr-macros: code generation for `schema!`, re-exported by forgr.

use std::fmt::Write as _;

use proc_macro::{TokenStream, TokenTree};

/// `schema!("<path>/schema.json")`, the path relative to the calling crate's
/// manifest directory: one module per table of the schema.json, which is read
/// at expansion by forgr-schema and tracked with `include_str!`, so an edit to
/// the file alone expands the call again.
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
        "const _: &str = ::core::include_str!(::core::concat!(::core::env!(\"CARGO_MANIFEST_DIR\"), \"/\", {path:?}));"
    );
    for table in &schema.tables {
        let _ = write!(out, "pub mod {} {{}}", table.name);
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
