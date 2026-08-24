//! `BlockLocation` anchor agreement.
//!
//! The struct carries two halves that must describe the same two points:
//! `start`/`start_line`/`start_column` all address the first content byte, and
//! `end`/`end_line`/`end_column` all address the byte just past the last.
//! Asserting every field of every block by full equality is what pins them
//! together — a partial assertion is exactly how #3602 stayed latent, because
//! the two halves only disagree once an opening tag spans more than one line.

use super::parse_sfc;
use crate::sfc::types::BlockLocation;

/// Every field of a [`BlockLocation`], so a comparison cannot silently skip one.
#[derive(Debug, PartialEq, Eq)]
struct Anchors {
    start: usize,
    end: usize,
    tag_start: usize,
    tag_end: usize,
    start_line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
}

impl From<&BlockLocation> for Anchors {
    fn from(loc: &BlockLocation) -> Self {
        Self {
            start: loc.start,
            end: loc.end,
            tag_start: loc.tag_start,
            tag_end: loc.tag_end,
            start_line: loc.start_line,
            start_column: loc.start_column,
            end_line: loc.end_line,
            end_column: loc.end_column,
        }
    }
}

/// `<template>`, `<script>`, `<script setup>`, `<style>` and a custom block,
/// each with an opening tag that wraps across lines.
const MULTILINE_OPENING_TAGS: &str = r#"<template
  lang="html"
>
  <p>ok</p>
</template>

<script
  lang="ts"
>
export default {}
</script>

<script
  setup
  lang="ts"
>
const x = 1
</script>

<style
  scoped
>
.a { color: red }
</style>

<docs
  kind="usage"
>
usage notes
</docs>
"#;

/// The same document with every opening tag on one line.
const SINGLE_LINE_OPENING_TAGS: &str = r#"<template lang="html">
  <p>ok</p>
</template>

<script lang="ts">
export default {}
</script>

<script setup lang="ts">
const x = 1
</script>

<style scoped>
.a { color: red }
</style>

<docs kind="usage">
usage notes
</docs>
"#;

fn parsed_anchors(source: &str) -> Vec<Anchors> {
    let descriptor = parse_sfc(source, Default::default()).expect("fixture must parse");
    assert_eq!(descriptor.styles.len(), 1);
    assert_eq!(descriptor.custom_blocks.len(), 1);
    vec![
        Anchors::from(&descriptor.template.as_ref().expect("template").loc),
        Anchors::from(&descriptor.script.as_ref().expect("script").loc),
        Anchors::from(&descriptor.script_setup.as_ref().expect("script setup").loc),
        Anchors::from(&descriptor.styles[0].loc),
        Anchors::from(&descriptor.custom_blocks[0].loc),
    ]
}

/// Assert the whole content slice too: an anchor that agrees with the source
/// text it delimits is the only kind that is actually verified.
fn assert_contents(source: &str, expected: [&str; 5]) {
    let descriptor = parse_sfc(source, Default::default()).expect("fixture must parse");
    let actual = [
        descriptor
            .template
            .as_ref()
            .expect("template")
            .content
            .as_ref()
            .to_string(),
        descriptor
            .script
            .as_ref()
            .expect("script")
            .content
            .as_ref()
            .to_string(),
        descriptor
            .script_setup
            .as_ref()
            .expect("script setup")
            .content
            .as_ref()
            .to_string(),
        descriptor.styles[0].content.as_ref().to_string(),
        descriptor.custom_blocks[0].content.as_ref().to_string(),
    ];
    assert_eq!(actual, expected.map(str::to_string));
}

