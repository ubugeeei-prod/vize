#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::disallowed_types
)]

use std::fs;

use super::{DiagnosticService, offset_to_line_col, sources};
use crate::server::ServerState;
use tower_lsp::lsp_types::{DiagnosticSeverity, NumberOrString, Url};

fn state_with_lsp_diagnostics(lint: bool, typecheck: bool) -> ServerState {
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(&serde_json::json!({
        "lint": lint,
        "typecheck": typecheck
    })));
    state
}

#[test]
fn reports_missing_required_component_props_on_template_tag() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("Child.vue"),
        r#"<script setup lang="ts">
defineProps<{
  someMessage: string
  count?: number
}>()
</script>
"#,
    )
    .unwrap();

    let source = r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child />
</template>
"#;
    let parent_path = dir.path().join("Parent.vue");
    fs::write(&parent_path, source).unwrap();

    let state = state_with_lsp_diagnostics(false, true);
    let uri = Url::from_file_path(&parent_path).unwrap();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    let diagnostics = DiagnosticService::collect(&state, &uri);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.source.as_deref() == Some(sources::COMPONENTS))
        .expect("expected a component required-props diagnostic");

    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(
        diagnostic.code,
        Some(NumberOrString::String(
            "component-required-props".to_string()
        ))
    );
    assert!(diagnostic.message.contains("<Child>"), "{diagnostic:?}");
    assert!(
        diagnostic.message.contains("`someMessage`"),
        "{diagnostic:?}"
    );
    assert!(
        diagnostic
            .code_description
            .as_ref()
            .is_some_and(|description| description.href.as_str().contains("components/props")),
        "{diagnostic:?}"
    );

    let tag_name_start = source.find("<Child").unwrap() + 1;
    let (line, character) = offset_to_line_col(source, tag_name_start);
    assert_eq!(diagnostic.range.start.line, line);
    assert_eq!(diagnostic.range.start.character, character);
}

#[test]
fn skips_required_component_prop_diagnostic_when_spread_attrs_are_present() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("Child.vue"),
        r#"<script setup lang="ts">
defineProps<{ someMessage: string }>()
</script>
"#,
    )
    .unwrap();

    let source = r#"<script setup lang="ts">
import Child from './Child.vue'
const attrs = {}
</script>

<template>
  <Child v-bind="attrs" />
</template>
"#;
    let parent_path = dir.path().join("Parent.vue");
    fs::write(&parent_path, source).unwrap();

    let state = state_with_lsp_diagnostics(false, true);
    let uri = Url::from_file_path(&parent_path).unwrap();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    let diagnostics = DiagnosticService::collect(&state, &uri);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.source.as_deref() != Some(sources::COMPONENTS)),
        "spread attrs may satisfy required props at runtime; got {diagnostics:#?}"
    );
}

#[test]
fn skips_required_component_prop_diagnostic_for_runtime_prop_name() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("Child.vue"),
        r#"<script setup lang="ts">
defineProps<{ someMessage: string }>()
</script>
"#,
    )
    .unwrap();

    let source = r#"<script setup lang="ts">
import Child from './Child.vue'
const propName = 'someMessage'
const value = 'hello'
</script>

<template>
  <Child :[propName]="value" />
</template>
"#;
    let parent_path = dir.path().join("Parent.vue");
    fs::write(&parent_path, source).unwrap();

    let state = state_with_lsp_diagnostics(false, true);
    let uri = Url::from_file_path(&parent_path).unwrap();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    let diagnostics = DiagnosticService::collect(&state, &uri);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.source.as_deref() != Some(sources::COMPONENTS)),
        "a runtime prop name may satisfy a required prop; got {diagnostics:#?}"
    );
}
