//! JavaScript values for pug's build-time code, with the conversions
//! `pug-runtime@3.0.1` applies when it renders them.
//!
//! Only what the evaluator can build without an engine exists here:
//! primitives, array literals and plain object literals. Objects keep
//! their keys in insertion order; the evaluator refuses integer-like keys,
//! so that order is also `for…in` order.

use alloc::vec::Vec as StdVec;

use oxc_syntax::number::ToJsString;
use vize_l0::String;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Value {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    Str(String),
    Array(StdVec<Value>),
    Object(StdVec<(String, Value)>),
}

impl Value {
    pub(super) fn str(text: &str) -> Self {
        Self::Str(String::from(text))
    }

    /// `ToBoolean`.
    pub(super) fn truthy(&self) -> bool {
        match self {
            Self::Undefined | Self::Null => false,
            Self::Bool(value) => *value,
            Self::Number(value) => *value != 0.0 && !value.is_nan(),
            Self::Str(text) => !text.is_empty(),
            Self::Array(_) | Self::Object(_) => true,
        }
    }

    pub(super) fn is_nullish(&self) -> bool {
        matches!(self, Self::Undefined | Self::Null)
    }

    pub(super) fn type_of(&self) -> &'static str {
        match self {
            Self::Undefined => "undefined",
            Self::Bool(_) => "boolean",
            Self::Number(_) => "number",
            Self::Str(_) => "string",
            Self::Null | Self::Array(_) | Self::Object(_) => "object",
        }
    }

    /// `String(value)`: arrays join with `,` (nullish elements empty).
    pub(super) fn to_js_string(&self) -> String {
        match self {
            Self::Undefined => String::from("undefined"),
            Self::Null => String::from("null"),
            Self::Bool(true) => String::from("true"),
            Self::Bool(false) => String::from("false"),
            Self::Number(value) => number_string(*value),
            Self::Str(text) => text.clone(),
            Self::Array(items) => {
                let mut out = String::default();
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    if !item.is_nullish() {
                        out.push_str(&item.to_js_string());
                    }
                }
                out
            }
            Self::Object(_) => String::from("[object Object]"),
        }
    }

    /// `ToNumber`; arrays and objects convert through their string.
    pub(super) fn to_number(&self) -> f64 {
        match self {
            Self::Undefined => f64::NAN,
            Self::Null => 0.0,
            Self::Bool(value) => f64::from(u8::from(*value)),
            Self::Number(value) => *value,
            Self::Str(text) => string_to_number(text),
            Self::Array(_) | Self::Object(_) => string_to_number(&self.to_js_string()),
        }
    }

    /// `ToPrimitive`: arrays and objects become their string.
    pub(super) fn to_primitive(&self) -> Self {
        match self {
            Self::Array(_) | Self::Object(_) => Self::Str(self.to_js_string()),
            other => other.clone(),
        }
    }

    /// `===` for primitives; `None` when identity would decide.
    pub(super) fn strict_equals(&self, other: &Self) -> Option<bool> {
        match (self, other) {
            (Self::Array(_) | Self::Object(_), _) | (_, Self::Array(_) | Self::Object(_)) => None,
            (Self::Number(left), Self::Number(right)) => Some(left == right),
            (left, right) => Some(left == right),
        }
    }

    /// `JSON.stringify(value)`; `None` for `undefined`.
    pub(super) fn json(&self) -> Option<String> {
        let mut out = String::default();
        self.write_json(&mut out).then_some(out)
    }

    fn write_json(&self, out: &mut String) -> bool {
        match self {
            Self::Undefined => return false,
            Self::Null => out.push_str("null"),
            Self::Bool(_) => out.push_str(&self.to_js_string()),
            Self::Number(value) if value.is_finite() => out.push_str(&number_string(*value)),
            Self::Number(_) => out.push_str("null"),
            Self::Str(text) => json_quote(text, out),
            Self::Array(items) => {
                out.push('[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    if !item.write_json(out) {
                        out.push_str("null");
                    }
                }
                out.push(']');
            }
            Self::Object(entries) => {
                out.push('{');
                let mut first = true;
                for (key, item) in entries {
                    if matches!(item, Self::Undefined) {
                        continue;
                    }
                    if !first {
                        out.push(',');
                    }
                    first = false;
                    json_quote(key, out);
                    out.push(':');
                    item.write_json(out);
                }
                out.push('}');
            }
        }
        true
    }
}

