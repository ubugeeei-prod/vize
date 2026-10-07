use tower_lsp::lsp_types::{CompletionItem, CompletionResponse, Url};

use crate::ide::{CompletionService, IdeContext};
use crate::server::ServerState;

const ORIGINAL: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/template-expression-globals-8015/Comp.vue.txt"
);
const EXPECTED: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/template-expression-globals-8015/globals.expected.json"
);

fn expected() -> Vec<CompletionItem> {
    serde_json::from_str(EXPECTED).unwrap()
}

fn complete(source: &str, marker: &str) -> Vec<CompletionItem> {
    let state = ServerState::new();
    let uri = Url::parse("file:///TemplateGlobals.vue").unwrap();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);
    let offset = source.find(marker).unwrap() + marker.len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    match CompletionService::complete(&ctx) {
        Some(CompletionResponse::Array(items)) => items,
        Some(CompletionResponse::List(list)) => list.items,
        None => Vec::new(),
    }
}

fn assert_global_bank(items: &[CompletionItem]) {
    let bank = expected();
    let actual: Vec<_> = items
        .iter()
        .filter(|item| bank.iter().any(|global| global.label == item.label))
        .cloned()
        .collect();
    assert_eq!(actual, bank);
    for excluded in ["window", "document", "require", "Promise", "_ctx", "_cache"] {
        assert!(
            !items.iter().any(|item| item.label == excluded),
            "{excluded}: {items:?}"
        );
    }
}

#[test]
fn original_event_and_interpolation_positions_offer_complete_vue_global_bank() {
    for marker in ["$e", "{{ St", "{{ Ma"] {
        let items = complete(ORIGINAL, marker);
        assert_global_bank(&items);
        for local in ["Child", "emit", "value"] {
            assert_eq!(items.iter().filter(|item| item.label == local).count(), 1);
        }
        let event_count = items.iter().filter(|item| item.label == "$event").count();
        assert_eq!(event_count, usize::from(marker == "$e"));
    }
}

#[test]
fn globals_are_available_without_setup_and_at_unquoted_directive_values() {
    for (source, marker) in [
        ("<template><p>{{ St }}</p></template>", "{{ St"),
        ("<template><button @click= /></template>", "@click="),
        ("<template><p :title= /></template>", ":title="),
    ] {
        assert_global_bank(&complete(source, marker));
    }
}

#[test]
fn authored_script_props_and_loop_aliases_keep_their_whole_completion_authority() {
    let source = "<script setup lang=\"ts\">\nconst String = 'local'\ndefineProps<{ Math: number; rows: string[] }>()\n</script>\n<template><p v-for=\"Array in rows\">{{ St }}</p></template>";
    let items = complete(source, "{{ St");
    let globals = expected();
    for name in ["String", "Math", "Array"] {
        let candidates: Vec<_> = items.iter().filter(|item| item.label == name).collect();
        assert_eq!(candidates.len(), 1, "{name}: {items:?}");
        assert_ne!(
            candidates[0],
            globals.iter().find(|item| item.label == name).unwrap()
        );
    }
    assert_eq!(
        items
            .iter()
            .find(|item| item.label == "Math")
            .unwrap()
            .detail
            .as_deref(),
        Some("prop: number")
    );
    assert_eq!(
        items
            .iter()
            .find(|item| item.label == "Array")
            .unwrap()
            .detail
            .as_deref(),
        Some("Local v-for binding")
    );
    assert_global_bank(&complete("<template><p>{{ St }}</p></template>", "{{ St"));
}

#[test]
fn expression_globals_do_not_leak_into_html_comments_plain_values_or_member_access() {
    let bank = expected();
    for (source, marker) in [
        ("<template><p title=\"St\" /></template>", "title=\"St"),
        ("<template><p cl /></template>", "<p cl"),
        ("<template><!-- St --></template>", "<!-- St"),
        ("<template><p>{{ Math. }}</p></template>", "Math."),
        (
            "<script setup lang=\"ts\">const value = 1; St</script><template />",
            "; St",
        ),
    ] {
        let items = complete(source, marker);
        assert!(
            !items.iter().any(|item| bank.contains(item)),
            "{marker}: {items:?}"
        );
    }
}
