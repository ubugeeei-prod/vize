//! Lenses keep their exact response while sharing one parse per revision.

use super::CodeLensService;
use crate::ide::{DocumentLinkService, HoverService, IdeContext};
use crate::server::ServerState;
use tower_lsp::lsp_types::{CodeLens, Url};
use vize_resident::DescriptorStats;

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

fn clean(text: &str, uri: &Url) -> Vec<CodeLens> {
    let descriptor = vize_resident::descriptor::parse_descriptor(uri.path(), text).unwrap();
    CodeLensService::lenses_from_descriptor(&descriptor)
}

fn same(left: &[CodeLens], right: &[CodeLens]) {
    assert_eq!(
        serde_json::to_value(left).unwrap(),
        serde_json::to_value(right).unwrap()
    );
}

#[test]
fn repeated_lenses_share_a_parse_with_links_and_hover() {
    let state = ServerState::new();
    let uri = uri();
    let first = CodeLensService::get_lenses(&state, SFC, &uri);
    same(&first, &clean(SFC, &uri));
    assert_eq!(first.len(), 2);
    assert_eq!(first[0].command.as_ref().unwrap().title, "2 references");
    assert_eq!(first[0].range.start.line, 2);
    assert_eq!(first[1].command.as_ref().unwrap().title, "1 reference");
    assert_eq!(first[1].range.start.line, 3);
    assert_stats(&state, 1, 1);
    same(&CodeLensService::get_lenses(&state, SFC, &uri), &first);
    assert_stats(&state, 1, 0);
    let _links = DocumentLinkService::get_links(&state, SFC, &uri);
    assert_stats(&state, 1, 0);
    let ctx = IdeContext::testing(&state, &uri, SFC.find("{{ count").unwrap() + 4, SFC.into());
    let _hover = HoverService::hover(&ctx);
    assert_stats(&state, 1, 0);
}

#[test]
fn an_edit_changes_reference_counts_without_stale_ranges() {
    let state = ServerState::new();
    let uri = uri();
    let _first = CodeLensService::get_lenses(&state, SFC, &uri);
    assert_stats(&state, 1, 1);
    let edited = SFC
        .replace("{{ count }} {{ count }}", "{{ count }}")
        .replace("<script setup>", "\r\n<script setup>");
    let lenses = CodeLensService::get_lenses(&state, &edited, &uri);
    same(&lenses, &clean(&edited, &uri));
    assert_eq!(lenses[0].command.as_ref().unwrap().title, "1 reference");
    assert_eq!(lenses[0].range.start.line, 3);
    assert_stats(&state, 1, 1);
}

#[test]
fn a_parse_rejection_is_cached_then_recovers() {
    let state = ServerState::new();
    let uri = uri();
    let broken = "<template><p>broken</p>";
    assert!(CodeLensService::get_lenses(&state, broken, &uri).is_empty());
    assert_stats(&state, 1, 1);
    assert!(CodeLensService::get_lenses(&state, broken, &uri).is_empty());
    assert_stats(&state, 1, 0);
    same(
        &CodeLensService::get_lenses(&state, SFC, &uri),
        &clean(SFC, &uri),
    );
    assert_stats(&state, 1, 1);
}

#[test]
fn close_and_reopen_releases_the_resident_input() {
    let state = ServerState::new();
    let uri = uri();
    let first = CodeLensService::get_lenses(&state, SFC, &uri);
    assert_stats(&state, 1, 1);
    state.close_document(&uri);
    let _closed = state.resident.take_stats();
    same(&CodeLensService::get_lenses(&state, SFC, &uri), &first);
    assert_stats(&state, 1, 1);
}

#[test]
fn regular_script_lenses_preserve_their_locations() {
    let state = ServerState::new();
    let uri = uri();
    let text = "<script>\nconst title = 'hello'\n</script>\n<template>{{ title }}</template>";
    let lenses = CodeLensService::get_lenses(&state, text, &uri);
    same(&lenses, &clean(text, &uri));
    assert_eq!(lenses.len(), 1);
    assert_eq!(lenses[0].range.start.line, 2);
    assert_stats(&state, 1, 1);
}
