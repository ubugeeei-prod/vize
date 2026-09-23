//! P5-6a: the hover, completion and definition requests between two
//! keystrokes share one parse — exact accounting from the resident tier.

use tower_lsp::lsp_types::{DiagnosticSeverity, Range, Url};
use vize_resident::DescriptorStats;

use crate::ide::diagnostics::sources;
use crate::ide::{
    CompletionService, DefinitionService, DiagnosticService, HoverService, IdeContext,
};
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

/// The buffer after keystroke `k`: consecutive keystrokes always differ.
fn after_keystroke(k: usize) -> String {
    let mut text = String::from(SFC);
    text.push_str(if k.is_multiple_of(2) { " " } else { "  " });
    text
}

fn uri() -> Url {
    Url::parse("file:///project/src/Counter.vue").unwrap()
}

/// Hover, completion and definition at the `{{ count }}` interpolation.
fn request_wave(state: &ServerState, uri: &Url, text: &str) {
    let offset = text.find("{{ count").unwrap() + "{{ co".len();
    let hover = IdeContext::testing(state, uri, offset, String::from(text));
    let _hover = HoverService::hover(&hover);
    let completion = IdeContext::testing_completion(state, uri, offset, String::from(text));
    let _completion = CompletionService::complete(&completion);
    let definition = IdeContext::testing(state, uri, offset, String::from(text));
    let _definition = DefinitionService::definition(&definition);
}

#[test]
fn every_request_between_two_keystrokes_shares_one_parse() {
    let state = ServerState::new();
    let uri = uri();
    for k in 0..6 {
        request_wave(&state, &uri, &after_keystroke(k));
        assert_eq!(
            state.resident.take_stats(),
            DescriptorStats {
                lookups: 3,
                parses: 1,
            },
            "keystroke {k}: three requests, one parse",
        );
    }
}

#[test]
fn a_request_on_an_unchanged_buffer_parses_nothing() {
    let state = ServerState::new();
    let uri = uri();
    let text = after_keystroke(0);
    request_wave(&state, &uri, &text);
    let _first = state.resident.take_stats();
    request_wave(&state, &uri, &text);
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 3,
            parses: 0,
        },
    );
}

#[test]
fn the_served_descriptor_is_the_clean_parse() {
    let state = ServerState::new();
    let uri = uri();
    for k in 0..3 {
        let text = after_keystroke(k);
        let ctx = IdeContext::testing(&state, &uri, 0, text.clone());
        let served = ctx.descriptor().expect("the SFC parses");
        let clean = vize_resident::descriptor::parse_descriptor(uri.path(), &text).unwrap();
        assert!(served == &clean, "keystroke {k}: served equals clean");
        let template = served.template.as_ref().unwrap();
        let clean_template = clean.template.as_ref().unwrap();
        assert_eq!(template.content, clean_template.content);
        assert_eq!(template.loc.start, clean_template.loc.start);
        assert_eq!(served.styles.len(), clean.styles.len());
    }
}

#[test]
fn diagnostics_publish_a_memoized_parse_error_without_a_second_parse() {
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(&serde_json::json!({
        "lint": true,
        "typecheck": true
    })));
    let uri = Url::parse("file:///Broken.vue").unwrap();
    let broken = "<template><div></div>";
    state
        .documents
        .open(uri.clone(), broken.to_string(), 1, "vue".to_string());

    let first = DiagnosticService::collect(&state, &uri);
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 1,
        },
    );
    let second = DiagnosticService::collect(&state, &uri);
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 0,
        },
    );
    assert_eq!(second, first);
    assert_eq!(first.len(), 1);
    let diagnostic = &first[0];
    assert_eq!(diagnostic.source.as_deref(), Some(sources::SFC_PARSER));
    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(diagnostic.code, None);
    assert_eq!(
        diagnostic.message,
        "Malformed <template> block: the closing tag is missing."
    );
    assert_eq!(
        diagnostic.range,
        Range {
            start: tower_lsp::lsp_types::Position {
                line: 0,
                character: 0,
            },
            end: tower_lsp::lsp_types::Position {
                line: 0,
                character: 21,
            },
        }
    );

    let fixed = "<template><div></div></template>\n";
    state
        .documents
        .open(uri.clone(), fixed.to_string(), 2, "vue".to_string());
    let repaired = DiagnosticService::collect(&state, &uri);
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 1,
        },
    );
    assert!(
        repaired
            .iter()
            .all(|diagnostic| diagnostic.source.as_deref() != Some(sources::SFC_PARSER))
    );
}

#[test]
fn a_valid_sfc_is_parsed_once_per_revision_for_diagnostics() {
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(&serde_json::json!({
        "lint": true,
        "typecheck": false,
        "ecosystem": false
    })));
    let uri = Url::parse("file:///Ok.vue").unwrap();
    state.documents.open(
        uri.clone(),
        "<template><p>ok</p></template>\n".to_string(),
        1,
        "vue".to_string(),
    );
    let first = DiagnosticService::collect(&state, &uri);
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 1,
        },
    );
    let second = DiagnosticService::collect(&state, &uri);
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 1,
            parses: 0,
        },
    );
    assert_eq!(second, first);
    assert!(
        first
            .iter()
            .all(|diagnostic| diagnostic.source.as_deref() != Some(sources::SFC_PARSER))
    );
}

#[test]
fn closing_a_document_releases_its_buffer() {
    let state = ServerState::new();
    let uri = uri();
    let text = after_keystroke(0);
    request_wave(&state, &uri, &text);
    state.close_document(&uri);
    let _before_reopen = state.resident.take_stats();
    request_wave(&state, &uri, &text);
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats {
            lookups: 3,
            parses: 1,
        },
        "the reopened buffer is parsed again",
    );
}
