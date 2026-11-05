//! Upstream `pascalCase` / `kebabCase` used when `shouldMatchCase` is false.
//! The file stem is not rewritten.

use vize_carton::String;

pub(super) fn names_match(name: &str, stem: &str) -> bool {
    pascal_case(name) == stem || kebab_case(name) == stem
}

fn pascal_case(value: &str) -> String {
    let camel = if is_pascal_case(value) {
        let mut chars = value.chars();
        match chars.next() {
            Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
            None => String::default(),
        }
    } else {
        camel_from_separators(value)
    };
    let mut chars = camel.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::default(),
    }
}

fn is_pascal_case(value: &str) -> bool {
    let Some(first) = value.chars().next() else {
        return false;
    };
    !first.is_ascii_lowercase()
        && value
            .chars()
            .all(|ch| !is_casing_symbol(ch) && ch != '-' && ch != '_' && !ch.is_whitespace())
}

fn camel_from_separators(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '-' || ch == '_' {
            if let Some(next) = chars.next() {
                out.extend(next.to_uppercase());
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn kebab_case(value: &str) -> String {
    let dashed: Vec<char> = value.replace('_', "-").chars().collect();
    let mut out = String::with_capacity(dashed.len() + 4);
    for (index, ch) in dashed.iter().copied().enumerate() {
        if ch.is_ascii_uppercase()
            && index > 0
            && dashed
                .get(index - 1)
                .is_some_and(|previous| is_word(*previous))
        {
            out.push('-');
        }
        out.extend(ch.to_lowercase());
    }
    out
}

fn is_word(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn is_casing_symbol(ch: char) -> bool {
    matches!(
        ch,
        '!' | '"'
            | '#'
            | '%'
            | '&'
            | '\''
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | '.'
            | '/'
            | ':'
            | ';'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '['
            | '\\'
            | ']'
            | '^'
            | '`'
            | '{'
            | '|'
            | '}'
    )
}
