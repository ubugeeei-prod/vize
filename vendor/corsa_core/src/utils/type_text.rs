use std::fmt;

use serde::{Deserialize, Serialize};

use super::split::{split_top_level_owned, split_type_text_owned, strip_wrapping_parens};

/// Classification bucket for a rendered TypeScript type text.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeTextKind {
    /// Type text is exactly `any`.
    Any,
    /// Type text represents `bigint` or a bigint literal.
    Bigint,
    /// Type text represents `boolean`, `true`, or `false`.
    Boolean,
    /// Type text is or contains `null` or `undefined`.
    Nullish,
    /// Type text represents `number` or a numeric literal.
    Number,
    /// Type text appears to represent a `RegExp`.
    Regexp,
    /// Type text represents `string` or a string literal.
    String,
    /// Type text is exactly `unknown` or `never`.
    Unknown,
    /// Type text did not match a known coarse bucket.
    Other,
}

impl TypeTextKind {
    /// Returns the stable lowercase label used across bindings.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Bigint => "bigint",
            Self::Boolean => "boolean",
            Self::Nullish => "nullish",
            Self::Number => "number",
            Self::Regexp => "regexp",
            Self::String => "string",
            Self::Unknown => "unknown",
            Self::Other => "other",
        }
    }
}

impl fmt::Display for TypeTextKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Returns a coarse-grained classification for a rendered type text.
pub fn classify_type_text(text: Option<&str>) -> TypeTextKind {
    let Some(text) = text.map(str::trim).filter(|text| !text.is_empty()) else {
        return TypeTextKind::Other;
    };
    let text = strip_wrapping_parens(text);
    classify_atomic_type_text(text)
        .or_else(|| contains_top_level_kind(text, TypeTextKind::Nullish))
        .or_else(|| contains_top_level_kind(text, TypeTextKind::Regexp))
        .unwrap_or(TypeTextKind::Other)
}

/// Splits a type text at top-level occurrences of `delimiter`.
pub fn split_top_level_type_text(text: &str, delimiter: char) -> Vec<String> {
    split_top_level_owned(text, delimiter)
}

/// Splits a type text at top-level union and intersection boundaries.
pub fn split_type_text(text: &str) -> Vec<String> {
    split_type_text_owned(text)
}

/// Returns whether any rendered type text is string-like.
pub fn is_string_like_type_texts<T: AsRef<str>>(type_texts: &[T]) -> bool {
    matches_kind(type_texts, TypeTextKind::String)
}

/// Returns whether any rendered type text is number-like.
pub fn is_number_like_type_texts<T: AsRef<str>>(type_texts: &[T]) -> bool {
    matches_kind(type_texts, TypeTextKind::Number)
}

/// Returns whether any rendered type text is bigint-like.
pub fn is_bigint_like_type_texts<T: AsRef<str>>(type_texts: &[T]) -> bool {
    matches_kind(type_texts, TypeTextKind::Bigint)
}

/// Returns whether any rendered type text is `any`.
pub fn is_any_like_type_texts<T: AsRef<str>>(type_texts: &[T]) -> bool {
    matches_kind(type_texts, TypeTextKind::Any)
}

/// Returns whether any rendered type text is `unknown` or `never`.
pub fn is_unknown_like_type_texts<T: AsRef<str>>(type_texts: &[T]) -> bool {
    matches_kind(type_texts, TypeTextKind::Unknown)
}

/// Returns whether any rendered type text has an array-like shape.
pub fn is_array_like_type_texts<T: AsRef<str>>(type_texts: &[T]) -> bool {
    type_texts.iter().any(|text| {
        let text = text.as_ref().trim();
        text.ends_with("[]")
            || text.starts_with("Array<")
            || text.starts_with("ReadonlyArray<")
            || (text.starts_with('[') && text.ends_with(']'))
    })
}

/// Returns whether any rendered type text is a string array-like shape.
pub fn is_string_array_like_type_texts<T: AsRef<str>>(type_texts: &[T]) -> bool {
    type_texts.iter().any(|text| {
        let text = text.as_ref().trim();
        text == "string[]"
            || text == "readonly string[]"
            || text.starts_with("Array<string>")
            || text.starts_with("ReadonlyArray<string>")
    })
}

/// Returns whether type text or known properties suggest a thenable value.
pub fn is_promise_like_type_texts<T: AsRef<str>, P: AsRef<str>>(
    type_texts: &[T],
    property_names: &[P],
) -> bool {
    type_texts
        .iter()
        .any(|text| is_promise_like_type_text(text.as_ref()))
        || property_names.iter().any(|name| name.as_ref() == "then")
}

/// Returns whether type text or known properties suggest an Error-like value.
pub fn is_error_like_type_texts<T: AsRef<str>, P: AsRef<str>>(
    type_texts: &[T],
    property_names: &[P],
) -> bool {
    type_texts.iter().any(|text| {
        let text = text.as_ref().trim();
        text == "Error" || text.ends_with("Error")
    }) || {
        let mut has_message = false;
        let mut has_name = false;
        for property_name in property_names {
            match property_name.as_ref() {
                "message" => has_message = true,
                "name" => has_name = true,
                _ => {}
            }
            if has_message && has_name {
                return true;
            }
        }
        false
    }
}

