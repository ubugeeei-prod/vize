//! Linked editing ranges (`textDocument/linkedEditingRange`).
//!
//! When the cursor is on a tag name, the editor keeps the open and close tag
//! names in sync as the user types. `@vue/language-server` 3.3.8 advertises
//! `linkedEditingRangeProvider: true`; Maestro advertised nothing and answered
//! `-32601 Method not found`, so renaming `<div>` in a `.vue` file left a
//! dangling `</div>` behind with no indication that anything was missing.
//!
//! Ranges are authored `.vue` coordinates, never virtual TypeScript.
//!
//! # Scope
//!
//! Only tag-name pairs are linked, matching the reference server. Self-closing
//! elements, void elements (`<br>`, `<img>`) and unmatched tags have no
//! counterpart and return `None` rather than a single-element list, because a
//! one-range response would make the editor believe it had linked something.

use tower_lsp::lsp_types::{LinkedEditingRanges, Position, Range};

use super::offset_to_position;

pub struct LinkedEditingService;

impl LinkedEditingService {
    /// Resolve the open/close tag-name pair the cursor sits on.
    ///
    /// `filename` only selects the region to scan (template block, or the whole
    /// file for standalone HTML and `.art.vue`); no server state is consulted.
    pub fn ranges(content: &str, filename: &str, offset: usize) -> Option<LinkedEditingRanges> {
        let offset = offset.min(content.len());
        let region = super::sfc_region::resolve(content, filename, offset).markup?;
        let names = super::tag_pair::names_at(content, region, offset)?;
        // A tag with no counterpart is deliberately dropped: see the module doc.
        let close = names.second?;

        Some(LinkedEditingRanges {
            // Open tag first: the reference server orders them by position and
            // clients apply edits in the order they receive them.
            ranges: vec![to_range(content, names.first), to_range(content, close)],
            // No word pattern: the reference server omits it, which lets the
            // client use its own tag-name pattern for the language.
            word_pattern: None,
        })
    }
}

fn to_range(content: &str, span: (usize, usize)) -> Range {
    let (start_line, start_character) = offset_to_position(content, span.0);
    let (end_line, end_character) = offset_to_position(content, span.1);
    Range {
        start: Position {
            line: start_line,
            character: start_character,
        },
        end: Position {
            line: end_line,
            character: end_character,
        },
    }
}

#[cfg(test)]
mod tests;
