//! `ignorePatterns`, as eslint-plugin-vue `vue/no-undef-components` reads it:
//! a tag is skipped when a pattern matches its authored name, its kebab-case
//! form or its PascalCase form.

use regex::Regex;
use vize_l0::String;

/// Compile the configured patterns. A pattern that is not a valid regular
/// expression matches nothing.
pub(super) fn compile(patterns: &[String]) -> Vec<Regex> {
    patterns
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .collect()
}

pub(super) fn matches_any(patterns: &[Regex], tag: &str) -> bool {
    if patterns.is_empty() {
        return false;
    }
    let kebab = kebab_case(tag);
    let pascal = pascal_case(tag);
    patterns.iter().any(|pattern| {
        pattern.is_match(tag) || pattern.is_match(&kebab) || pattern.is_match(&pascal)
    })
}

/// eslint-plugin-vue `casing.kebabCase`: `_` becomes `-`, and an uppercase
/// letter after a word character gets a `-` before it.
fn kebab_case(name: &str) -> String {
    let mut result = String::with_capacity(name.len() + 4);
    let mut after_word = false;
    for character in name.chars() {
        let character = if character == '_' { '-' } else { character };
        if character.is_ascii_uppercase() && after_word {
            result.push('-');
        }
        result.extend(character.to_lowercase());
        after_word = character.is_ascii_alphanumeric();
    }
    result
}

/// eslint-plugin-vue `casing.pascalCase`.
fn pascal_case(name: &str) -> String {
    let mut camel = String::with_capacity(name.len());
    if is_pascal_case(name) {
        camel.push_str(name);
    } else {
        let mut characters = name.chars().peekable();
        while let Some(character) = characters.next() {
            if matches!(character, '-' | '_')
                && let Some(next) =
                    characters.next_if(|next| next.is_ascii_alphanumeric() || *next == '_')
            {
                camel.push(next.to_ascii_uppercase());
            } else {
                camel.push(character);
            }
        }
    }
    let mut characters = camel.chars();
    let mut result = String::with_capacity(camel.len());
    if let Some(first) = characters.next() {
        result.extend(first.to_uppercase());
        result.push_str(characters.as_str());
    }
    result
}

/// eslint-plugin-vue `casing.isPascalCase`.
fn is_pascal_case(name: &str) -> bool {
    const SYMBOLS: &str = "-_!\"#%&'()*+,./:;<=>?@[\\]^`{|}";
    !name.starts_with(|character: char| character.is_ascii_lowercase())
        && !name
            .chars()
            .any(|character| character.is_whitespace() || SYMBOLS.contains(character))
}

#[cfg(test)]
mod tests {
    use super::{compile, kebab_case, matches_any, pascal_case};
    use vize_l0::String;

    #[test]
    fn casing_matches_eslint_plugin_vue() {
        assert_eq!(kebab_case("CButton").as_str(), "c-button");
        assert_eq!(kebab_case("my_button").as_str(), "my-button");
        assert_eq!(kebab_case("XMLHttp").as_str(), "x-m-l-http");
        assert_eq!(pascal_case("c-button").as_str(), "CButton");
        assert_eq!(pascal_case("my_button").as_str(), "MyButton");
        assert_eq!(pascal_case("CButton").as_str(), "CButton");
    }

    #[test]
    fn patterns_match_any_casing_of_the_tag() {
        let patterns = compile(&[String::from("^C[A-Z].*"), String::from("(")]);
        assert_eq!(patterns.len(), 1);
        assert!(matches_any(&patterns, "CButton"));
        assert!(matches_any(&patterns, "c-button"));
        assert!(!matches_any(&patterns, "Card"));
        assert!(!matches_any(&patterns, "card"));
    }
}
