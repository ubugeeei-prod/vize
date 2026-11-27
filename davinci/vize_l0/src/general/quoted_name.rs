//! Authored names that compilers write between quotes.
//!
//! A quoted name (a prop or event key, a dynamic prop name, a directive
//! argument or modifier, a slot name) is authored text, or is assembled from
//! authored text and fixed ASCII. The markup lexer ends a name at whitespace
//! and reports a double quote inside one, so a quoted name can change meaning
//! only through a backslash. A source without a backslash spells each of its
//! names as it is: one scan of the source stands for a scan of every name.

use crate::String;
use crate::js_string::push_js_string_literal;

/// Whether a name taken from `source` may need escaping in a JavaScript
/// string literal. An empty `source` says nothing about its names, so it may.
#[inline]
pub fn source_may_need_name_escapes(source: &str) -> bool {
    source.is_empty() || memchr::memchr(b'\\', source.as_bytes()).is_some()
}

/// The contents of the double-quoted JavaScript string literal that spells
/// `name`, when they differ from `name` itself.
#[cold]
#[inline(never)]
pub fn escaped_name_contents(name: &str) -> Option<String> {
    if memchr::memchr2(b'\\', b'"', name.as_bytes()).is_none()
        && !name.bytes().any(|byte| byte < 0x20)
    {
        return None;
    }
    Some(js_string_contents(name))
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
    use super::{escaped_name_contents, source_may_need_name_escapes};

    #[test]
    fn only_a_backslash_marks_a_source() {
        assert!(!source_may_need_name_escapes(
            r#"<div data-a="1" :b="c"></div>"#
        ));
        assert!(source_may_need_name_escapes(r#"<div :e\f="v"></div>"#));
        assert!(source_may_need_name_escapes(""));
    }

    #[test]
    fn escaped_contents_spell_the_name() {
        assert_eq!(escaped_name_contents("data-a"), None);
        assert_eq!(escaped_name_contents(r"e\f").as_deref(), Some(r"e\\f"));
        assert_eq!(escaped_name_contents("a\"b").as_deref(), Some("a\\\"b"));
        assert_eq!(escaped_name_contents("a\u{1}").as_deref(), Some("a\\u0001"));
    }
}