/// `StringToNumber`: trimmed decimal, `Infinity` or `0x`/`0o`/`0b`
/// literals; anything else is `NaN`.
fn string_to_number(text: &str) -> f64 {
    let text = text.trim_matches(super::literal::is_js_space);
    if text.is_empty() {
        return 0.0;
    }
    for (prefix, radix) in [("0x", 16), ("0o", 8), ("0b", 2)] {
        let digits = text
            .get(..2)
            .filter(|head| head.eq_ignore_ascii_case(prefix))
            .and_then(|_| text.get(2..));
        if let Some(digits) = digits {
            if digits.is_empty() || !digits.chars().all(|ch| ch.is_digit(radix)) {
                return f64::NAN;
            }
            return digits.chars().fold(0.0, |value, ch| {
                value * f64::from(radix) + f64::from(ch.to_digit(radix).unwrap_or(0))
            });
        }
    }
    let unsigned = text.strip_prefix(['+', '-']).unwrap_or(text);
    if unsigned == "Infinity" {
        return if text.starts_with('-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }
    let (mantissa, exponent) = match unsigned.find(['e', 'E']) {
        Some(at) => (
            unsigned.get(..at).unwrap_or_default(),
            unsigned.get(at + 1..),
        ),
        None => (unsigned, None),
    };
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    let exponent_ok = exponent.is_none_or(|exponent| {
        let exponent = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
        !exponent.is_empty() && digits(exponent)
    });
    if (whole.is_empty() && fraction.is_empty())
        || !digits(whole)
        || !digits(fraction)
        || !exponent_ok
    {
        return f64::NAN;
    }
    text.parse().unwrap_or(f64::NAN)
}

/// `Number#toString()`.
pub(super) fn number_string(value: f64) -> String {
    String::from(value.to_js_string().as_str())
}

/// `JSON.stringify` of a string (`QuoteJSONString`).
fn json_quote(text: &str, out: &mut String) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str("\\u00");
                for shift in [4, 0] {
                    let digit = ((c as u32) >> shift) & 0xf;
                    out.push(char::from_digit(digit, 16).unwrap_or('0'));
                }
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

/// `pug_classes` for one class source: arrays join their truthy
/// classes, objects list keys with truthy values, primitives `val || ''`.
pub(super) fn classes(value: &Value) -> String {
    match value {
        Value::Array(items) => {
            let mut out = String::default();
            for item in items {
                let class = classes(item);
                if class.is_empty() {
                    continue;
                }
                if !out.is_empty() {
                    out.push(' ');
                }
                out.push_str(&class);
            }
            out
        }
        Value::Object(entries) => {
            let mut out = String::default();
            for (key, item) in entries {
                if key.is_empty() || !item.truthy() {
                    continue;
                }
                if !out.is_empty() {
                    out.push(' ');
                }
                out.push_str(key);
            }
            out
        }
        other if other.truthy() => other.to_js_string(),
        _ => String::default(),
    }
}

/// `pug_style`: objects (arrays included) render `key:value;` pairs.
pub(super) fn style(value: &Value) -> String {
    if !value.truthy() {
        return String::default();
    }
    let mut out = String::default();
    let mut pair = |key: &str, item: &Value| {
        out.push_str(key);
        out.push(':');
        out.push_str(&item.to_js_string());
        out.push(';');
    };
    match value {
        Value::Object(entries) => entries.iter().for_each(|(key, item)| pair(key, item)),
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                pair(&number_string(index as f64), item);
            }
        }
        other => return other.to_js_string(),
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{Value, classes, number_string, style};
    use alloc::vec;
    use vize_l0::String;

    #[test]
    fn numbers_print_like_javascript() {
        for (value, text) in [
            (1.0, "1"),
            (-0.0, "0"),
            (0.1 + 0.2, "0.30000000000000004"),
            (1e21, "1e+21"),
            (1.5e-7, "1.5e-7"),
            (f64::NAN, "NaN"),
            (f64::NEG_INFINITY, "-Infinity"),
        ] {
            assert_eq!(number_string(value).as_str(), text);
        }
    }

    #[test]
    fn json_and_strings_follow_the_runtime() {
        let value = Value::Object(vec![
            (
                String::from("a"),
                Value::Array(vec![Value::Undefined, Value::Number(f64::NAN)]),
            ),
            (String::from("b"), Value::Undefined),
            (String::from("c"), Value::str("q\"\u{1}")),
        ]);
        assert_eq!(
            value.json().unwrap().as_str(),
            "{\"a\":[null,null],\"c\":\"q\\\"\\u0001\"}"
        );
        let list = Value::Array(vec![Value::Null, Value::Number(2.0), Value::str("x")]);
        assert_eq!(list.to_js_string().as_str(), ",2,x");
        assert_eq!(classes(&list).as_str(), "2 x");
        assert_eq!(style(&list).as_str(), "0:null;1:2;2:x;");
    }
}
