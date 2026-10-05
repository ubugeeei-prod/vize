//! Document highlight provider.
//!
//! Highlights the tag-name pair of the element under the cursor, or the
//! occurrences of the identifier under the cursor, in Vue and Art documents.
//!
//! Tag highlighting resolves the enclosing element with the shared stack-based
//! scanner in [`super::tag_pair`]. The previous implementation scanned the whole
//! document for the name, so a cursor on one of four `<div>`s highlighted all
//! eight names — which destroys the one signal the feature exists to give:
//! *which* close tag belongs to the tag under the cursor (#3454).

use tower_lsp::lsp_types::{DocumentHighlight, DocumentHighlightKind, Position, Range};

use super::IdeContext;

pub struct DocumentHighlightService;

/// Forward-only cursor that converts ascending byte offsets to LSP positions
/// in a single pass over the document.
///
/// The previous code called `offset_to_position` (which re-walks the document
/// from offset 0) twice per match, making highlighting O(occurrences × length).
/// Matches are produced left-to-right, so a monotonic cursor turns the whole
/// pass into O(length). Mirrors `offset_to_position_str`: lines count `\n`,
/// columns count UTF-16 code units and reset at each newline.
struct PositionWalker<'a> {
    chars: std::str::CharIndices<'a>,
    content_len: usize,
    /// Byte offset of the next char to process.
    offset: usize,
    line: u32,
    character: u32,
}

impl<'a> PositionWalker<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            chars: content.char_indices(),
            content_len: content.len(),
            offset: 0,
            line: 0,
            character: 0,
        }
    }

    /// Advance to `target` (a byte offset >= any previously requested target)
    /// and return its (line, character) position.
    fn position_at(&mut self, target: usize) -> (u32, u32) {
        let target = target.min(self.content_len);
        while self.offset < target {
            let Some((byte, ch)) = self.chars.next() else {
                break;
            };
            if ch == '\n' {
                self.line += 1;
                self.character = 0;
            } else {
                self.character += ch.len_utf16() as u32;
            }
            self.offset = byte + ch.len_utf8();
        }
        (self.line, self.character)
    }
}

impl DocumentHighlightService {
    pub fn highlights(ctx: &IdeContext<'_>) -> Option<Vec<DocumentHighlight>> {
        let offset = ctx.offset.min(ctx.content.len());
        // A cursor inside a raw-text block (`<script>`, `<style>`) has no markup
        // region, so it never takes the tag path and falls through to the
        // binding facts below.
        if let Some(region) =
            super::sfc_region::resolve(&ctx.content, ctx.uri.path(), offset).markup
            && let Some(names) = super::tag_pair::names_at(&ctx.content, region, offset)
        {
            return Some(tag_highlights(&ctx.content, &names));
        }

        let packet = ctx.state.binding_occurrence_facts(ctx.uri, &ctx.content)?;
        let binding = packet.facts.binding_at(offset)?;
        Some(identifier_highlights(&ctx.content, &packet.facts, binding))
    }
}

fn identifier_highlights(
    content: &str,
    packet: &crate::virtual_code::PhysicalOccurrences,
    binding: vize_croquis::binding_occurrences::BindingIdentity,
) -> Vec<DocumentHighlight> {
    let mut spans = vec![(
        binding.start as usize,
        binding.end as usize,
        Some(DocumentHighlightKind::WRITE),
    )];
    spans.extend(
        packet
            .references
            .iter()
            .filter(|reference| reference.binding == binding)
            .map(|reference| {
                (
                    reference.start,
                    reference.end,
                    Some(DocumentHighlightKind::READ),
                )
            }),
    );
    spans.sort_unstable_by_key(|&(start, end, _)| (start, end));
    spans.dedup();
    let mut walker = PositionWalker::new(content);
    spans
        .into_iter()
        .map(|(start, end, kind)| {
            let (start_line, start_character) = walker.position_at(start);
            let (end_line, end_character) = walker.position_at(end);
            span_highlight(start_line, start_character, end_line, end_character, kind)
        })
        .collect()
}

/// One highlight per name of the resolved element: two for a matched pair, one
/// for a self-closing, void or unmatched tag. Never a document-wide name scan.
fn tag_highlights(content: &str, names: &super::tag_pair::TagNames) -> Vec<DocumentHighlight> {
    let mut walker = PositionWalker::new(content);
    let mut highlights = Vec::with_capacity(2);
    for (start, end) in [Some(names.first), names.second].into_iter().flatten() {
        let (start_line, start_character) = walker.position_at(start);
        let (end_line, end_character) = walker.position_at(end);
        highlights.push(span_highlight(
            start_line,
            start_character,
            end_line,
            end_character,
            Some(DocumentHighlightKind::TEXT),
        ));
    }
    highlights
}

fn span_highlight(
    start_line: u32,
    start_character: u32,
    end_line: u32,
    end_character: u32,
    kind: Option<DocumentHighlightKind>,
) -> DocumentHighlight {
    DocumentHighlight {
        range: Range {
            start: Position {
                line: start_line,
                character: start_character,
            },
            end: Position {
                line: end_line,
                character: end_character,
            },
        },
        kind,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod semantic_tests;