#[test]
fn multiline_opening_tags_anchor_lines_to_the_content_offsets() {
    assert_eq!(
        parsed_anchors(MULTILINE_OPENING_TAGS),
        vec![
            Anchors {
                start: 25,
                end: 38,
                tag_start: 0,
                tag_end: 49,
                start_line: 3,
                start_column: 2,
                end_line: 5,
                end_column: 1,
            },
            Anchors {
                start: 72,
                end: 91,
                tag_start: 51,
                tag_end: 100,
                start_line: 9,
                start_column: 2,
                end_line: 11,
                end_column: 1,
            },
            Anchors {
                start: 131,
                end: 144,
                tag_start: 102,
                tag_end: 153,
                start_line: 16,
                start_column: 2,
                end_line: 18,
                end_column: 1,
            },
            Anchors {
                start: 172,
                end: 191,
                tag_start: 155,
                tag_end: 199,
                start_line: 22,
                start_column: 2,
                end_line: 24,
                end_column: 1,
            },
            Anchors {
                start: 223,
                end: 236,
                tag_start: 201,
                tag_end: 243,
                start_line: 28,
                start_column: 2,
                end_line: 30,
                end_column: 1,
            },
        ],
    );

    assert_contents(
        MULTILINE_OPENING_TAGS,
        [
            "\n  <p>ok</p>\n",
            "\nexport default {}\n",
            "\nconst x = 1\n",
            "\n.a { color: red }\n",
            "\nusage notes\n",
        ],
    );
}

#[test]
fn single_line_opening_tags_keep_every_anchor_on_the_tag_line() {
    assert_eq!(
        parsed_anchors(SINGLE_LINE_OPENING_TAGS),
        vec![
            Anchors {
                start: 22,
                end: 35,
                tag_start: 0,
                tag_end: 46,
                start_line: 1,
                start_column: 23,
                end_line: 3,
                end_column: 1,
            },
            Anchors {
                start: 66,
                end: 85,
                tag_start: 48,
                tag_end: 94,
                start_line: 5,
                start_column: 19,
                end_line: 7,
                end_column: 1,
            },
            Anchors {
                start: 120,
                end: 133,
                tag_start: 96,
                tag_end: 142,
                start_line: 9,
                start_column: 25,
                end_line: 11,
                end_column: 1,
            },
            Anchors {
                start: 158,
                end: 177,
                tag_start: 144,
                tag_end: 185,
                start_line: 13,
                start_column: 15,
                end_line: 15,
                end_column: 1,
            },
            Anchors {
                start: 206,
                end: 219,
                tag_start: 187,
                tag_end: 226,
                start_line: 17,
                start_column: 20,
                end_line: 19,
                end_column: 1,
            },
        ],
    );

    assert_contents(
        SINGLE_LINE_OPENING_TAGS,
        [
            "\n  <p>ok</p>\n",
            "\nexport default {}\n",
            "\nconst x = 1\n",
            "\n.a { color: red }\n",
            "\nusage notes\n",
        ],
    );
}

/// Each block's `start_line` is exactly the line its opening tag's `>` sits on,
/// and each `end_line` the line its closing tag starts on — stated as a
/// property over the source text rather than as pinned integers, so the two
/// fixtures above cannot both be wrong in the same direction.
#[test]
fn anchors_match_the_lines_counted_from_the_offsets() {
    for source in [MULTILINE_OPENING_TAGS, SINGLE_LINE_OPENING_TAGS] {
        let descriptor = parse_sfc(source, Default::default()).expect("fixture must parse");
        let blocks = [
            &descriptor.template.as_ref().expect("template").loc,
            &descriptor.script.as_ref().expect("script").loc,
            &descriptor.script_setup.as_ref().expect("script setup").loc,
            &descriptor.styles[0].loc,
            &descriptor.custom_blocks[0].loc,
        ];
        for loc in blocks {
            assert_eq!(
                (loc.start_line, loc.start_column),
                line_column(source, loc.start),
            );
            assert_eq!((loc.end_line, loc.end_column), line_column(source, loc.end));
        }
    }
}

/// 1-based line and 1-based byte column of `offset`, counted from scratch.
fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let prefix = &source.as_bytes()[..offset];
    let line = prefix.iter().filter(|byte| **byte == b'\n').count() + 1;
    let column = match prefix.iter().rposition(|byte| *byte == b'\n') {
        Some(newline) => offset - newline,
        None => offset + 1,
    };
    (line, column)
}