fn matches_kind<T: AsRef<str>>(type_texts: &[T], kind: TypeTextKind) -> bool {
    type_texts
        .iter()
        .any(|text| classify_type_text(Some(text.as_ref())) == kind)
}

fn classify_atomic_type_text(text: &str) -> Option<TypeTextKind> {
    match text {
        "any" => Some(TypeTextKind::Any),
        "unknown" | "never" => Some(TypeTextKind::Unknown),
        "string" => Some(TypeTextKind::String),
        "number" => Some(TypeTextKind::Number),
        "bigint" => Some(TypeTextKind::Bigint),
        "boolean" | "true" | "false" => Some(TypeTextKind::Boolean),
        "null" | "undefined" => Some(TypeTextKind::Nullish),
        "RegExp" => Some(TypeTextKind::Regexp),
        _ if is_string_literal(text) => Some(TypeTextKind::String),
        _ if is_number_literal(text) => Some(TypeTextKind::Number),
        _ if is_bigint_literal(text) => Some(TypeTextKind::Bigint),
        _ => None,
    }
}

fn contains_top_level_kind(text: &str, kind: TypeTextKind) -> Option<TypeTextKind> {
    split_type_text_owned(text)
        .into_iter()
        .any(|part| classify_atomic_type_text(part.as_str()) == Some(kind))
        .then_some(kind)
}

fn is_promise_like_type_text(text: &str) -> bool {
    split_type_text_owned(strip_wrapping_parens(text))
        .into_iter()
        .any(|part| is_promise_like_atom(part.trim()))
}

fn is_promise_like_atom(text: &str) -> bool {
    matches!(text, "Promise" | "PromiseLike" | "Thenable")
        || text.starts_with("Promise<")
        || text.starts_with("PromiseLike<")
        || text.starts_with("Thenable<")
}

pub(crate) fn is_string_literal(text: &str) -> bool {
    (text.starts_with('"') && text.ends_with('"'))
        || (text.starts_with('\'') && text.ends_with('\''))
        || (text.starts_with('`') && text.ends_with('`'))
}

pub(crate) fn is_number_literal(text: &str) -> bool {
    text.parse::<f64>().is_ok()
}

pub(crate) fn is_bigint_literal(text: &str) -> bool {
    text.ends_with('n') && text[..text.len() - 1].parse::<i128>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::{
        TypeTextKind, classify_type_text, is_array_like_type_texts, is_error_like_type_texts,
        is_promise_like_type_texts, is_string_array_like_type_texts, split_top_level_type_text,
        split_type_text,
    };

    #[test]
    fn classifies_type_texts() {
        assert_eq!(classify_type_text(Some("'value'")), TypeTextKind::String);
        assert_eq!(classify_type_text(Some("42")), TypeTextKind::Number);
        assert_eq!(classify_type_text(Some("42n")), TypeTextKind::Bigint);
        assert_eq!(classify_type_text(Some("boolean")), TypeTextKind::Boolean);
        assert_eq!(
            classify_type_text(Some("null | string")),
            TypeTextKind::Nullish
        );
        assert_eq!(
            classify_type_text(Some("string | undefined")),
            TypeTextKind::Nullish
        );
        assert_eq!(
            classify_type_text(Some("Promise<string | null>")),
            TypeTextKind::Other
        );
        assert_eq!(
            classify_type_text(Some("Promise<RegExp>")),
            TypeTextKind::Other
        );
        assert_eq!(classify_type_text(Some("RegExp")), TypeTextKind::Regexp);
        assert_eq!(classify_type_text(None), TypeTextKind::Other);
    }

    #[test]
    fn splits_type_texts_at_top_level_only() {
        assert_eq!(
            split_top_level_type_text("Promise<string | number> | null", '|'),
            vec!["Promise<string | number>", "null"]
        );
        assert_eq!(
            split_type_text("string | Promise<Array<number>> & undefined"),
            vec!["string", "Promise<Array<number>>", "undefined"]
        );
    }

    #[test]
    fn detects_array_promise_and_error_shapes() {
        assert!(is_array_like_type_texts(&["ReadonlyArray<string>"]));
        assert!(is_promise_like_type_texts(
            &["Promise<string>"],
            &[] as &[&str]
        ));
        assert!(is_promise_like_type_texts(
            &["Promise<string> | null"],
            &[] as &[&str]
        ));
        assert!(!is_promise_like_type_texts(
            &["NotPromise<string>"],
            &[] as &[&str]
        ));
        assert!(is_promise_like_type_texts(
            &[] as &[&str],
            &["then", "catch"]
        ));
        assert!(is_error_like_type_texts(&["TypeError"], &[] as &[&str]));
        assert!(is_error_like_type_texts(
            &[] as &[&str],
            &["message", "name"]
        ));
    }

    #[test]
    fn detects_string_array_shapes() {
        assert!(is_string_array_like_type_texts(&["string[]"]));
        assert!(is_string_array_like_type_texts(&["readonly string[]"]));
        assert!(is_string_array_like_type_texts(&["Array<string>"]));
        assert!(is_string_array_like_type_texts(&["ReadonlyArray<string>"]));
        assert!(!is_string_array_like_type_texts(&["number[]"]));
        assert!(!is_string_array_like_type_texts(&["Array<number>"]));
    }
}
