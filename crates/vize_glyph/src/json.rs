//! JSON formatting for non-SFC sources (e.g. `package.json`, `tsconfig.json`).
//!
//! Adds the smallest first step toward replacing Prettier on project config
//! files: parse the source through `serde_json::Value` with preserved object
//! key order, then re-emit it with the indent/newline configured in
//! `FormatOptions`. JSONC (JSON-with-comments) is **not** supported in this
//! pass — comments are dropped and the file would fail to parse — see #1891
//! and #2249 for the configurable follow-up.

use crate::error::FormatError;
use crate::options::FormatOptions;
use serde_json::Value;
use vize_carton::{String, ToCompactString, cstr};

/// Format a JSON source string.
///
/// The output ends with the configured line terminator so it round-trips
/// through `vize fmt --check` (idempotent) and matches the convention used by
/// every other formatter path.
pub fn format_json_source(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Ok(String::default());
    }

    let value: Value = serde_json::from_str(trimmed)
        .map_err(|error| FormatError::JsonFormatError(error.to_compact_string()))?;

    let newline = options.newline_string();
    let indent = options.indent_string();

    let mut output: String = String::with_capacity(source.len() + 32);
    write_value(&mut output, &value, 0, indent.as_str(), newline);
    output.push_str(newline);
    Ok(output)
}

fn write_value(output: &mut String, value: &Value, depth: usize, indent: &str, newline: &str) {
    match value {
        Value::Null => output.push_str("null"),
        Value::Bool(true) => output.push_str("true"),
        Value::Bool(false) => output.push_str("false"),
        Value::Number(number) => output.push_str(cstr!("{number}").as_str()),
        Value::String(string) => write_string(output, string),
        Value::Array(items) => {
            if items.is_empty() {
                output.push_str("[]");
                return;
            }
            output.push('[');
            for (index, item) in items.iter().enumerate() {
                output.push_str(newline);
                write_indent(output, depth + 1, indent);
                write_value(output, item, depth + 1, indent, newline);
                if index + 1 < items.len() {
                    output.push(',');
                }
            }
            output.push_str(newline);
            write_indent(output, depth, indent);
            output.push(']');
        }
        Value::Object(entries) => {
            if entries.is_empty() {
                output.push_str("{}");
                return;
            }
            output.push('{');
            let len = entries.len();
            for (index, (key, item)) in entries.iter().enumerate() {
                output.push_str(newline);
                write_indent(output, depth + 1, indent);
                write_string(output, key);
                output.push_str(": ");
                write_value(output, item, depth + 1, indent, newline);
                if index + 1 < len {
                    output.push(',');
                }
            }
            output.push_str(newline);
            write_indent(output, depth, indent);
            output.push('}');
        }
    }
}

fn write_indent(output: &mut String, depth: usize, indent: &str) {
    for _ in 0..depth {
        output.push_str(indent);
    }
}

fn write_string(output: &mut String, value: &str) {
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\x08' => output.push_str("\\b"),
            '\x0c' => output.push_str("\\f"),
            ch if (ch as u32) < 0x20 => {
                let code = ch as u32;
                output.push_str(cstr!("\\u{code:04x}").as_str());
            }
            ch => output.push(ch),
        }
    }
    output.push('"');
}

#[cfg(test)]
mod tests {
    use super::{FormatOptions, format_json_source};

    fn opts() -> FormatOptions {
        FormatOptions::default()
    }

    #[test]
    fn pretty_prints_minified_object() {
        let source = r#"{"name":"vize","version":"0.259.0","keywords":["vue","toolchain"]}"#;
        let result = format_json_source(source, &opts()).unwrap();
        assert_eq!(
            result.as_str(),
            "{\n  \"name\": \"vize\",\n  \"version\": \"0.259.0\",\n  \"keywords\": [\n    \"vue\",\n    \"toolchain\"\n  ]\n}\n",
        );
    }

    #[test]
    fn preserves_key_order_from_source() {
        let source = r#"{"z":1,"a":2,"m":3}"#;
        let result = format_json_source(source, &opts()).unwrap();
        assert_eq!(
            result.as_str(),
            "{\n  \"z\": 1,\n  \"a\": 2,\n  \"m\": 3\n}\n"
        );
    }

    #[test]
    fn already_formatted_is_idempotent() {
        let source = "{\n  \"a\": 1,\n  \"b\": [\n    true,\n    null\n  ]\n}\n";
        let first = format_json_source(source, &opts()).unwrap();
        let second = format_json_source(first.as_str(), &opts()).unwrap();
        assert_eq!(first.as_str(), second.as_str());
    }

    #[test]
    fn empty_collections_stay_compact() {
        let result = format_json_source(r#"{"a":[],"b":{}}"#, &opts()).unwrap();
        assert_eq!(result.as_str(), "{\n  \"a\": [],\n  \"b\": {}\n}\n");
    }

    #[test]
    fn empty_input_yields_empty_output() {
        assert!(format_json_source("", &opts()).unwrap().is_empty());
        assert!(format_json_source("   \n\t  ", &opts()).unwrap().is_empty());
    }

    #[test]
    fn escapes_required_string_characters() {
        let source = r#"{"k":"line\nbreak\t\"quoted\""}"#;
        let result = format_json_source(source, &opts()).unwrap();
        assert!(result.contains(r#""line\nbreak\t\"quoted\"""#));
    }

    #[test]
    fn invalid_json_returns_error() {
        assert!(format_json_source("{\"a\":}", &opts()).is_err());
    }

    #[test]
    fn honors_custom_indent_width() {
        let mut options = opts();
        options.tab_width = 4;
        let result = format_json_source(r#"{"a":1}"#, &options).unwrap();
        assert_eq!(result.as_str(), "{\n    \"a\": 1\n}\n");
    }
}
