//! Seventh pinned event owner; original six-case corpus remains unchanged.

use super::{CompletionService, IdeContext, ServerState, imported_declared_events};
use std::fs;
use tower_lsp::lsp_types::{CompletionItemKind, CompletionResponse, InsertTextFormat, Url};
use vize_l0::String;

const CHILD: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/airi-authored-events/CharacterItem.vue.txt"
);
const AUTHORITY: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/airi-authored-events/case.json"
);

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Authority {
    predecessor_completion_count: usize,
    predecessor_completion_items: Vec<RankedItem>,
    event_labels: Vec<String>,
    declaration: String,
}

#[derive(Debug, PartialEq, serde::Deserialize)]
struct RankedItem {
    label: String,
    rank: usize,
}

#[test]
fn pinned_airi_component_returns_the_complete_predecessor_and_exact_declared_event_bank() {
    let authority: Authority = serde_json::from_str(AUTHORITY).unwrap();
    assert_eq!(authority.predecessor_completion_count, 40);
    assert_eq!(authority.predecessor_completion_items.len(), 40);
    assert!(CHILD.contains(authority.declaration.as_str()));
    assert_eq!(imported_declared_events(CHILD), authority.event_labels);

    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("CharacterItem.vue"), CHILD).unwrap();
    let source = "<script setup lang='ts'>\nimport CharacterItem from './CharacterItem.vue'\n</script>\n<template><CharacterItem  /></template>";
    let path = dir.path().join("Parent.vue");
    fs::write(&path, source).unwrap();
    let uri = Url::from_file_path(path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    state.update_virtual_docs(&uri, source);
    let offset = source.find("<CharacterItem  />").unwrap() + "<CharacterItem ".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let items = match CompletionService::complete(&ctx).unwrap() {
        CompletionResponse::Array(items) => items,
        CompletionResponse::List(list) => list.items,
    };
    let actual: Vec<_> = items
        .iter()
        .enumerate()
        .map(|(rank, item)| RankedItem {
            label: item.label.as_str().into(),
            rank,
        })
        .collect();
    let mut expected = authority.predecessor_completion_items;
    expected.extend(
        authority
            .event_labels
            .iter()
            .enumerate()
            .map(|(index, label)| RankedItem {
                label: label.clone(),
                rank: 40 + index,
            }),
    );
    assert_eq!(actual, expected);
    assert_eq!(items.len(), 43);
    for item in &items[40..] {
        assert_eq!(item.kind, Some(CompletionItemKind::EVENT));
        assert_eq!(item.insert_text_format, Some(InsertTextFormat::SNIPPET));
        let expected = vize_l0::cstr!("{}=\"$1\"", item.label);
        assert_eq!(item.insert_text.as_deref(), Some(expected.as_str()));
    }
}
