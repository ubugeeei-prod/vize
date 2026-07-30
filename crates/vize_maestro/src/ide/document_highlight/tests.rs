use super::{DocumentHighlightService, PositionWalker};
use crate::{ide::IdeContext, server::ServerState};
use tower_lsp::lsp_types::{DocumentHighlightKind, Url};

/// The fixture from #3454. Four `<div>`s so a document-wide name scan and a
/// stack-based pair resolution give visibly different answers.
const FOUR_DIVS: &str = r#"<script setup lang="ts">
const count = 1
</script>

<template>
  <div class="a">
    <div class="b">{{ count }}</div>
  </div>
  <div class="c" />
</template>
"#;

const TEXT: Option<DocumentHighlightKind> = Some(DocumentHighlightKind::TEXT);
const READ: Option<DocumentHighlightKind> = Some(DocumentHighlightKind::READ);
const WRITE: Option<DocumentHighlightKind> = Some(DocumentHighlightKind::WRITE);

#[test]
fn position_walker_matches_offset_to_position_str() {
    // Multi-line content with a multi-byte (UTF-16 surrogate pair) char so
    // the walker's line/column tracking is exercised against the canonical
    // converter at every char boundary, including ascending re-queries.
    let content = "abc\ndé😀f\n\nghi";
    let mut walker = PositionWalker::new(content);
    let mut prev = 0usize;
    for offset in 0..=content.len() {
        if !content.is_char_boundary(offset) {
            continue;
        }
        // Walker requires monotonic targets; advance from the previous one.
        assert!(offset >= prev);
        prev = offset;
        let expected = crate::utils::offset_to_position_str(content, offset);
        let (line, character) = walker.position_at(offset);
        assert_eq!(
            (line, character),
            (expected.line, expected.character),
            "mismatch at byte offset {offset}",
        );
    }
}

fn state_for(source: &str, uri: &str, language_id: &str) -> (ServerState, Url) {
    let state = ServerState::new();
    let uri = Url::parse(uri).unwrap();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, language_id.to_string());
    state.update_virtual_docs(&uri, source);
    (state, uri)
}

/// Full response as `(line, start_character, end_character, kind)` so every test
/// can assert the complete highlight list in one `assert_eq!`.
type FlatHighlight = (u32, u32, u32, Option<DocumentHighlightKind>);

fn highlights_at(
    state: &ServerState,
    uri: &Url,
    source: &str,
    line: u32,
    character: u32,
) -> Vec<FlatHighlight> {
    let offset = crate::ide::position_to_offset(source, line, character)
        .expect("test position must resolve to a byte offset");
    let ctx = IdeContext::new(state, uri, offset).unwrap();
    let Some(highlights) = DocumentHighlightService::highlights(&ctx) else {
        return Vec::new();
    };
    highlights
        .into_iter()
        .map(|highlight| {
            assert_eq!(
                highlight.range.start.line, highlight.range.end.line,
                "a highlight never spans lines: {highlight:?}"
            );
            (
                highlight.range.start.line,
                highlight.range.start.character,
                highlight.range.end.character,
                highlight.kind,
            )
        })
        .collect()
}

#[test]
fn tag_highlight_is_the_matching_pair_not_every_same_named_tag() {
    let (state, uri) = state_for(FOUR_DIVS, "file:///App.vue", "vue");

    // Recorded from @vue/language-server 3.3.8 for this fixture at line 6,
    // character 6: [[6,5,6,8], [6,32,6,35]]. Before #3454 Maestro answered five
    // ranges here, adding the outer `<div class="a">` pair and the unrelated
    // self-closing `<div class="c" />`.
    assert_eq!(
        highlights_at(&state, &uri, FOUR_DIVS, 6, 6),
        vec![(6, 5, 8, TEXT), (6, 32, 35, TEXT)]
    );

    // The close tag name resolves back to the same pair.
    assert_eq!(
        highlights_at(&state, &uri, FOUR_DIVS, 6, 33),
        vec![(6, 5, 8, TEXT), (6, 32, 35, TEXT)]
    );

    // The outer `<div class="a">` pairs with the `</div>` on line 7, not with
    // the inner element's close tag on line 6.
    assert_eq!(
        highlights_at(&state, &uri, FOUR_DIVS, 5, 4),
        vec![(5, 3, 6, TEXT), (7, 4, 7, TEXT)]
    );
}

#[test]
fn a_self_closing_tag_highlights_only_its_own_name() {
    let (state, uri) = state_for(FOUR_DIVS, "file:///App.vue", "vue");
    assert_eq!(
        highlights_at(&state, &uri, FOUR_DIVS, 8, 4),
        vec![(8, 3, 6, TEXT)]
    );
}

#[test]
fn the_template_block_tag_highlights_as_a_pair() {
    let (state, uri) = state_for(FOUR_DIVS, "file:///App.vue", "vue");
    assert_eq!(
        highlights_at(&state, &uri, FOUR_DIVS, 4, 3),
        vec![(4, 1, 9, TEXT), (9, 2, 10, TEXT)]
    );
}

#[test]
fn a_script_body_is_never_scanned_as_markup() {
    // `a < b` in a script body must not look like a tag: the old scanner would
    // have treated `< 2` as a tag start.
    let source = "<script setup lang=\"ts\">\nconst ok = 1 < 2\n</script>\n<template><div>x</div></template>\n";
    let (state, uri) = state_for(source, "file:///App.vue", "vue");

    // The cursor sits right after the `<` on the script line; the identifier
    // scan owns this position, and there is no identifier there.
    assert_eq!(highlights_at(&state, &uri, source, 1, 14), Vec::new());
    assert_eq!(
        highlights_at(&state, &uri, source, 3, 12),
        vec![(3, 11, 14, TEXT), (3, 18, 21, TEXT)]
    );
}

#[test]
fn highlights_identifier_occurrences_in_art_variant() {
    let source = r#"<art title="Button">
  <variant name="Primary">
    <Button :label="label">{{ label }}</Button>
  </variant>
</art>

<script setup lang="ts">
const label = "Primary"
</script>"#;
    let (state, uri) = state_for(source, "file:///Button.art.vue", "art-vue");

    // Cursor on the `label` attribute *value*, which is not a tag name.
    assert_eq!(
        highlights_at(&state, &uri, source, 2, 21),
        vec![
            (2, 13, 18, READ),
            (2, 20, 25, READ),
            (2, 30, 35, READ),
            (7, 6, 11, WRITE),
        ]
    );
}

#[test]
fn highlights_matching_component_tags_in_art_variant() {
    let source = r#"<art title="Button">
  <variant name="Primary">
    <Button :label="label"><span>Label</span></Button>
  </variant>
</art>"#;
    let (state, uri) = state_for(source, "file:///Button.art.vue", "art-vue");
    assert_eq!(
        highlights_at(&state, &uri, source, 2, 6),
        vec![(2, 5, 11, TEXT), (2, 47, 53, TEXT)]
    );
}
