//! Column names to Rust identifiers.

/// The Rust keywords of every edition, strict and reserved, that a raw
/// identifier can name.
const KEYWORDS: [&str; 48] = [
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "do", "dyn",
    "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl", "in", "let",
    "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref", "return",
    "static", "struct", "trait", "true", "try", "type", "typeof", "unsafe", "unsized", "use",
    "virtual", "where", "while", "yield",
];

/// The identifier for `name`: a raw identifier such as `r#type` when `name` is
/// a Rust keyword, else `name` itself.
pub(crate) fn ident(name: &str) -> String {
    if KEYWORDS.contains(&name) {
        format!("r#{name}")
    } else {
        name.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_keyword_gets_a_raw_identifier() {
        assert_eq!(ident("type"), "r#type");
        assert_eq!(ident("gen"), "r#gen");
        assert_eq!(ident("while"), "r#while");
        assert_eq!(ident("owner"), "owner");
    }
}
