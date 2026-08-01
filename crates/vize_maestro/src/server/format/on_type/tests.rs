use tower_lsp::lsp_types::{Position, Range, TextEdit};

use super::format_on_type;

/// A whole SFC that is `vize fmt`-clean apart from two closing braces left one
/// level too deep — one in TypeScript, one in CSS. That is the state a file is
/// in while `editor.formatOnType` keeps up with the typist.
///
/// ```text
///  0  <script setup lang="ts">
///  1  function greet() {
///  2    const name = "vize";
///  3    return name;
///  4    }
///  5  </script>
///  6
///  7  <template>
///  8    <p>
///  9      hello
/// 10    </p>
/// 11  </template>
/// 12
/// 13  <style scoped>
/// 14  .box {
/// 15    color: red;
/// 16    }
/// 17  </style>
/// ```
const SOURCE: &str = "<script setup lang=\"ts\">\nfunction greet() {\n  const name = \"vize\";\n  return name;\n  }\n</script>\n\n<template>\n  <p>\n    hello\n  </p>\n</template>\n\n<style scoped>\n.box {\n  color: red;\n  }\n</style>\n";

/// SOURCE with both braces where `vize fmt` puts them.
const FORMATTED: &str = "<script setup lang=\"ts\">\nfunction greet() {\n  const name = \"vize\";\n  return name;\n}\n</script>\n\n<template>\n  <p>\n    hello\n  </p>\n</template>\n\n<style scoped>\n.box {\n  color: red;\n}\n</style>\n";

fn edits_at(source: &str, line: u32) -> Option<Vec<TextEdit>> {
    format_on_type(
        source,
        "/App.vue",
        Position::new(line, 0),
        &vize_glyph::FormatOptions::default(),
    )
}

/// The one edit that drops `width` leading columns from `line`.
fn dedent(line: u32, width: u32) -> Option<Vec<TextEdit>> {
    Some(vec![TextEdit {
        range: Range {
            start: Position { line, character: 0 },
            end: Position {
                line,
                character: width,
            },
        },
        new_text: String::new(),
    }])
}

#[test]
fn a_closing_brace_loses_its_stray_indent() {
    assert_eq!(edits_at(SOURCE, 4), dedent(4, 2));
}

#[test]
fn the_same_holds_for_a_closing_brace_in_css() {
    // Proof the answer is the SFC formatter's and not a TypeScript-only path:
    // the CSS rule's brace is fixed on its own, without the script's coming
    // along for the ride.
    assert_eq!(edits_at(SOURCE, 16), dedent(16, 2));
}

#[test]
fn an_already_correct_line_yields_no_edit() {
    // Line 2 is `vize fmt`-clean even though the document is not.
    assert_eq!(edits_at(SOURCE, 2), Some(Vec::new()));
}

#[test]
fn lines_outside_block_content_yield_no_edit() {
    for line in [
        0,  // the `<script setup>` tag itself
        6,  // the blank line between two blocks
        7,  // the `<template>` tag itself
        12, // the blank line before `<style>`
    ] {
        assert_eq!(
            edits_at(SOURCE, line),
            Some(Vec::new()),
            "line {line} is not block content"
        );
    }
}

#[test]
fn a_line_whose_content_the_formatter_would_rewrite_is_left_alone() {
    // `const   name="vize"` needs more than an indent change. Rewriting it
    // under the caret would undo what the user is halfway through typing.
    let source = SOURCE.replace("  const name = \"vize\";", "  const   name=\"vize\"");
    assert_eq!(edits_at(&source, 2), Some(Vec::new()));
}

#[test]
fn a_block_the_formatter_reflows_declines_without_silencing_its_neighbours() {
    // `<p>hello</p>` on one line becomes three, so template line N is no longer
    // formatted line N and the template can answer nothing...
    let source = SOURCE.replace("  <p>\n    hello\n  </p>", "  <p>hello</p>");
    assert_eq!(edits_at(&source, 8), Some(Vec::new()));
    // ...while the script block, paired independently, still re-indents. This
    // is the whole reason the pairing is per block.
    assert_eq!(edits_at(&source, 4), dedent(4, 2));
}

#[test]
fn a_crlf_document_is_re_indented_like_an_lf_one() {
    // The formatter writes LF by default, so the authored line keeps a '\r'
    // the formatted line does not have. That difference is the line ending,
    // not content the formatter wants rewritten.
    let source = SOURCE.replace('\n', "\r\n");
    assert_eq!(edits_at(&source, 4), dedent(4, 2));
}

#[test]
fn a_line_the_document_does_not_have_is_not_answerable() {
    assert_eq!(edits_at(SOURCE, 900), None);
}

#[test]
fn an_already_formatted_document_needs_no_edit() {
    for line in 0..18 {
        assert_eq!(
            edits_at(FORMATTED, line),
            Some(Vec::new()),
            "formatted line {line}"
        );
    }
}

#[test]
fn a_whitespace_only_line_loses_the_indent_the_editor_guessed() {
    // Pressing Enter leaves the caret on a line the editor pre-indented by its
    // own rules; `\n` is a trigger character precisely so that guess can be
    // corrected against the formatter's.
    let source = FORMATTED.replace("  return name;", "    \n  return name;");
    assert_eq!(edits_at(&source, 3), dedent(3, 4));
}
