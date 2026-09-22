//! P5-6a: the hover, completion and definition requests between two
//! keystrokes share one parse — exact accounting from the resident tier.

use tower_lsp::lsp_types::Url;
use vize_resident::DescriptorStats;

use crate::ide::{CompletionService, DefinitionService, HoverService, IdeContext};
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
    let hover = IdeContext::with_content(state, uri, offset, String::from(text));
    let _hover = HoverService::hover(&hover);
    let completion =
        IdeContext::with_content_for_completion(state, uri, offset, String::from(text));
    let _completion = CompletionService::complete(&completion);
    let definition = IdeContext::with_content(state, uri, offset, String::from(text));
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
        let ctx = IdeContext::with_content(&state, &uri, 0, text.clone());
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
