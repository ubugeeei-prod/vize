//! JSON access with the reference's semantics: every accessor fails instead of
//! coercing, objects iterate in key order, and only integer spellings are
//! numbers to compute with (fractions, exponents and `-0` fail closed).

use serde_json::{Map, Value};

pub type Json = Value;

pub fn get<'v>(value: &'v Json, name: &str) -> Result<&'v Json, String> {
    match value {
        Json::Object(fields) => fields
            .get(name)
            .ok_or_else(|| format!("property not found: {name}")),
        _ => Err(format!("object expected for {name}")),
    }
}

pub fn get_str(value: &Json) -> Result<&str, String> {
    value
        .as_str()
        .ok_or_else(|| String::from("string expected"))
}

pub fn get_bool(value: &Json) -> Result<bool, String> {
    value.as_bool().ok_or_else(|| String::from("Bool expected"))
}

pub fn get_arr(value: &Json) -> Result<&Vec<Json>, String> {
    value
        .as_array()
        .ok_or_else(|| String::from("array expected"))
}

pub fn get_obj(value: &Json) -> Result<&Map<String, Json>, String> {
    value
        .as_object()
        .ok_or_else(|| String::from("object expected"))
}

pub fn get_int(value: &Json) -> Result<i64, String> {
    value
        .as_i64()
        .ok_or_else(|| String::from("integer expected"))
}

pub fn get_nat(value: &Json) -> Result<u64, String> {
    value
        .as_u64()
        .ok_or_else(|| String::from("natural number expected"))
}

/// Fields in key order.
pub fn entries(value: &Json) -> Result<Vec<(&String, &Json)>, String> {
    let mut fields: Vec<_> = get_obj(value)?.iter().collect();
    fields.sort_by(|a, b| a.0.cmp(b.0));
    Ok(fields)
}

pub fn keys(value: &Json) -> Result<Vec<&str>, String> {
    Ok(entries(value)?
        .into_iter()
        .map(|(k, _)| k.as_str())
        .collect())
}

/// `value` with `name` set; only objects take fields.
pub fn set(value: &Json, name: &str, next: Json) -> Result<Json, String> {
    let mut fields = get_obj(value)?.clone();
    fields.insert(name.into(), next);
    Ok(Json::Object(fields))
}

pub fn object<const N: usize>(fields: [(&str, Json); N]) -> Json {
    Json::Object(fields.into_iter().map(|(k, v)| (k.into(), v)).collect())
}

pub fn parse(text: &str) -> Result<Json, String> {
    serde_json::from_str(text).map_err(|e| e.to_string())
}

fn escape(out: &mut String, text: &str) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c if u32::from(c) < 0x20 => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write(out: &mut String, value: &Json) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Json::Number(n) => out.push_str(&n.to_string()),
        Json::String(s) => escape(out, s),
        Json::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write(out, item);
            }
            out.push(']');
        }
        Json::Object(_) => {
            out.push('{');
            for (index, (key, item)) in entries(value).unwrap_or_default().into_iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                escape(out, key);
                out.push(':');
                write(out, item);
            }
            out.push('}');
        }
    }
}

/// Compact text with sorted keys, the spelling the committed JSON Lines use.
pub fn compress(value: &Json) -> String {
    let mut out = String::new();
    write(&mut out, value);
    out
}
