//! The SQL in a member's non-test src: string literals read by syn, the format
//! strings of `format!`, and each statement of its .sql files. `#[cfg(test)]`
//! code is skipped, and so is every file of a `#[cfg(test)] mod`.

use std::path::{Path, PathBuf};

use proc_macro2::{TokenStream, TokenTree};
use syn::visit::{self, Visit};

use super::{Found, Held};

/// First words of statements that read or write rows, or read or set a pragma.
const DML: [&str; 7] = [
    "SELECT", "INSERT", "UPDATE", "DELETE", "REPLACE", "WITH", "PRAGMA",
];
/// First words of statements that change the schema.
const DDL: [&str; 3] = ["CREATE", "ALTER", "DROP"];

/// The SQL in the non-test src of the member at `dir`, its places relative to
/// the workspace `root`, in file then source order.
pub(super) fn member(root: &Path, dir: &Path) -> Result<Vec<Found>, String> {
    let mut files = Vec::new();
    walk(&dir.join("src"), &mut files)?;
    files.sort();

    let mut test_paths = Vec::new();
    let mut by_file = Vec::new();
    for file in files {
        let text =
            std::fs::read_to_string(&file).map_err(|err| format!("{}: {err}", file.display()))?;
        let at = relative(root, &file);
        let mut found = Vec::new();
        match file.extension().and_then(|e| e.to_str()) {
            Some("rs") => match syn::parse_file(&text) {
                Ok(ast) => {
                    let mut literals = Literals::default();
                    literals.visit_file(&ast);
                    test_paths.extend(literals.test_mods.iter().flat_map(|m| m.resolve(&file)));
                    found.extend(literals.found.into_iter().map(|(line, sql, held)| Found {
                        at: format!("{at}:{line}"),
                        sql,
                        held,
                    }));
                }
                Err(err) => found.push(Found {
                    at,
                    sql: format!("syn could not read this file: {err}"),
                    held: Held::Unread,
                }),
            },
            Some("sql") => {
                found.extend(split(&text).into_iter().enumerate().map(|(n, sql)| Found {
                    at: format!("{at} #{}", n + 1),
                    sql,
                    held: Held::Statement,
                }));
            }
            _ => {}
        }
        by_file.push((file, found));
    }
    Ok(by_file
        .into_iter()
        .filter(|(file, _)| !test_paths.iter().any(|p| file.starts_with(p)))
        .flat_map(|(_, found)| found)
        .collect())
}

/// Every .rs and .sql file under `dir`.
fn walk(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir).map_err(|err| format!("{}: {err}", dir.display()))? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.is_dir() {
            walk(&path, files)?;
        } else if path.extension().is_some_and(|e| e == "rs" || e == "sql") {
            files.push(path);
        }
    }
    Ok(())
}

/// `file` relative to `root`, with forward slashes on every platform.
fn relative(root: &Path, file: &Path) -> String {
    let rel = file.strip_prefix(root).unwrap_or(file);
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// The first word of `sql` past whitespace and comments, upper-cased, when it
/// starts a statement that reads, writes, runs a pragma or changes the schema.
fn keyword(sql: &str) -> Option<String> {
    let word: String = skip_comments(sql)
        .chars()
        .take_while(char::is_ascii_alphabetic)
        .collect();
    let word = word.to_ascii_uppercase();
    (DML.contains(&word.as_str()) || DDL.contains(&word.as_str())).then_some(word)
}

/// Whether `sql`'s first statement changes the schema.
pub(super) fn is_ddl(sql: &str) -> bool {
    keyword(sql).is_some_and(|word| DDL.contains(&word.as_str()))
}

/// `sql` past leading whitespace, `--` comments and `/* */` comments.
pub(super) fn skip_comments(mut sql: &str) -> &str {
    loop {
        sql = sql.trim_start();
        if let Some(rest) = sql.strip_prefix("--") {
            sql = rest.split_once('\n').map_or("", |(_, rest)| rest);
        } else if let Some(rest) = sql.strip_prefix("/*") {
            sql = rest.split_once("*/").map_or("", |(_, rest)| rest);
        } else {
            return sql;
        }
    }
}

/// The statements of `sql`, split at each `;` outside quotes and comments,
/// leaving out those that hold only whitespace and comments.
pub(super) fn split(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut start = 0;
    let mut chars = sql.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let close = match c {
            '\'' => Some("'"),
            '"' => Some("\""),
            '`' => Some("`"),
            '[' => Some("]"),
            '-' if chars.peek().is_some_and(|&(_, n)| n == '-') => Some("\n"),
            '/' if chars.peek().is_some_and(|&(_, n)| n == '*') => Some("*/"),
            ';' => {
                statements.push(&sql[start..i]);
                start = i + 1;
                None
            }
            _ => None,
        };
        if let Some(close) = close {
            let from = i + c.len_utf8();
            let end = sql[from..]
                .find(close)
                .map_or(sql.len(), |at| from + at + close.len());
            while chars.peek().is_some_and(|&(j, _)| j < end) {
                chars.next();
            }
        }
    }
    statements.push(&sql[start..]);
    statements
        .into_iter()
        .filter(|s| !skip_comments(s).is_empty())
        .map(|s| s.trim().to_owned())
        .collect()
}

