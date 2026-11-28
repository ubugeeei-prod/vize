//! Authored names that compilers write between quotes.
//!
//! A quoted name (a prop or event key, a dynamic prop name, a directive
//! argument or modifier, a slot name) is authored text, or is assembled from
//! authored text and fixed ASCII. The markup lexer ends a name at whitespace
//! and reports a double quote inside one, so a quoted name can change meaning
//! only through a backslash. A name without one is written as it is. The lexer
//! also records whether any name of a template holds a backslash, so an
//! emitter reaches these helpers only for such a template.

use crate::String;
use crate::js_string::push_js_string_literal;

/// Whether `name` must be escaped to be written between double quotes.
#[inline(always)]
pub fn name_needs_escape(name: &str) -> bool {
    name.as_bytes().contains(&b'\\')
}

/// The contents of the double-quoted JavaScript string literal that spells
/// `name`, for a name that [`name_needs_escape`].
#[cold]
#[inline(never)]
pub fn escaped_name_contents(name: &str) -> String {
    js_string_contents(name)
}

/// The contents of the double-quoted JavaScript string literal that spells
/// `name` exactly: the literal without its quotes. `name` is final text, so no
/// entity in it is decoded.
pub fn js_string_contents(name: &str) -> String {
    let mut literal = String::with_capacity(name.len() + 2);
    push_js_string_literal(&mut literal, name);
    let end = literal.len().saturating_sub(1);
    String::from(literal.get(1..end).unwrap_or(name))
}

#[cfg(test)]
mod tests {
    use super::{escaped_name_contents, name_needs_escape};

    #[test]
    fn only_a_backslash_needs_escaping() {
        assert!(!name_needs_escape("data-a"));
        assert!(!name_needs_escape("onUpdate:modelValue"));
        assert!(name_needs_escape(r"e\f"));
    }

    #[test]
    fn escaped_contents_spell_the_name() {
        assert_eq!(escaped_name_contents(r"e\f").as_str(), r"e\\f");
        assert_eq!(escaped_name_contents("a\\\"b").as_str(), "a\\\\\\\"b");
        assert_eq!(escaped_name_contents("a\\\u{1}").as_str(), "a\\\\\\u0001");
    }
}
