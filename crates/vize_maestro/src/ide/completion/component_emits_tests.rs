use std::fs;

use tower_lsp::lsp_types::{CompletionResponse, CompletionTextEdit, Url};
use vize_l0::String;

use super::CompletionService;
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn imported_component_emits_complete_in_empty_and_at_prefixed_attributes() {
    let dir = tempfile::tempdir().unwrap();
    let child_path = dir.path().join("Child.vue");
    fs::write(
        &child_path,
        "<script setup lang='ts'>defineEmits<{ select: [id: number] }>()</script>",
    )
    .unwrap();
    let source = "<script setup lang='ts'>\nimport Child from './Child.vue'\n</script>\n<template>\n  <Child  />\n  <Child @s />\n</template>";
    let parent_path = dir.path().join("Parent.vue");
    fs::write(&parent_path, source).unwrap();
    let uri = Url::from_file_path(&parent_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    for (needle, offset) in [
        ("<Child  />", "<Child ".len()),
        ("<Child @s />", "<Child @s".len()),
    ] {
        let offset = source.find(needle).unwrap() + offset;
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let items = match CompletionService::complete(&ctx).unwrap() {
            CompletionResponse::Array(items) => items,
            CompletionResponse::List(list) => list.items,
        };
        let matches: Vec<_> = items
            .iter()
            .filter(|item| item.label == "@select")
            .collect();
        assert_eq!(matches.len(), 1, "{needle}");
        let event = matches[0];
        if needle.contains("@s") {
            let Some(CompletionTextEdit::Edit(edit)) = &event.text_edit else {
                panic!("prefixed event completion must replace the partial token");
            };
            assert_eq!(edit.new_text, "@select=\"$1\"");
        } else {
            assert_eq!(event.insert_text.as_deref(), Some("@select=\"$1\""));
        }
    }
}

#[derive(serde::Deserialize)]
struct EventCorpus {
    cases: Vec<EventCase>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventCase {
    project_id: String,
    source: String,
    event_labels: Vec<String>,
}

#[test]
fn pinned_ecosystem_declarations_complete_only_their_authored_events() {
    let corpus: EventCorpus = serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/lsp-authored-component-event-contracts.json"
    ))
    .unwrap();
    assert_eq!(corpus.cases.len(), 6);
    for case in corpus.cases {
        assert_eq!(
            imported_declared_events(&case.source),
            case.event_labels,
            "{}",
            case.project_id
        );
    }
}

#[test]
fn event_like_strings_and_comments_do_not_publish_component_events() {
    let source = r#"<script setup lang="ts">
const note = "defineEmits(['phantom'])";
// defineModel('phantom')
/* defineEmits<{ phantom: [] }>() */
</script>"#;
    assert!(imported_declared_events(source).is_empty());
}

fn imported_declared_events(child: &str) -> Vec<String> {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("Child.vue"), child).unwrap();
    let source = "<script setup lang='ts'>\nimport Child from './Child.vue'\n</script>\n<template><Child  /></template>";
    let parent = dir.path().join("Parent.vue");
    fs::write(&parent, source).unwrap();
    let uri = Url::from_file_path(&parent).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);
    let offset = source.find("<Child  />").unwrap() + "<Child ".len();
    let context = IdeContext::new(&state, &uri, offset).unwrap();
    let items = match CompletionService::complete(&context).unwrap() {
        CompletionResponse::Array(items) => items,
        CompletionResponse::List(list) => list.items,
    };
    items
        .into_iter()
        .filter(|item| {
            item.sort_text
                .as_deref()
                .is_some_and(|text| text.starts_with("00-event-"))
        })
        .map(|item| {
            assert_eq!(
                item.kind,
                Some(tower_lsp::lsp_types::CompletionItemKind::EVENT)
            );
            let expected = vize_l0::cstr!("{}=\"$1\"", item.label);
            assert_eq!(item.insert_text.as_deref(), Some(expected.as_str()));
            item.label.into()
        })
        .collect()
}
