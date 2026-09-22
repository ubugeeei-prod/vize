//! Inlay hints read the resident descriptor (P5-6c). A full request, a range
//! request and a hover on the same buffer share one parse per revision.

use tower_lsp::lsp_types::{InlayHint, InlayHintLabel, InlayHintTooltip, Position, Range, Url};
use vize_resident::DescriptorStats;

use super::InlayHintService;
use crate::ide::{HoverService, IdeContext};
use crate::server::ServerState;

const SFC: &str = r#"<script setup lang="ts">
import { ref, computed } from 'vue'
const count = ref(0)
const doubled = computed(() => count.value * 2)
</script>

<template>
  <button type="button" @click="count++">{{ count }}</button>
</template>
"#;

fn uri() -> Url {
    Url::parse("file:///project/src/Counter.vue").unwrap()
}

fn counts(lookups: u32, parses: u32) -> DescriptorStats {
    DescriptorStats { lookups, parses }
}

fn full_range() -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 100,
            character: 0,
        },
    }
}

/// The source line of `needle`, as an LSP range covering that line only.
fn line_of(text: &str, needle: &str) -> Range {
    let line = text[..text.find(needle).expect("needle")]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count() as u32;
    Range {
        start: Position { line, character: 0 },
        end: Position {
            line,
            character: 10_000,
        },
    }
}

fn label_of(hint: &InlayHint) -> String {
    match &hint.label {
        InlayHintLabel::String(text) => text.clone(),
        InlayHintLabel::LabelParts(parts) => parts.iter().map(|part| part.value.as_str()).collect(),
    }
}

fn tooltip_of(hint: &InlayHint) -> String {
    match &hint.tooltip {
        Some(InlayHintTooltip::String(text)) => text.clone(),
        Some(InlayHintTooltip::MarkupContent(markup)) => markup.value.clone(),
        None => String::new(),
    }
}

fn fingerprint(hints: &[InlayHint]) -> Vec<(u32, u32, String, String)> {
    hints
        .iter()
        .map(|hint| {
            (
                hint.position.line,
                hint.position.character,
                label_of(hint),
                tooltip_of(hint),
            )
        })
        .collect()
}

fn clean_hints(text: &str, document: &Url, range: Range) -> Vec<InlayHint> {
    let descriptor = vize_resident::descriptor::parse_descriptor(document.path(), text)
        .expect("the fixture parses");
    InlayHintService::hints_from_descriptor(text, document, range, true, &descriptor)
}

#[test]
fn full_and_range_share_one_parse_with_hover() {
    let state = ServerState::new();
    let uri = uri();
    let range = full_range();
    let full = InlayHintService::get_hints(&state, SFC, &uri, range);
    assert_eq!(
        state.resident.take_stats(),
        counts(1, 1),
        "the first request parses"
    );
    assert_eq!(
        fingerprint(&full),
        fingerprint(&clean_hints(SFC, &uri, range))
    );
    assert!(
        fingerprint(&full)
            .iter()
            .any(|hint| hint.2.contains("Ref<number>")),
        "ref(0) keeps its type hint, got {full:?}"
    );

    let count_line = line_of(SFC, "const count = ref(0)");
    let ranged = InlayHintService::get_hints(&state, SFC, &uri, count_line);
    assert_eq!(
        state.resident.take_stats(),
        counts(1, 0),
        "the range request reads the memo"
    );
    assert_eq!(
        fingerprint(&ranged),
        fingerprint(&clean_hints(SFC, &uri, count_line))
    );
    assert!(!ranged.is_empty());
    assert!(
        ranged
            .iter()
            .all(|hint| hint.position.line == count_line.start.line)
    );

    let offset = SFC.find("{{ count").unwrap() + "{{ co".len();
    let ctx = IdeContext::with_content(&state, &uri, offset, String::from(SFC));
    let _hover = HoverService::hover(&ctx);
    assert_eq!(
        state.resident.take_stats(),
        counts(1, 0),
        "hover shares the inlay-hint parse"
    );

    let edited = SFC.replace("const count = ref(0)", "const count = ref(\"\")");
    let edited_hints = InlayHintService::get_hints(&state, &edited, &uri, range);
    assert_eq!(
        state.resident.take_stats(),
        counts(1, 1),
        "an edit is a new revision"
    );
    assert_eq!(
        fingerprint(&edited_hints),
        fingerprint(&clean_hints(&edited, &uri, range))
    );
    assert_ne!(fingerprint(&edited_hints), fingerprint(&full));
    assert!(
        fingerprint(&edited_hints)
            .iter()
            .any(|hint| hint.2.contains("Ref<string>")),
        "ref(\"\") keeps its type hint, got {edited_hints:?}"
    );
}

#[test]
fn an_unchanged_buffer_parses_nothing() {
    let state = ServerState::new();
    let uri = uri();
    let range = full_range();
    let first = InlayHintService::get_hints(&state, SFC, &uri, range);
    let _first_parse = state.resident.take_stats();
    let second = InlayHintService::get_hints(&state, SFC, &uri, range);
    assert_eq!(state.resident.take_stats(), counts(1, 0));
    assert_eq!(fingerprint(&second), fingerprint(&first));
}

#[test]
fn a_rejected_buffer_stays_memoized() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Broken.vue").unwrap();
    let broken = "<template><div></div>";
    let range = full_range();
    assert!(InlayHintService::get_hints(&state, broken, &uri, range).is_empty());
    assert_eq!(state.resident.take_stats(), counts(1, 1));
    assert!(InlayHintService::get_hints(&state, broken, &uri, range).is_empty());
    assert_eq!(state.resident.take_stats(), counts(1, 0));

    let fixed = "<script setup lang=\"ts\">\nconst count = ref(0)\n</script>\n";
    let hints = InlayHintService::get_hints(&state, fixed, &uri, range);
    assert_eq!(state.resident.take_stats(), counts(1, 1));
    assert_eq!(
        fingerprint(&hints),
        fingerprint(&clean_hints(fixed, &uri, range))
    );
}

#[test]
fn closing_the_document_reparses() {
    let state = ServerState::new();
    let uri = uri();
    let range = full_range();
    let first = InlayHintService::get_hints(&state, SFC, &uri, range);
    state.close_document(&uri);
    let _closed = state.resident.take_stats();
    let reopened = InlayHintService::get_hints(&state, SFC, &uri, range);
    assert_eq!(state.resident.take_stats(), counts(1, 1));
    assert_eq!(fingerprint(&reopened), fingerprint(&first));
}
