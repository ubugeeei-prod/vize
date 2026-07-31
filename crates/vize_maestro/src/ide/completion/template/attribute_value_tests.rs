//! The attribute *value* position reached by typing `=`, before any quote.
//!
//! `=` is a completion trigger character (#3458), so this position must answer
//! with something: the reference server opens the value list there, and a
//! trigger that can only ever open an empty list is worse than no trigger.

use std::fs;

use tower_lsp::lsp_types::{CompletionResponse, Url};

use crate::ide::{CompletionService, IdeContext};
use crate::server::ServerState;

const SOURCE: &str = r#"<script setup lang="ts">
const count = 1
const label = 'x'
function pick(n: number) { return n }
</script>
<template>
  <input type= class= />
  <button :title= @click= v-if= type= class= />
</template>
"#;

fn labels_after(marker: &str) -> Vec<String> {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("AttributeValue.vue");
    fs::write(&source_path, SOURCE).unwrap();

    let uri = Url::from_file_path(&source_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), SOURCE.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, SOURCE);

    let offset = SOURCE.find(marker).unwrap() + marker.len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let Some(CompletionResponse::Array(items)) = CompletionService::complete(&ctx) else {
        return Vec::new();
    };
    let mut labels: Vec<String> = items.into_iter().map(|item| item.label).collect();
    labels.sort();
    labels
}

#[test]
fn a_bound_attribute_offers_the_template_bindings_right_after_the_equals() {
    // Every binding in scope, and nothing else: no directive names, because
    // the attribute-name list is over once `=` has been typed.
    for marker in [":title=", "@click=", "v-if="] {
        assert_eq!(
            labels_after(marker),
            vec!["count".to_string(), "label".to_string(), "pick".to_string(),],
            "`{marker}` must offer the template bindings"
        );
    }
}

#[test]
fn plain_html_type_values_are_specific_to_the_element() {
    assert_eq!(
        labels_after("<button :title= @click= v-if= type="),
        ["button", "reset", "submit"].map(str::to_string),
    );
    assert_eq!(
        labels_after("<input type="),
        [
            "button",
            "checkbox",
            "color",
            "date",
            "datetime",
            "datetime-local",
            "email",
            "file",
            "hidden",
            "image",
            "month",
            "number",
            "password",
            "radio",
            "range",
            "reset",
            "search",
            "submit",
            "tel",
            "text",
            "time",
            "url",
            "week",
        ]
        .map(str::to_string),
    );
}

#[test]
fn literal_valued_attributes_never_offer_template_bindings() {
    for marker in [
        "<input type=",
        "<button :title= @click= v-if= type=",
        "class=",
    ] {
        let labels = labels_after(marker);
        for binding in ["count", "label", "pick"] {
            assert!(
                !labels.iter().any(|label| label == binding),
                "{marker}: {labels:?}"
            );
        }
    }

    assert_eq!(labels_after("class="), Vec::<String>::new());
}
