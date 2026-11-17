//! The canonical plan serialization shared with `loader/plugin-canonical.js`:
//! compact JSON, object keys in UTF-8 byte order, integers only.

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{String, cstr};

/// Serialize `value` canonically, refusing data the JS resolver cannot produce.
pub fn canonical_json(value: &Value) -> Result<String, String> {
    let mut output = String::default();
    write_value(value, &mut output)?;
    Ok(output)
}

pub(super) fn digest(canonical: &str) -> String {
    let hash = Sha256::digest(canonical.as_bytes());
    let mut hex = String::with_capacity(hash.len() * 2);
    for byte in hash {
        hex.push_str(&cstr!("{byte:02x}"));
    }
    hex
}

fn write_value(value: &Value, output: &mut String) -> Result<(), String> {
    match value {
        Value::Null => output.push_str("null"),
        Value::Bool(true) => output.push_str("true"),
        Value::Bool(false) => output.push_str("false"),
        Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                check_safe(integer.unsigned_abs())?;
                output.push_str(&cstr!("{integer}"));
            } else if let Some(integer) = number.as_u64() {
                check_safe(integer)?;
                output.push_str(&cstr!("{integer}"));
            } else {
                return Err(cstr!("non-integer number {number} is not canonical"));
            }
        }
        Value::String(text) => write_string(text, output),
        Value::Array(items) => {
            output.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_value(item, output)?;
            }
            output.push(']');
        }
        Value::Object(members) => {
            let mut keys: Vec<&str> = members.keys().map(|key| key.as_str()).collect();
            keys.sort_unstable_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
            output.push('{');
            for (index, key) in keys.into_iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_string(key, output);
                output.push(':');
                if let Some(member) = members.get(key) {
                    write_value(member, output)?;
                }
            }
            output.push('}');
        }
    }
    Ok(())
}

const MAX_SAFE_INTEGER: u64 = (1 << 53) - 1;

fn check_safe(magnitude: u64) -> Result<(), String> {
    if magnitude > MAX_SAFE_INTEGER {
        return Err(cstr!("integer magnitude {magnitude} exceeds 2^53 - 1"));
    }
    Ok(())
}

/// `JSON.stringify` escaping for well-formed strings.
fn write_string(text: &str, output: &mut String) {
    output.push('"');
    for character in text.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{8}' => output.push_str("\\b"),
            '\u{c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                output.push_str(&cstr!("\\u{:04x}", control as u32));
            }
            other => output.push(other),
        }
    }
    output.push('"');
}
