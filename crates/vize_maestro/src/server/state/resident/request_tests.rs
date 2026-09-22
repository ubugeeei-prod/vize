//! Cross-provider cache tests exercise the real request entry points.

use crate::server::{ServerState, annotations, document_structure};
use tower_lsp::lsp_types::{
    CodeLensParams, DocumentColorParams, DocumentSymbolParams, FoldingRangeParams,
    TextDocumentIdentifier, Url,
};
use vize_resident::DescriptorStats;

const SFC: &str = "<script setup>\r\nconst count = 1\r\n</script>\r\n<template>\r\n  <div>日本語😀 {{ count }}</div>\r\n</template>\r\n<style>\r\n.a { color: #ff0000; }\r\n</style>";

fn open(state: &ServerState, uri: &Url, source: &str, version: i32) {
    state
        .documents
        .open(uri.clone(), source.into(), version, "vue".into());
}

fn responses(state: &ServerState, uri: &Url) -> serde_json::Value {
    let doc = TextDocumentIdentifier { uri: uri.clone() };
    serde_json::json!({
        "lenses": annotations::code_lens(state, &CodeLensParams {
            text_document: doc.clone(), work_done_progress_params: Default::default(), partial_result_params: Default::default(),
        }),
        "colors": annotations::document_color(state, &DocumentColorParams {
            text_document: doc.clone(), work_done_progress_params: Default::default(), partial_result_params: Default::default(),
        }),
        "symbols": document_structure::document_symbols(state, &DocumentSymbolParams {
            text_document: doc.clone(), work_done_progress_params: Default::default(), partial_result_params: Default::default(),
        }),
        "folding": document_structure::folding_ranges(state, &FoldingRangeParams {
            text_document: doc, work_done_progress_params: Default::default(), partial_result_params: Default::default(),
        }),
    })
}

fn stats(state: &ServerState, parses: u32) {
    assert_eq!(
        state.resident.take_stats(),
        DescriptorStats { lookups: 4, parses }
    );
}

fn clean(uri: &Url, source: &str) -> serde_json::Value {
    let state = ServerState::new();
    open(&state, uri, source, 1);
    responses(&state, uri)
}

#[test]
fn annotations_and_structure_share_one_descriptor_per_revision() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Counter.vue").unwrap();
    open(&state, &uri, SFC, 1);
    let first = responses(&state, &uri);
    assert!(first["lenses"].as_array().is_some_and(|v| !v.is_empty()));
    assert_eq!(first["colors"].as_array().unwrap().len(), 1);
    assert_eq!(first["symbols"].as_array().unwrap().len(), 3);
    assert!(first["folding"].as_array().is_some_and(|v| !v.is_empty()));
    stats(&state, 1);
    assert_eq!(responses(&state, &uri), first);
    stats(&state, 0);
    let edited = SFC
        .replace("#ff0000", "#0000ff")
        .replace("<script setup>", "\r\n<script setup>");
    open(&state, &uri, &edited, 2);
    let changed = responses(&state, &uri);
    stats(&state, 1);
    assert_ne!(changed, first);
    assert_eq!(changed, clean(&uri, &edited));
}

#[test]
fn rejected_revision_and_close_invalidate_all_four_providers_together() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Counter.vue").unwrap();
    open(&state, &uri, "<template><p>broken</p>", 1);
    let rejected = responses(&state, &uri);
    assert_eq!(
        rejected,
        serde_json::json!({"lenses": null, "colors": [], "symbols": null, "folding": null})
    );
    stats(&state, 1);
    assert_eq!(responses(&state, &uri), rejected);
    stats(&state, 0);
    open(&state, &uri, SFC, 2);
    let fixed = responses(&state, &uri);
    stats(&state, 1);
    assert_eq!(fixed, clean(&uri, SFC));
    state.close_document(&uri);
    let _close = state.resident.take_stats();
    open(&state, &uri, SFC, 1);
    assert_eq!(responses(&state, &uri), fixed);
    stats(&state, 1);
}
