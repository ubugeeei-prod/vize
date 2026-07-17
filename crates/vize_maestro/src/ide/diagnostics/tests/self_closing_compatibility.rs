use super::{DiagnosticService, state_with_lsp_diagnostics};
use tower_lsp::lsp_types::Url;

#[test]
fn collect_silences_vue_compatible_self_closing_native_elements() {
    let state = state_with_lsp_diagnostics(false, false);
    let uri = Url::parse("file:///NotFound.vue").unwrap();
    let source = r#"<template>
  <div class="NotFound">
    <div class="divider" />
    <span>{{ message }}</span>
  </div>
</template>

<script setup lang="ts">
const message = "missing"
</script>
"#;
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());

    let diagnostics = DiagnosticService::collect(&state, &uri);

    assert!(
        diagnostics.iter().all(|diagnostic| {
            !diagnostic
                .message
                .starts_with("Invalid self-closing syntax on non-void HTML element")
        }),
        "Vue-compatible self-closing syntax should stay silent: {diagnostics:?}",
    );
}
