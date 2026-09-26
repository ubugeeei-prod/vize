//! Existing public v3 anchors become a checked EmitDocument, then are rebased.
//!
//! Public maps expose anchors rather than full generated ranges. No range or
//! new authored provenance is invented for whitespace or output comments.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "serialized plugin boundary uses standard strings"
)]

use super::rewrite::Edit;
use crate::CompileResult;
use oxc_sourcemap::SourceMap;
use serde_json::Value;
use vize_atelier_core::codegen::document::{EmitDocument, SpanLink};
use vize_l0::{Span, String as CompactString};

pub(super) fn rebase(
    compiled: &CompileResult,
    code: &str,
    edits: &[Edit],
) -> Result<Option<Value>, String> {
    let Some(map) = &compiled.map else {
        return Ok(None);
    };
    if map.get("version").and_then(Value::as_u64) != Some(3)
        || map.get("sources").and_then(Value::as_array).map(Vec::len) != Some(1)
        || map
            .get("sourcesContent")
            .and_then(Value::as_array)
            .map(Vec::len)
            != Some(1)
    {
        return Err("output hooks require a native single-source v3 map".to_owned());
    }
    let json = map.to_string();
    let decoded = SourceMap::from_json_string(&json)
        .map_err(|e| format!("invalid output source map: {e}"))?;
    let source = decoded
        .get_source_content(0)
        .ok_or_else(|| "output map has no authored content".to_owned())?;
    let filename = decoded
        .get_source(0)
        .ok_or_else(|| "output map has no authored filename".to_owned())?;
    let mut links = Vec::new();
    let generated_lines = line_starts(&compiled.code);
    let authored_lines = line_starts(source);
    for token in decoded.get_source_view_tokens() {
        if token.get_name_id().is_some() && token.get_name().is_none() {
            return Err("output map contains an unknown authored name".to_owned());
        }
        if token.get_source_id() != Some(0) {
            return Err("output map contains non-native source anchors".to_owned());
        }
        let generated = offset(
            &compiled.code,
            &generated_lines,
            token.get_dst_line(),
            token.get_dst_col(),
        )?;
        let authored = offset(
            source,
            &authored_lines,
            token.get_src_line(),
            token.get_src_col(),
        )?;
        let generated = shifted(generated, edits)?;
        links.push(SpanLink {
            generated: Span::new(generated, generated),
            authored: Span::new(authored, authored),
            name: token.get_name().map(CompactString::new),
            segment: true,
        });
    }
    let document = EmitDocument::from_parts(CompactString::new(code), links);
    let rebuilt = document.source_map(filename, source);
    let mut rebuilt: Value = serde_json::from_str(&rebuilt).map_err(|e| e.to_string())?;
    // Preserve map metadata; only generated positions and their name indices change.
    if let (Some(original), Some(target)) = (map.as_object(), rebuilt.as_object_mut()) {
        for (key, value) in original {
            if !matches!(key.as_str(), "mappings" | "names") {
                target.insert(key.clone(), value.clone());
            }
        }
    }
    Ok(Some(rebuilt))
}

fn line_starts(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(
            text.bytes()
                .enumerate()
                .filter_map(|(index, byte)| (byte == b'\n').then_some(index + 1)),
        )
        .collect()
}

fn offset(text: &str, lines: &[usize], line: u32, column: u32) -> Result<u32, String> {
    let start = *lines
        .get(line as usize)
        .ok_or_else(|| "map line is outside text".to_owned())?;
    let mut units = 0;
    for (byte, ch) in text.get(start..).unwrap_or_default().char_indices() {
        if units == column {
            return u32::try_from(start + byte).map_err(|e| e.to_string());
        }
        if ch == '\n' {
            break;
        }
        units += ch.len_utf16() as u32;
        if units > column {
            return Err("map column splits a UTF-16 surrogate".to_owned());
        }
    }
    if units == column {
        return u32::try_from(text.len()).map_err(|e| e.to_string());
    }
    Err("map column is outside text".to_owned())
}

fn shifted(offset: u32, edits: &[Edit]) -> Result<u32, String> {
    let mut delta = 0i64;
    for edit in edits {
        if offset < edit.start {
            break;
        }
        if offset < edit.end {
            return u32::try_from(
                i64::from(edit.start)
                    + delta
                    + i64::from(offset - edit.start).min(edit.text.len() as i64),
            )
            .map_err(|e| e.to_string());
        }
        delta += edit.text.len() as i64 - i64::from(edit.end - edit.start);
    }
    u32::try_from(i64::from(offset) + delta).map_err(|e| e.to_string())
}
