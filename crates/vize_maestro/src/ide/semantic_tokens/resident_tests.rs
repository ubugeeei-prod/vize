//! Semantic tokens read the resident descriptor (P5-6c). A full request, a
//! range request and a hover on the same buffer share one parse per revision.

use tower_lsp::lsp_types::{
    Position, Range, SemanticToken, SemanticTokens, SemanticTokensRangeResult,
    SemanticTokensResult, Url,
};
use vize_resident::DescriptorStats;

use super::{SemanticTokensService, encode_tokens, token_overlaps_range};
use crate::ide::{HoverService, IdeContext};
use crate::server::ServerState;

const SFC: &str = r#"<script setup lang="ts">
import { ref } from 'vue'
const count = ref(0)
</script>

<template>
  <button type="button" @click="count++">{{ count }}</button>
</template>

<style scoped>
button { color: red; }
</style>
"#;

fn uri() -> Url {
    Url::parse("file:///project/src/Counter.vue").unwrap()
}

fn counts(lookups: u32, parses: u32) -> DescriptorStats {
    DescriptorStats { lookups, parses }
}

fn data(tokens: SemanticTokens) -> Vec<SemanticToken> {
    tokens.data
}

fn full_data(result: Option<SemanticTokensResult>) -> Vec<SemanticToken> {
    match result {
        Some(SemanticTokensResult::Tokens(tokens)) => data(tokens),
        other => panic!("expected semantic tokens, got {other:?}"),
    }
}

fn range_data(result: Option<SemanticTokensRangeResult>) -> Vec<SemanticToken> {
    match result {
        Some(SemanticTokensRangeResult::Tokens(tokens)) => data(tokens),
        other => panic!("expected range semantic tokens, got {other:?}"),
    }
}

fn clean_tokens(text: &str, filename: &str) -> Vec<SemanticToken> {
    let descriptor =
        vize_resident::descriptor::parse_descriptor(filename, text).expect("the fixture parses");
    encode_tokens(&SemanticTokensService::tokens_from_descriptor(
        text,
        &descriptor,
    ))
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
            line: line + 1,
            character: 0,
        },
    }
}

#[test]
fn full_and_range_share_one_parse_with_hover() {
    let state = ServerState::new();
    let uri = uri();
    let full = full_data(SemanticTokensService::get_tokens(&state, SFC, &uri));
    assert_eq!(
        state.resident.take_stats(),
        counts(1, 1),
        "the first request parses"
    );
    assert_eq!(full, clean_tokens(SFC, uri.path()));
    assert!(!full.is_empty());

    let range = line_of(SFC, "{{ count }}");
    let ranged = range_data(SemanticTokensService::get_tokens_range(
        &state, SFC, &uri, range,
    ));
    assert_eq!(
        state.resident.take_stats(),
        counts(1, 0),
        "the range request reads the memo"
    );
    let descriptor =
        vize_resident::descriptor::parse_descriptor(uri.path(), SFC).expect("the fixture parses");
    let mut absolute = SemanticTokensService::tokens_from_descriptor(SFC, &descriptor);
    absolute.retain(|token| token_overlaps_range(token, range));
    assert_eq!(ranged, encode_tokens(&absolute));
    assert!(!ranged.is_empty());

    let offset = SFC.find("{{ count").unwrap() + "{{ co".len();
    let ctx = IdeContext::testing(&state, &uri, offset, String::from(SFC));
    let _hover = HoverService::hover(&ctx);
    assert_eq!(
        state.resident.take_stats(),
        counts(1, 0),
        "hover shares the semantic-token parse"
    );

    let edited = SFC.replace("{{ count }}", "{{ counts }}");
    let edited_tokens = full_data(SemanticTokensService::get_tokens(&state, &edited, &uri));
    assert_eq!(
        state.resident.take_stats(),
        counts(1, 1),
        "an edit is a new revision"
    );
    assert_eq!(edited_tokens, clean_tokens(&edited, uri.path()));
    assert_ne!(edited_tokens, full);
}

#[test]
fn an_unchanged_buffer_parses_nothing() {
    let state = ServerState::new();
    let uri = uri();
    let first = full_data(SemanticTokensService::get_tokens(&state, SFC, &uri));
    let _first_parse = state.resident.take_stats();
    let second = full_data(SemanticTokensService::get_tokens(&state, SFC, &uri));
    assert_eq!(state.resident.take_stats(), counts(1, 0));
    assert_eq!(second, first);
}

#[test]
fn a_rejected_buffer_stays_memoized() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Broken.vue").unwrap();
    let broken = "<template><div></div>";
    assert!(SemanticTokensService::get_tokens(&state, broken, &uri).is_none());
    assert_eq!(state.resident.take_stats(), counts(1, 1));
    assert!(SemanticTokensService::get_tokens(&state, broken, &uri).is_none());
    assert_eq!(state.resident.take_stats(), counts(1, 0));

    let fixed = "<template><div></div></template>\n";
    let tokens = full_data(SemanticTokensService::get_tokens(&state, fixed, &uri));
    assert_eq!(state.resident.take_stats(), counts(1, 1));
    assert_eq!(tokens, clean_tokens(fixed, uri.path()));
}

#[test]
fn art_files_do_not_parse() {
    let state = ServerState::new();
    let uri = Url::parse("file:///test.art.vue").unwrap();
    let content = "<art title=\"Button\">\n  <variant name=\"Primary\">\n  </variant>\n</art>\n";
    let tokens = full_data(SemanticTokensService::get_tokens(&state, content, &uri));
    assert_eq!(state.resident.take_stats(), counts(0, 0));
    assert!(!tokens.is_empty());
}

#[test]
fn closing_the_document_reparses() {
    let state = ServerState::new();
    let uri = uri();
    let first = full_data(SemanticTokensService::get_tokens(&state, SFC, &uri));
    state.close_document(&uri);
    let _closed = state.resident.take_stats();
    let reopened = full_data(SemanticTokensService::get_tokens(&state, SFC, &uri));
    assert_eq!(state.resident.take_stats(), counts(1, 1));
    assert_eq!(reopened, first);
}
