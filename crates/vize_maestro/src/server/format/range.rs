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

#[cfg(test)]
pub(crate) fn format_range(
    content: &str,
    filename: &str,
    range: Range,
    options: &vize_glyph::FormatOptions,
    vue_version: vize_glyph::VueVersion,
) -> Option<Vec<TextEdit>> {
    format_range_with_template_whitespace(content, filename, range, options, vue_version, false)
}

pub(crate) fn format_range_with_template_whitespace(
    content: &str,
    filename: &str,
    range: Range,
    options: &vize_glyph::FormatOptions,
    vue_version: vize_glyph::VueVersion,
    preserve_template_whitespace: bool,
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
    let formatted =
        vize_glyph::GlyphFormatter::new_with_vue_version(options, &allocator, vue_version)
            .with_template_whitespace_preserved(preserve_template_whitespace)
            .format(content)
            .ok()?;
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
        let (Some(new_text), Some(authored_text)) = (
            formatted.code.get(target_start..target_end),
            content.get(authored_start..authored_end),
        ) else {
            continue;
        };
        if new_text == authored_text {
            continue;
        }
        edits.push(TextEdit {
            range: Range {
                start: offset_position(content, authored_start),
                end: offset_position(content, authored_end),
            },
            new_text: new_text.to_owned(),
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
mod tests {
    use tower_lsp::lsp_types::{Position, Range, TextEdit};

    use super::format_range;

    /// Two badly formatted blocks, so "only the selected one changes" is visible.
    ///
    /// ```text
    /// 0  <script setup lang="ts">
    /// 1  const   a=1
    /// 2  const   b=2
    /// 3  </script>
    /// 4
    /// 5  <template>
    /// 6    <div   class="x"   >{{ a }}</div>
    /// 7  </template>
    /// ```
    const SOURCE: &str = "<script setup lang=\"ts\">\nconst   a=1\nconst   b=2\n</script>\n\n<template>\n  <div   class=\"x\"   >{{ a }}</div>\n</template>\n";

    /// Whole-document `vize fmt` output for SOURCE, as the block contents it
    /// rewrites. Recorded by running `vize fmt --write` on the fixture.
    const FORMATTED_SCRIPT: &str = "\nconst a = 1;\nconst b = 2;\n";
    const FORMATTED_TEMPLATE: &str = "\n  <div class=\"x\">{{ a }}</div>\n";

    /// `<template>`'s content, from just after the open tag to just before
    /// `</template>`.
    const TEMPLATE_CONTENT: Range = Range {
        start: Position {
            line: 5,
            character: 10,
        },
        end: Position {
            line: 7,
            character: 0,
        },
    };
    /// `<script setup>`'s content, on the same terms.
    const SCRIPT_CONTENT: Range = Range {
        start: Position {
            line: 0,
            character: 24,
        },
        end: Position {
            line: 3,
            character: 0,
        },
    };

    fn edits(source: &str, range: Range) -> Option<Vec<TextEdit>> {
        format_range(
            source,
            "/App.vue",
            range,
            &vize_glyph::FormatOptions::default(),
            vize_glyph::VueVersion::V3,
        )
    }

    fn selection(start: (u32, u32), end: (u32, u32)) -> Range {
        Range {
            start: Position {
                line: start.0,
                character: start.1,
            },
            end: Position {
                line: end.0,
                character: end.1,
            },
        }
    }

    #[test]
    fn a_selection_inside_one_block_leaves_every_other_block_untouched() {
        // Lines 1-2: entirely inside `<script setup>`. The equally badly formatted
        // template must not appear in the response at all — before #3456 this
        // handler returned a single whole-document edit, so "Format Selection"
        // rewrote the template too.
        assert_eq!(
            edits(SOURCE, selection((1, 0), (2, 11))),
            Some(vec![TextEdit {
                range: SCRIPT_CONTENT,
                new_text: FORMATTED_SCRIPT.to_owned(),
            }])
        );

        // Line 6: entirely inside `<template>`.
        assert_eq!(
            edits(SOURCE, selection((6, 0), (6, 35))),
            Some(vec![TextEdit {
                range: TEMPLATE_CONTENT,
                new_text: FORMATTED_TEMPLATE.to_owned(),
            }])
        );
    }

    #[test]
    fn a_selection_spanning_both_blocks_edits_both_in_document_order() {
        assert_eq!(
            edits(SOURCE, selection((0, 0), (7, 11))),
            Some(vec![
                TextEdit {
                    range: SCRIPT_CONTENT,
                    new_text: FORMATTED_SCRIPT.to_owned(),
                },
                TextEdit {
                    range: TEMPLATE_CONTENT,
                    new_text: FORMATTED_TEMPLATE.to_owned(),
                },
            ])
        );
    }

    #[test]
    fn an_inverted_range_selects_the_same_blocks() {
        // A backwards selection is a legal range; the client sends the anchor last.
        assert_eq!(
            edits(SOURCE, selection((2, 11), (1, 0))),
            edits(SOURCE, selection((1, 0), (2, 11)))
        );
    }

    #[test]
    fn a_caret_with_no_selection_formats_the_block_it_sits_in() {
        assert_eq!(
            edits(SOURCE, selection((1, 5), (1, 5))),
            Some(vec![TextEdit {
                range: SCRIPT_CONTENT,
                new_text: FORMATTED_SCRIPT.to_owned(),
            }])
        );
    }

    #[test]
    fn an_already_formatted_document_produces_no_edits() {
        // `Some(vec![])`, not `None`: the request succeeded and there is nothing to
        // change, which is a different answer from "this document cannot be
        // formatted".
        let formatted = "<script setup lang=\"ts\">\nconst a = 1;\n</script>\n\n<template>\n  <div class=\"x\">\n    {{ a }}\n  </div>\n</template>\n";
        assert_eq!(
            edits(formatted, selection((0, 0), (8, 11))),
            Some(Vec::new())
        );
    }

    #[test]
    fn a_selection_between_blocks_edits_nothing() {
        // Line 4 is the blank line separating the two blocks: it is inside no
        // block's content, so no block is reformatted.
        assert_eq!(edits(SOURCE, selection((4, 0), (4, 0))), Some(Vec::new()));
    }

    #[test]
    fn an_out_of_bounds_range_is_refused_rather_than_guessed() {
        assert_eq!(edits(SOURCE, selection((0, 0), (900, 0))), None);
    }
}
