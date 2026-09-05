use std::collections::HashMap;

use tower_lsp::lsp_types::{Position, Range, TextEdit, Url, WorkspaceEdit};

use super::merge_missing_authored_rename;
use crate::{ide::IdeContext, server::ServerState};

const SOURCE: &str =
    "<script setup lang=\"ts\">\nconst total = 1\n</script>\n<template>{{ total }}</template>\n";

#[test]
fn appends_missing_authored_ranges_after_canonical_rename() {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Scenario.vue").unwrap();
    state
        .documents
        .open(uri.clone(), SOURCE.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, SOURCE);
    let ctx = IdeContext::new(&state, &uri, SOURCE.find("total").unwrap()).unwrap();

    let script_edit = edit(1, 6, 1, 11);
    let template_edit = edit(3, 13, 3, 18);
    let canonical = changes(&uri, vec![template_edit.clone()]);
    let authored = changes(&uri, vec![script_edit.clone(), template_edit.clone()]);

    let merged =
        merge_missing_authored_rename(&ctx, Some(canonical), Some(authored)).expect("merged edit");
    let merged = merged.changes.expect("merged changes");

    assert_eq!(merged[&uri], vec![script_edit, template_edit]);
}

#[test]
fn keeps_canonical_text_for_overlapping_authored_ranges() {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Scenario.vue").unwrap();
    state
        .documents
        .open(uri.clone(), SOURCE.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, SOURCE);
    let ctx = IdeContext::new(&state, &uri, SOURCE.find("total").unwrap()).unwrap();

    let mut canonical_edit = edit(1, 0, 1, 11);
    canonical_edit.new_text = "const quantity".to_string();
    let mut authored_edit = edit(1, 6, 1, 11);
    authored_edit.new_text = "renamed".to_string();

    let merged = merge_missing_authored_rename(
        &ctx,
        Some(changes(&uri, vec![canonical_edit.clone()])),
        Some(changes(&uri, vec![authored_edit])),
    )
    .expect("merged edit");
    let merged = merged.changes.expect("merged changes");

    assert_eq!(merged[&uri], vec![canonical_edit]);
}

#[test]
fn keeps_multi_range_canonical_rename_authoritative() {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Scenario.vue").unwrap();
    state
        .documents
        .open(uri.clone(), SOURCE.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, SOURCE);
    let ctx = IdeContext::new(&state, &uri, SOURCE.find("total").unwrap()).unwrap();

    let canonical = changes(&uri, vec![edit(1, 6, 1, 11), edit(3, 13, 3, 18)]);
    let authored = changes(
        &uri,
        vec![edit(1, 6, 1, 11), edit(3, 13, 3, 18), edit(3, 36, 3, 41)],
    );

    let merged = merge_missing_authored_rename(&ctx, Some(canonical.clone()), Some(authored))
        .expect("merged edit");

    assert_eq!(merged, canonical);
}

fn changes(uri: &Url, edits: Vec<TextEdit>) -> WorkspaceEdit {
    WorkspaceEdit {
        changes: Some(HashMap::from([(uri.clone(), edits)])),
        document_changes: None,
        change_annotations: None,
    }
}

fn edit(start_line: u32, start_character: u32, end_line: u32, end_character: u32) -> TextEdit {
    TextEdit {
        range: Range::new(
            Position::new(start_line, start_character),
            Position::new(end_line, end_character),
        ),
        new_text: "quantity".to_string(),
    }
}
