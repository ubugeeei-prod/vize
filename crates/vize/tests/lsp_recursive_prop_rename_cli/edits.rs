#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use crate::support::position;
use serde_json::{Value, json};

pub(super) fn location(uri: &str, source: &str, needle: &str, length: usize) -> Value {
    json!({"uri":uri,"range":range(source, needle, length)})
}

pub(super) fn text_edit(source: &str, needle: &str, length: usize, replacement: &str) -> Value {
    json!({"range":range(source, needle, length),"newText":replacement})
}

pub(super) fn location_after(
    uri: &str,
    source: &str,
    prefix: &str,
    needle: &str,
    length: usize,
) -> Value {
    json!({"uri":uri,"range":range_after(source, prefix, needle, length)})
}

pub(super) fn text_edit_after(
    source: &str,
    prefix: &str,
    needle: &str,
    length: usize,
    replacement: &str,
) -> Value {
    json!({"range":range_after(source, prefix, needle, length),"newText":replacement})
}

fn range_after(source: &str, prefix: &str, needle: &str, length: usize) -> Value {
    let mut selected = range(source, &format!("{prefix}{needle}"), prefix.len() + length);
    selected["start"]["character"] =
        json!(selected["start"]["character"].as_u64().unwrap() + prefix.len() as u64);
    selected
}

fn range(source: &str, needle: &str, length: usize) -> Value {
    let start = position(source, needle);
    let mut end = start.clone();
    end["character"] = json!(start["character"].as_u64().unwrap() + length as u64);
    json!({"start":start,"end":end})
}

pub(super) fn apply(source: &str, entries: Option<&Value>) -> String {
    let mut edits = entries
        .map(|entries| entries.as_array().unwrap().clone())
        .unwrap_or_default();
    edits.sort_by_key(|edit| {
        (
            edit["range"]["start"]["line"].as_u64().unwrap(),
            edit["range"]["start"]["character"].as_u64().unwrap(),
        )
    });
    let mut result = source.to_owned();
    for edit in edits.iter().rev() {
        result.replace_range(
            offset(source, &edit["range"]["start"])..offset(source, &edit["range"]["end"]),
            edit["newText"].as_str().unwrap(),
        );
    }
    result
}

fn offset(source: &str, position: &Value) -> usize {
    let line = position["line"].as_u64().unwrap() as usize;
    let character = position["character"].as_u64().unwrap() as usize;
    let start: usize = source.split_inclusive('\n').take(line).map(str::len).sum();
    let mut utf16 = 0;
    for (at, ch) in source[start..].char_indices() {
        if utf16 == character {
            return start + at;
        }
        utf16 += ch.len_utf16();
    }
    assert_eq!(utf16, character, "invalid edit position: {position}");
    source.len()
}
