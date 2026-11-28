//! eslint-plugin-vue's casing helpers: the `pascalCase` / `kebabCase`
//! converters `vue/match-component-file-name` compares with when
//! `shouldMatchCase` is false (the file stem is not rewritten), and the
//! `isPascalCase` / `isKebabCase` checkers.

use vize_l0::String;

pub(in crate::rules::vue) fn names_match(name: &str, stem: &str) -> bool {
    pascal_case(name) == stem || kebab_case(name) == stem
}

/// Upstream `isPascalCase`: no symbol, separator or whitespace, and no
/// lowercase ASCII first letter. The empty name passes, as upstream.
pub(in crate::rules::vue) fn is_pascal(value: &str) -> bool {
    !value.starts_with(|ch: char| ch.is_ascii_lowercase())
        && value
            .chars()
            .all(|ch| !is_casing_symbol(ch) && ch != '-' && ch != '_' && !ch.is_whitespace())
}

/// Upstream `isKebabCase`: no uppercase ASCII letter, symbol, underscore,
/// whitespace, leading hyphen or doubled hyphen. The empty name passes.
pub(in crate::rules::vue) fn is_kebab(value: &str) -> bool {
    !value.starts_with('-')
        && !value.contains("--")
        && value.chars().all(|ch| {
            !ch.is_ascii_uppercase() && !is_casing_symbol(ch) && ch != '_' && !ch.is_whitespace()
        })
}

pub(in crate::rules::vue) fn pascal_case(value: &str) -> String {
    // The empty name takes either branch to "".
    let camel = if is_pascal(value) {
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

pub(in crate::rules::vue) fn kebab_case(value: &str) -> String {
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