/// A `#[cfg(test)] mod name;` and its `#[path]`, if it has one.
struct TestMod {
    name: String,
    path: Option<String>,
}

impl TestMod {
    /// The files and directory that hold the module, declared in `file`.
    fn resolve(&self, file: &Path) -> Vec<PathBuf> {
        let Some(dir) = file.parent() else {
            return Vec::new();
        };
        if let Some(path) = &self.path {
            return vec![dir.join(path)];
        }
        let owns_dir = file
            .file_name()
            .is_some_and(|n| n == "mod.rs" || n == "lib.rs" || n == "main.rs");
        let base = if owns_dir {
            dir.to_path_buf()
        } else {
            dir.join(file.file_stem().unwrap_or_default())
        };
        vec![
            base.join(format!("{}.rs", self.name)),
            base.join(&self.name),
        ]
    }
}

/// The SQL-looking string literals of one file, outside `#[cfg(test)]` code
/// and attributes, and its `#[cfg(test)] mod name;` declarations.
#[derive(Default)]
struct Literals {
    found: Vec<(usize, String, Held)>,
    test_mods: Vec<TestMod>,
}

impl Literals {
    fn literal(&mut self, lit: &syn::LitStr, format: bool) {
        let sql = lit.value();
        if keyword(&sql).is_some() {
            let held = if format { Held::Format } else { Held::Literal };
            self.found.push((lit.span().start().line, sql, held));
        }
    }

    /// The string literals among a macro's tokens; with `format`, the first
    /// token is the format string of a `format!`.
    fn tokens(&mut self, tokens: TokenStream, format: bool) {
        let tokens: Vec<TokenTree> = tokens.into_iter().collect();
        for (i, token) in tokens.iter().enumerate() {
            match token {
                TokenTree::Literal(literal) => {
                    if let syn::Lit::Str(lit) = syn::Lit::new(literal.clone()) {
                        self.literal(&lit, format && i == 0);
                    }
                }
                TokenTree::Group(group) => {
                    let format = i >= 2
                        && matches!(&tokens[i - 2], TokenTree::Ident(id) if id == "format")
                        && matches!(&tokens[i - 1], TokenTree::Punct(p) if p.as_char() == '!');
                    self.tokens(group.stream(), format);
                }
                _ => {}
            }
        }
    }
}

