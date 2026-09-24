//! Vue's static inline-style normalization, shared by codegen and type checking.

use std::borrow::Cow;

use vize_s0::{FxHashMap, String};

/// Parse the object produced by Vue's `parseStringStyle`: comments are removed,
/// semicolons before a closing parenthesis are kept in the value, and a repeated
/// property replaces its value without changing its first insertion position.
pub fn parse_inline_style(source: &str) -> Vec<(String, String)> {
    let source = without_comments(source);
    // Equivalent to Vue's /;(?![^(]*\))/ split. Scanning backwards avoids
    // repeatedly searching the suffix for each separator.
    let mut closing_parenthesis_ahead = false;
    let mut separators = Vec::new();
    for (index, byte) in source.bytes().enumerate().rev() {
        match byte {
            b')' => closing_parenthesis_ahead = true,
            b'(' => closing_parenthesis_ahead = false,
            b';' if !closing_parenthesis_ahead => separators.push(index),
            _ => {}
        }
    }
    separators.reverse();
    separators.push(source.len());
    let mut properties: Vec<(String, String)> = Vec::new();
    let mut indices: FxHashMap<String, usize> = FxHashMap::default();
    let mut start = 0;
    for end in separators {
        let declaration = source.get(start..end).unwrap_or_default();
        start = end + 1;
        let Some((key, value)) = declaration.split_once(':') else {
            continue;
        };
        // Vue's delimiter requires at least one character after the colon.
        if value.is_empty() {
            continue;
        }
        let key = String::from(key.trim_matches(is_js_whitespace));
        let value = String::from(value.trim_matches(is_js_whitespace));
        // Vue assigns onto an ordinary JavaScript object. A string assigned
        // to its inherited __proto__ setter creates no own style property.
        if key == "__proto__" {
            continue;
        }
        if let Some(property) = indices
            .get(&key)
            .and_then(|&index| properties.get_mut(index))
        {
            property.1 = value;
        } else {
            indices.insert(key.clone(), properties.len());
            properties.push((key, value));
        }
    }
    properties
}

fn is_js_whitespace(character: char) -> bool {
    character == '\u{feff}' || (character.is_whitespace() && character != '\u{0085}')
}

fn without_comments(source: &str) -> Cow<'_, str> {
    let Some(first) = source.find("/*") else {
        return Cow::Borrowed(source);
    };
    let mut output = String::with_capacity(source.len());
    let mut copied = 0;
    let mut comment = first;
    while let Some(end) = source.get(comment + 2..).and_then(|rest| rest.find("*/")) {
        output.push_str(source.get(copied..comment).unwrap_or_default());
        copied = comment + 2 + end + 2;
        let Some(next) = source.get(copied..).and_then(|rest| rest.find("/*")) else {
            break;
        };
        comment = copied + next;
    }
    output.push_str(source.get(copied..).unwrap_or_default());
    Cow::Owned(output.into())
}
