//! Whole legacy positive lenses now reuse the checker revision, not request parsing.
use super::CodeLensService;
use crate::ide::{DocumentLinkService, HoverService, IdeContext};
use crate::server::ServerState;
use tower_lsp::lsp_types::{CodeLens, Command, Position, Range, Url};
use vize_incremental::DescriptorStats;

const SFC: &str = "<script setup>\r\nconst count = ref(0)\r\nconst color = 'red'\r\n</script>\r\n<template><p>日本語😀 {{ count }} {{ count }}</p></template>\r\n<style>p { color: v-bind(color); }</style>";

fn uri() -> Url {
    Url::parse("file:///project/Counter.vue").unwrap()
}
fn assert_stats(state: &ServerState, lookups: u32, parses: u32) {
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats { lookups, parses }
    );
}
fn open(state: &ServerState, uri: &Url, source: &str) {
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    state.update_virtual_docs(uri, source);
}
fn lens(line: u32, title: &str) -> CodeLens {
    CodeLens {
        range: Range {
            start: Position::new(line, 0),
            end: Position::new(line, 0),
        },
        command: Some(Command {
            title: title.into(),
            command: "vize.findReferences".into(),
            arguments: None,
        }),
        data: None,
    }
}
fn same(left: &[CodeLens], right: &[CodeLens]) {
    assert_eq!(
        serde_json::to_value(left).unwrap(),
        serde_json::to_value(right).unwrap()
    );
}

#[test]
fn repeated_lenses_share_one_owned_checker_packet_with_highlights() {
    let state = ServerState::new();
    let uri = uri();
    open(&state, &uri, SFC);
    assert_stats(&state, 1, 1);
    let expected = vec![
        lens(1, "2 template/style references"),
        lens(2, "1 template/style reference"),
    ];
    let first = CodeLensService::get_lenses(&state, SFC, &uri);
    same(&first, &expected);
    same(&CodeLensService::get_lenses(&state, SFC, &uri), &expected);
    assert_stats(&state, 0, 0);
    let before = state.binding_occurrence_facts(&uri, SFC).unwrap();
    let ctx = IdeContext::new(&state, &uri, SFC.find("{{ count").unwrap() + 4).unwrap();
    let _highlights = crate::ide::DocumentHighlightService::highlights(&ctx).unwrap();
    let after = state.binding_occurrence_facts(&uri, SFC).unwrap();
    assert!(std::sync::Arc::ptr_eq(&before, &after));
    let _links = DocumentLinkService::get_links(&state, SFC, &uri);
    let _hover = HoverService::hover(&ctx);
    assert_stats(&state, 2, 0);
}

#[test]
fn an_edit_changes_counts_and_locations_without_old_source_fallback() {
    let state = ServerState::new();
    let uri = uri();
    open(&state, &uri, SFC);
    let _stats = state.resident.take_stats();
    let edited = SFC
        .replace("{{ count }} {{ count }}", "{{ count }}")
        .replace("<script setup>", "\r\n<script setup>");
    state
        .documents
        .open(uri.clone(), edited.clone(), 1, "vue".into());
    assert!(CodeLensService::get_lenses(&state, &edited, &uri).is_empty());
    assert!(CodeLensService::get_lenses(&state, SFC, &uri).is_empty());
    state.update_virtual_docs(&uri, &edited);
    same(
        &CodeLensService::get_lenses(&state, &edited, &uri),
        &[
            lens(2, "1 template/style reference"),
            lens(3, "1 template/style reference"),
        ],
    );
    assert_stats(&state, 1, 1);
}

#[test]
fn rejected_source_has_no_packet_and_recovers_on_a_real_new_revision() {
    let state = ServerState::new();
    let uri = uri();
    open(&state, &uri, "<template><p>broken</p>");
    assert_stats(&state, 1, 1);
    assert!(CodeLensService::get_lenses(&state, "<template><p>broken</p>", &uri).is_empty());
    assert_stats(&state, 0, 0);
    open(&state, &uri, SFC);
    same(
        &CodeLensService::get_lenses(&state, SFC, &uri),
        &[
            lens(1, "2 template/style references"),
            lens(2, "1 template/style reference"),
        ],
    );
    assert_stats(&state, 1, 1);
}

#[test]
fn close_and_reopen_releases_the_packet_even_when_client_version_is_reused() {
    let state = ServerState::new();
    let uri = uri();
    open(&state, &uri, SFC);
    let first = CodeLensService::get_lenses(&state, SFC, &uri);
    state.close_document(&uri);
    assert!(CodeLensService::get_lenses(&state, SFC, &uri).is_empty());
    state
        .documents
        .open(uri.clone(), SFC.into(), 1, "vue".into());
    assert!(CodeLensService::get_lenses(&state, SFC, &uri).is_empty());
    state.update_virtual_docs(&uri, SFC);
    same(&CodeLensService::get_lenses(&state, SFC, &uri), &first);
}

#[test]
fn regular_script_lenses_preserve_their_whole_locations() {
    let state = ServerState::new();
    let uri = uri();
    let text = "<script>\nconst title = 'hello'\n</script>\n<template>{{ title }}</template>";
    open(&state, &uri, text);
    same(
        &CodeLensService::get_lenses(&state, text, &uri),
        &[lens(1, "1 template/style reference")],
    );
}

#[test]
fn lens_title_counts_template_scope_while_highlights_keep_script_uses() {
    let state = ServerState::new();
    let uri = uri();
    let text =
        "<script setup>\nconst count = 0\ncount + 1\n</script>\n<template>{{ count }}</template>";
    open(&state, &uri, text);
    same(
        &CodeLensService::get_lenses(&state, text, &uri),
        &[lens(1, "1 template/style reference")],
    );
    let packet = state.binding_occurrence_facts(&uri, text).unwrap();
    assert_eq!(packet.authored().unwrap().references.len(), 2);
}
