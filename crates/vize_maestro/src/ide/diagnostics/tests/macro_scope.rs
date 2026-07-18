use super::{DiagnosticService, DiagnosticSeverity, Url, sources, state_with_lsp_diagnostics};

#[test]
fn collect_reports_hoisted_macro_local_scope_reference_for_lsp() {
    let state = state_with_lsp_diagnostics(false, false);
    let uri = Url::parse("file:///MacroScope.vue").unwrap();
    let source = r#"<script setup lang="ts">
const items = []

withDefaults(defineProps<{
  items?: string[]
}>(), { items })
</script>

<template>{{ items.join() }}</template>
"#;
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());

    let diagnostics = DiagnosticService::collect(&state, &uri);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("SCRIPT_SETUP_MACRO_SCOPE"))
        .expect("LSP should surface the SFC macro scope diagnostic");
    assert_eq!(diagnostic.source.as_deref(), Some(sources::SFC_COMPILER));
    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(
        diagnostic.range,
        tower_lsp::lsp_types::Range {
            start: tower_lsp::lsp_types::Position {
                line: 5,
                character: 8,
            },
            end: tower_lsp::lsp_types::Position {
                line: 5,
                character: 13,
            },
        }
    );
    assert_eq!(&source.lines().nth(5).unwrap()[8..13], "items");
}