impl<'ast> Visit<'ast> for Literals {
    fn visit_attribute(&mut self, _: &'ast syn::Attribute) {}

    fn visit_item(&mut self, item: &'ast syn::Item) {
        if is_test(item_attrs(item)) {
            if let syn::Item::Mod(module) = item
                && module.content.is_none()
            {
                self.test_mods.push(TestMod {
                    name: module.ident.to_string(),
                    path: path_attr(&module.attrs),
                });
            }
            return;
        }
        visit::visit_item(self, item);
    }

    fn visit_impl_item(&mut self, item: &'ast syn::ImplItem) {
        let attrs: &[syn::Attribute] = match item {
            syn::ImplItem::Const(i) => &i.attrs,
            syn::ImplItem::Fn(i) => &i.attrs,
            syn::ImplItem::Type(i) => &i.attrs,
            syn::ImplItem::Macro(i) => &i.attrs,
            _ => &[],
        };
        if !is_test(attrs) {
            visit::visit_impl_item(self, item);
        }
    }

    fn visit_lit_str(&mut self, lit: &'ast syn::LitStr) {
        self.literal(lit, false);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let format = mac
            .path
            .segments
            .last()
            .is_some_and(|s| s.ident == "format");
        self.tokens(mac.tokens.clone(), format);
    }
}

fn item_attrs(item: &syn::Item) -> &[syn::Attribute] {
    match item {
        syn::Item::Const(i) => &i.attrs,
        syn::Item::Enum(i) => &i.attrs,
        syn::Item::ExternCrate(i) => &i.attrs,
        syn::Item::Fn(i) => &i.attrs,
        syn::Item::ForeignMod(i) => &i.attrs,
        syn::Item::Impl(i) => &i.attrs,
        syn::Item::Macro(i) => &i.attrs,
        syn::Item::Mod(i) => &i.attrs,
        syn::Item::Static(i) => &i.attrs,
        syn::Item::Struct(i) => &i.attrs,
        syn::Item::Trait(i) => &i.attrs,
        syn::Item::TraitAlias(i) => &i.attrs,
        syn::Item::Type(i) => &i.attrs,
        syn::Item::Union(i) => &i.attrs,
        syn::Item::Use(i) => &i.attrs,
        _ => &[],
    }
}

/// Whether `attrs` hold `#[cfg(test)]`, or a `#[cfg(all(..))]` that needs test.
fn is_test(attrs: &[syn::Attribute]) -> bool {
    attrs
        .iter()
        .filter(|a| a.path().is_ident("cfg"))
        .filter_map(|a| a.parse_args::<syn::Meta>().ok())
        .any(|meta| needs_test(&meta))
}

fn needs_test(meta: &syn::Meta) -> bool {
    match meta {
        syn::Meta::Path(path) => path.is_ident("test"),
        syn::Meta::List(list) if list.path.is_ident("all") => list
            .parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            )
            .is_ok_and(|metas| metas.iter().any(needs_test)),
        _ => false,
    }
}

/// The value of a `#[path = ".."]` attribute.
fn path_attr(attrs: &[syn::Attribute]) -> Option<String> {
    attrs.iter().find_map(|a| match &a.meta {
        syn::Meta::NameValue(nv) if nv.path.is_ident("path") => match &nv.value {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) => Some(s.value()),
            _ => None,
        },
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_keeps_quoted_and_commented_semicolons() {
        let sql = "-- a; comment\nSELECT ';' FROM t; /* ; */ UPDATE t SET a = \"b;\";\n-- only a comment;";
        assert_eq!(
            split(sql),
            [
                "-- a; comment\nSELECT ';' FROM t",
                "/* ; */ UPDATE t SET a = \"b;\""
            ]
        );
    }

    #[test]
    fn literals_skip_cfg_test_and_attributes_and_mark_format() {
        let file = syn::parse_file(
            r#"
            /// SELECT in a doc comment
            const A: &str = "SELECT a FROM t";
            fn f(t: &str) -> String { format!("DELETE FROM {t}") }
            #[cfg(test)]
            mod tests { const B: &str = "SELECT b FROM t"; }
            #[cfg(test)]
            mod helpers;
            "#,
        )
        .unwrap();
        let mut literals = Literals::default();
        literals.visit_file(&file);
        let found: Vec<_> = literals
            .found
            .iter()
            .map(|(_, sql, held)| (sql.as_str(), matches!(held, Held::Format)))
            .collect();
        assert_eq!(
            found,
            [("SELECT a FROM t", false), ("DELETE FROM {t}", true)]
        );
        let lib = Path::new("src/lib.rs");
        assert_eq!(
            literals.test_mods[0].resolve(lib),
            [Path::new("src/helpers.rs"), Path::new("src/helpers")]
        );
    }
}
