//! Authored names that compilers write between quotes.
//!
//! A quoted name (a prop or event key, a dynamic prop name, a directive
//! argument or modifier, a slot name) is authored text, or is assembled from
//! authored text and fixed ASCII. The markup lexer ends a name at whitespace
//! and reports a double quote inside one, so a quoted name can change meaning
//! only through a backslash. A name without one is written as it is. An
//! object key is classified by [`key_spelling`], which reads a key's bytes
//! once for both questions.

use crate::String;
use crate::js_string::push_js_string_literal;

/// Whether `name` must be escaped to be written between double quotes.
#[inline(always)]
pub fn name_needs_escape(name: &str) -> bool {
    holds_backslash(name.as_bytes())
}

#[inline(always)]
fn holds_backslash(bytes: &[u8]) -> bool {
    bytes.iter().any(|&byte| byte == b'\\')
}

/// How an object key is spelled in JavaScript.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySpelling {
    /// An identifier, written bare.
    Bare,
    /// Written between double quotes as it is.
    Quoted,
    /// Written between double quotes through the string-literal writer.
    Escaped,
}

/// Classify `key` in one pass: the identifier test stops at the first byte
/// that is not an identifier byte, and only a key that needs quotes reads
/// its remaining bytes, for a backslash. An identifier is ASCII letters,
/// digits, `_` and `$`, not starting with a digit.
#[inline(always)]
pub fn key_spelling(key: &str) -> KeySpelling {
    let bytes = key.as_bytes();
    let starts_identifier = bytes
        .first()
        .is_some_and(|&byte| byte.is_ascii_alphabetic() || byte == b'_' || byte == b'$');
    let rest = if starts_identifier {
        let body = bytes.get(1..).unwrap_or_default();
        match body
            .iter()
            .position(|&byte| !(byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'))
        {
            None => return KeySpelling::Bare,
            Some(stop) => body.get(stop..).unwrap_or_default(),
        }
    } else {
        bytes
    };
    if holds_backslash(rest) {
        KeySpelling::Escaped
    } else {
        KeySpelling::Quoted
    }
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
    use super::{KeySpelling, escaped_name_contents, key_spelling, name_needs_escape};

    #[test]
    fn keys_are_spelled_in_one_pass() {
        assert_eq!(key_spelling("onClick"), KeySpelling::Bare);
        assert_eq!(key_spelling("_a$1"), KeySpelling::Bare);
        assert_eq!(key_spelling(""), KeySpelling::Quoted);
        assert_eq!(key_spelling("1a"), KeySpelling::Quoted);
        assert_eq!(key_spelling("data-a"), KeySpelling::Quoted);
        assert_eq!(key_spelling("onUpdate:modelValue"), KeySpelling::Quoted);
        assert_eq!(key_spelling("é"), KeySpelling::Quoted);
        assert_eq!(key_spelling(r"e\f"), KeySpelling::Escaped);
        assert_eq!(key_spelling(r"\a"), KeySpelling::Escaped);
        assert_eq!(key_spelling(r"a-b\c"), KeySpelling::Escaped);
    }

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
