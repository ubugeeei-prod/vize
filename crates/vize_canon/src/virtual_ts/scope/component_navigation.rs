use std::ops::Range;

use vize_carton::String;

pub(super) fn push_ts_single_quoted_literal(ts: &mut String, value: &str) -> Range<usize> {
    ts.push('\'');
    let start = ts.len();
    for ch in value.chars() {
        match ch {
            '\\' => ts.push_str("\\\\"),
            '\'' => ts.push_str("\\'"),
            '\n' => ts.push_str("\\n"),
            '\r' => ts.push_str("\\r"),
            '\t' => ts.push_str("\\t"),
            _ => ts.push(ch),
        }
    }
    let end = ts.len();
    ts.push('\'');
    start..end
}

pub(super) fn is_ts_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}
