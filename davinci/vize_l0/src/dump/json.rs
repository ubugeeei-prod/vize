//! JSON escaping shared by dump observers and host feeds.

use core::fmt::Write as _;
use vize_l0::String;

/// Append `text` as a JSON string literal: `"` and `\` escaped, control
/// characters as `\n`/`\r`/`\t` or `\u00XX`, everything else (multi-byte
/// UTF-8 included) verbatim - the minimal escape set RFC 8259 requires.
pub fn push_json_string(out: &mut String, text: &str) {
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if control < ' ' => {
                let _ = write!(out, "\\u{:04x}", control as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
}
