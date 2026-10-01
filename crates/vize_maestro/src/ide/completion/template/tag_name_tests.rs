//! Tag completions contain whole names and replace only the authored name range.
use super::tag_names;
use crate::{ide::IdeContext, server::ServerState};
use tower_lsp::lsp_types::{CompletionItem, CompletionTextEdit, Url};

fn items(source: &str, needle: &str) -> Option<Vec<CompletionItem>> {
    let state = ServerState::new();
    let uri = Url::parse("file:///tag-completion/Parent.vue").expect("URI");
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    let ctx = IdeContext::new(
        &state,
        &uri,
        source.find(needle).expect("caret") + needle.len(),
    )
    .expect("context");
    tag_names::complete(&ctx, &[])
}

#[test]
fn imported_component_and_kebab_names_have_exact_edits() {
    let source = "<script setup lang=\"ts\">\nimport Child from './Child.vue'\nimport type { ChildType } from './types'\n</script>\n<template>😀 <Chi</template>";
    let output = items(source, "<Chi").expect("tag names");
    assert_eq!(
        output
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        ["Child"]
    );
    let edit = output[0].text_edit.as_ref().expect("edit");
    let CompletionTextEdit::Edit(edit) = edit else {
        panic!("plain edit");
    };
    assert_eq!(
        serde_json::to_value(edit).expect("edit JSON"),
        serde_json::json!({"range":{"start":{"line":4,"character":14},"end":{"line":4,"character":17}},"newText":"Child"})
    );
    let output = items(&source.replace("<Chi", "<chi"), "<chi").expect("kebab tag");
    assert_eq!(
        output
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        ["child"]
    );
}

#[test]
fn native_names_and_closing_tags_replace_the_entire_name() {
    let output = items("<template><div><sp</div></template>", "<sp").expect("native");
    assert_eq!(
        output
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        ["span"]
    );
    let output = items("<template><span></spaX></template>", "</spa").expect("closing");
    assert_eq!(
        output
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        ["span"]
    );
    let CompletionTextEdit::Edit(edit) = output[0].text_edit.as_ref().expect("edit") else {
        panic!("plain edit");
    };
    assert_eq!(
        serde_json::to_value(edit).expect("JSON"),
        serde_json::json!({"range":{"start":{"line":0,"character":18},"end":{"line":0,"character":22}},"newText":"span"})
    );
}

#[test]
fn options_registrations_use_public_component_names() {
    let source = "<script>\nimport Child from './Child.vue'\nexport default { components: { LocalCard: Child, 'local-alias': Child } }\n</script>\n<template><Local</template>";
    let output = items(source, "<Local").expect("registered component");
    assert_eq!(
        output
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        ["LocalCard"]
    );
}

#[test]
fn attribute_values_comments_and_expressions_do_not_use_tag_candidates() {
    for (source, needle) in [
        ("<template><Child prop=\"<Chi\" /></template>", "<Chi"),
        ("<template><!-- <Chi --></template>", "<Chi"),
        ("<template>{{ '<Chi' }}</template>", "<Chi"),
        ("<template><Child cla /></template>", "cla"),
    ] {
        assert_eq!(items(source, needle), None);
    }
}
