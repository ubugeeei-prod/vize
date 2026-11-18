//! An explicit `useTemplateRef` import from `#imports` (#13319).

use serde_json::{Value, json};
use vize_l0::path::canonicalize_non_verbatim;

use super::test_support::{exchange, frames, initialize_request, open_project_request};

const SOURCE: &str = r##"<script setup lang="ts">
import { useTemplateRef } from "#imports"

const form = useTemplateRef("form")
</script>

<template>
  <form ref="form" />
</template>
"##;

fn open_and_transform(imports: &str) -> (Value, vize_l0::String) {
    let directory = tempfile::tempdir().unwrap();
    let declaration = directory.path().join("imports.d.ts");
    std::fs::write(&declaration, imports).unwrap();
    let config = directory.path().join("tsconfig.json");
    std::fs::write(
        &config,
        r##"{"compilerOptions":{"paths":{"#imports":["./imports"]}}}"##,
    )
    .unwrap();
    let mut open = open_project_request(2, Value::Null, json!({}));
    open["params"]["configFileName"] = json!(config);
    let transform = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "transform",
        "params": {
            "fileName": directory.path().join("Form.vue"),
            "content": SOURCE,
            "projectHandle": open["params"]["projectHandle"],
        }
    });
    let responses = exchange(&frames(&[initialize_request(), open, transform]));
    let watched = responses[1]["result"]["watchedFiles"].clone();
    let declaration = canonicalize_non_verbatim(&declaration);
    assert!(
        watched
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path.as_str() == declaration.to_str()),
        "{watched}"
    );
    (
        responses[1]["result"].clone(),
        responses[2]["result"]["text"].as_str().unwrap().into(),
    )
}

/// The generated statement holding the authored `useTemplateRef` call.
fn form_statement(text: &str) -> Option<&str> {
    text.lines().find(|line| {
        line.trim_end().ends_with("useTemplateRef(\"form\")")
            || line.trim_end().ends_with("__vizeInferRef(\"form\")")
    })
}

fn identity_suffix(project: &Value) -> Option<&str> {
    project["configIdentity"]
        .as_str()
        .and_then(|identity| identity.split_once(":imports-vue-template-ref:"))
        .map(|(_, flag)| flag)
}

#[test]
fn imports_alias_reexporting_vue_infers_the_template_ref() {
    let (project, text) = open_and_transform("export { useTemplateRef } from 'vue'\n");
    assert_eq!(identity_suffix(&project), Some("1"));
    assert_eq!(
        form_statement(&text),
        Some("void (null! as $TemplateRefs[\"form\"]); const form = __vizeInferRef(\"form\")"),
        "{text}"
    );
}

#[test]
fn imports_alias_with_its_own_composable_keeps_its_contract() {
    let (project, text) =
        open_and_transform("export declare function useTemplateRef<T>(key: string): T\n");
    assert_eq!(identity_suffix(&project), None);
    assert_eq!(
        form_statement(&text),
        Some("const form = useTemplateRef(\"form\")"),
        "{text}"
    );
}
