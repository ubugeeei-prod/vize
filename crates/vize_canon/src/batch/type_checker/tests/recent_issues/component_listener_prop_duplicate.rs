//! A required component prop is checked once when the tag also has a listener.
//!
//! The emit-props inference literal is a second check of the same binding. Its
//! public input widens `boolean` to `boolean | undefined` (#10397). That copy
//! is inference-only; the canonical prop call owns the diagnostic.

use std::path::Path;

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn prop_binding_reported_twice_reports_once() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "component-listener-prop-duplicate",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ checked: boolean }>();
</script>
<template><input :checked="checked" /></template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Child from "./Child.vue";
function onChange(_value: boolean) {}
</script>
<template>
  <Child :checked="1" />
  <Child :checked="1" @change="onChange" />
  <Child :checked="true" @change="onChange" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let snapshot = snapshot.expect("tsgo diagnostics");

    let app: Vec<_> = snapshot
        .iter()
        .filter(|(file, _, _)| file.replace('\\', "/") == "src/App.vue")
        .map(|(_, code, message)| {
            let mut parts = message.split(':');
            let line = parts.next().unwrap_or("");
            (line, *code, prop_error_kind(message), message.as_str())
        })
        .collect();
    let rows = |line: &str| -> Vec<_> {
        app.iter()
            .filter(|(row_line, _, _, _)| *row_line == line)
            .map(|(_, code, kind, _)| (*code, *kind))
            .collect()
    };
    assert_eq!(
        rows("6"),
        vec![(Some(2322), "declared")],
        "a prop with no listener reports the declared type once: {snapshot:#?}"
    );
    assert_eq!(
        rows("7"),
        vec![(Some(2322), "declared")],
        "a listener must not add a widened twin: {snapshot:#?}"
    );
    assert_eq!(
        rows("8"),
        Vec::<(Option<u32>, &str)>::new(),
        "a valid prop next to a listener stays clean: {snapshot:#?}"
    );
}

#[test]
fn prop_binding_reported_twice_ignores_inference_literal() {
    let source = r#"<script setup lang="ts">
import Child from "./Child.vue";
function onChange(_value: boolean) {}
</script>
<template>
  <Child :checked="1" @change="onChange" />
</template>
"#;
    let result = crate::batch::generate_vue_content_mapper_transform(Path::new("App.vue"), source)
        .expect("transform");
    let inference = result
        .text
        .find("const __vize_emit_props_")
        .expect("listener inference");
    let probe = result.text[inference..]
        .split_once("});")
        .map(|(probe, _)| probe)
        .expect("inference literal");
    let ignore_lines: Vec<_> = probe
        .lines()
        .filter(|line| line.contains("@ts-ignore"))
        .map(str::trim)
        .collect();
    assert_eq!(
        ignore_lines,
        vec![
            "// @ts-ignore Inference-only prop; the canonical component-props call owns diagnostics."
        ],
        "{probe}"
    );
    let checked_lines: Vec<_> = probe
        .lines()
        .filter(|line| line.contains("\"checked\""))
        .map(str::trim)
        .collect();
    assert_eq!(checked_lines, vec!["\"checked\": 1,"], "{probe}");
}

fn prop_error_kind(message: &str) -> &'static str {
    if message.contains("boolean | undefined") {
        "widened"
    } else if message.contains("not assignable to type 'boolean'") {
        "declared"
    } else {
        "other"
    }
}
