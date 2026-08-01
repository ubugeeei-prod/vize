//! `textDocument/rangeFormatting`: format only the blocks the selection touches.
//!
//! # Why this is not "format the document and return that"
//!
//! The handler used to ignore `params.range` and return the whole-document
//! edit, which is why the capability was never advertised: "Format Selection"
//! would have silently reformatted the entire file, and a user who selected ten
//! lines would get a diff over a thousand (#3456).
//!
//! # Why it is not "format the selected text on its own"
//!
//! Handing the selected substring to a formatter loses the context that decides
//! its shape — the block's language, its base indentation, whether it is inside
//! `<script>` or `<template>`. The result would not match what Format Document
//! produces for the same lines, so the two commands would fight.
//!
//! Instead the whole document is formatted once and the result is *projected*
//! back per SFC block: only blocks whose content the requested range intersects
//! contribute a `TextEdit`. Format Selection is therefore always a subset of
//! Format Document, by construction.
//!
//! Block *tags* are never rewritten, only block content, so a selection can
//! never reorder or retag the file.

use tower_lsp::lsp_types::{Position, Range, TextEdit};

use super::blocks::block_spans;
use crate::ide::position_to_offset;

pub(crate) fn format_range(
    content: &str,
    filename: &str,
    range: Range,
    options: &vize_glyph::FormatOptions,
) -> Option<Vec<TextEdit>> {
    let start = position_to_offset(content, range.start.line, range.start.character)?;
    let end = position_to_offset(content, range.end.line, range.end.character)?;
    // A client is allowed to send an inverted range (selecting backwards).
    let (selection_start, selection_end) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };

    let allocator = vize_glyph::Allocator::with_capacity(content.len());
    let formatted = vize_glyph::format_sfc_with_allocator(content, options, &allocator).ok()?;
    if !formatted.changed {
        return Some(Vec::new());
    }

    let authored = block_spans(content, filename)?;
    let target = block_spans(&formatted.code, filename)?;
    // A formatter must not add, drop or reorder blocks. If it did, the two
    // lists cannot be paired and there is no safe partial edit to make.
    if authored.len() != target.len() {
        return None;
    }

    let mut edits = Vec::new();
    for (&(authored_start, authored_end), &(target_start, target_end)) in
        authored.iter().zip(target.iter())
    {
        if selection_start > authored_end || authored_start > selection_end {
            continue;
        }
        let new_text = &formatted.code[target_start..target_end];
        if new_text == &content[authored_start..authored_end] {
            continue;
        }
        edits.push(TextEdit {
            range: Range {
                start: offset_position(content, authored_start),
                end: offset_position(content, authored_end),
            },
            #[allow(clippy::disallowed_methods)]
            new_text: new_text.to_string(),
        });
    }

    // Block content spans are disjoint, so sorting by start is enough to make
    // the list non-overlapping and ascending, which is what clients apply.
    edits.sort_by_key(|edit| (edit.range.start.line, edit.range.start.character));
    Some(edits)
}

fn offset_position(content: &str, offset: usize) -> Position {
    let (line, character) = crate::ide::offset_to_position(content, offset);
    Position { line, character }
}

#[cfg(test)]
mod tests;
