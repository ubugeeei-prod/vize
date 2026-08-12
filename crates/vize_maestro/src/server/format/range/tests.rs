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
