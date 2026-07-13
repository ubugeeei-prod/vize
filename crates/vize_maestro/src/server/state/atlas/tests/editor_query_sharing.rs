//! Editor-query integration tests for persistent frontend products.

use tower_lsp::lsp_types::Url;

use crate::server::ServerState;

#[test]
fn one_sfc_revision_shares_script_frontend_across_editor_queries() {
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(&serde_json::json!({
        "lint": true,
        "typecheck": false
    })));
    let uri = Url::parse("file:///tmp/SharedEditor.vue").unwrap();
    let content = r#"<script setup lang="ts">
const { count = 1 } = defineProps<{ count?: number }>()
</script>
<template><button>{{ count }}</button></template>"#;
    state
        .documents
        .open(uri.clone(), content.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, content);
    let source = *state.artifact_sources.get(&uri).unwrap();

    let _ = crate::ide::DiagnosticService::collect(&state, &uri);
    let offset = content.rfind("count").unwrap();
    let context = crate::ide::IdeContext::with_content(&state, &uri, offset, content.to_string());
    let _ = crate::ide::CompletionService::complete(&context);
    let _ = crate::ide::HoverService::hover(&context);
    let _ = crate::ide::DefinitionService::definition(&context);
    let _ = crate::ide::ReferencesService::references(&context, true);
    assert!(
        crate::ide::WorkspaceSymbolsService::search(&state, "count")
            .iter()
            .any(|symbol| symbol.name == "count")
    );
    assert!(state.virtual_documents(&uri).is_some());

    let compilation = state.artifact_compilation.read();
    assert_eq!(
        compilation
            .counters()
            .for_product::<vize_atelier_sfc::SfcScriptSyntaxProduct>()
            .executions(),
        1
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<vize_module::ModuleSyntaxProduct>()
            .executions(),
        1
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<vize_croquis::CroquisDocumentProduct>()
            .executions(),
        1
    );
    assert_eq!(
        state.artifact_sources.get(&uri).map(|entry| *entry),
        Some(source)
    );
}

#[cfg(feature = "glyph")]
#[test]
fn one_sfc_revision_shares_frontend_across_virtual_diagnostics_format_and_actions() {
    use tower_lsp::lsp_types::{Position, Range};

    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(&serde_json::json!({
        "lint": true,
        "typecheck": false
    })));
    let uri = Url::parse("file:///tmp/SharedBoundary.vue").unwrap();
    let content = r#"<script setup lang="ts">
const { count = 1 } = defineProps<{ count?: number }>()
</script>
<template><button>{{ count }}</button></template>"#;
    state
        .documents
        .open(uri.clone(), content.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, content);
    let source = *state.artifact_sources.get(&uri).unwrap();
    let _ = crate::ide::DiagnosticService::collect(&state, &uri);
    assert!(
        crate::server::format::format_document(
            &state,
            &uri,
            content,
            &vize_glyph::FormatOptions::default(),
        )
        .is_some()
    );
    let offset = content.rfind("count").unwrap();
    let context = crate::ide::IdeContext::with_content(&state, &uri, offset, content.to_string());
    let _ = crate::ide::CodeActionService::code_actions(
        &context,
        Range::new(Position::new(3, 22), Position::new(3, 27)),
    );

    let counters = state.artifact_compilation.read();
    assert_eq!(
        counters
            .counters()
            .for_product::<vize_atelier_sfc::SfcDescriptorProduct>()
            .executions(),
        1
    );
    assert_eq!(
        counters
            .counters()
            .for_product::<vize_atelier_sfc::SfcScriptSyntaxProduct>()
            .executions(),
        1
    );
    assert_eq!(
        counters
            .counters()
            .for_product::<vize_module::ModuleSyntaxProduct>()
            .executions(),
        1
    );
    assert_eq!(
        counters
            .counters()
            .for_product::<vize_relief::ReliefProduct>()
            .executions(),
        1
    );
    assert_eq!(
        counters
            .counters()
            .for_product::<vize_croquis::CroquisDocumentProduct>()
            .executions(),
        1
    );
    assert_eq!(
        counters
            .counters()
            .for_product::<vize_glyph::GlyphFormatProduct>()
            .executions(),
        1
    );
    assert_eq!(
        counters
            .counters()
            .for_product::<crate::virtual_code::VirtualDocumentsProduct>()
            .executions(),
        1
    );
    assert_eq!(
        counters
            .counters()
            .for_product::<vize_patina::PatinaDocumentReportProduct>()
            .executions(),
        1
    );
    assert_eq!(
        state.artifact_sources.get(&uri).map(|entry| *entry),
        Some(source)
    );
}

#[test]
fn normal_sfc_editor_paths_do_not_construct_compatibility_frontends() {
    for source in [
        include_str!("../../../../ide/completion/script.rs"),
        include_str!("../../../../ide/completion/script/member_access.rs"),
        include_str!("../../../../ide/completion/script/reactive_infer.rs"),
        include_str!("../../../../ide/hover/script.rs"),
        include_str!("../../../../ide/definition/template.rs"),
        include_str!("../../../../ide/workspace_symbols.rs"),
    ] {
        assert!(!source.contains("Drawer::"));
        assert!(!source.contains("ScriptCompileContext"));
    }
    let virtual_root = include_str!("../../../../virtual_code/artifact.rs");
    assert!(virtual_root.contains("CroquisDocumentProduct"));
    let compile_diagnostics = include_str!("../../../../ide/diagnostics/collectors/frontend.rs");
    assert!(compile_diagnostics.contains("state.sfc_script_syntax(uri)"));
    assert!(!compile_diagnostics.contains("validate_script_setup_semantics_located"));
    let format = include_str!("../../../format.rs");
    assert!(format.contains("formatted_sfc_for"));
    assert!(!format.contains("format_sfc("));
}

#[test]
fn one_standalone_html_revision_shares_raw_template_frontend_across_editor_queries() {
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(&serde_json::json!({
        "lint": true,
        "typecheck": false
    })));
    let uri = Url::parse("file:///tmp/SharedEditor.html").unwrap();
    let content = r#"<!doctype html>
<html>
<head><script src="https://unpkg.com/petite-vue" defer init></script></head>
<body>
  <div v-scope="{ count: 0 }">
    <Counter :count="count" />
    <button @click="count++">{{ count }}</button>
  </div>
</body>
</html>"#;
    state
        .documents
        .open(uri.clone(), content.to_string(), 1, "html".to_string());
    state.update_virtual_docs(&uri, content);
    let source = *state.artifact_sources.get(&uri).unwrap();
    state.refresh_artifact_croquis_mode();
    assert!(
        state
            .artifact_compilation
            .read()
            .source_input::<vize_atelier_sfc::SfcCroquisSettingsInput>(source)
            .is_none()
    );

    let _ = crate::ide::DiagnosticService::collect(&state, &uri);
    let expression_offset = content.rfind("{{ count").unwrap() + "{{ co".len();
    let expression =
        crate::ide::IdeContext::with_content(&state, &uri, expression_offset, content.to_string());
    assert!(crate::ide::CompletionService::complete(&expression).is_some());
    assert!(crate::ide::HoverService::hover(&expression).is_some());

    let component_offset = content.find("<Counter").unwrap() + "<Cou".len();
    let component =
        crate::ide::IdeContext::with_content(&state, &uri, component_offset, content.to_string());
    assert!(crate::ide::HoverService::hover(&component).is_some());
    assert!(state.get_virtual_docs(&uri).is_some());

    let compilation = state.artifact_compilation.read();
    assert_eq!(
        compilation
            .counters()
            .for_product::<vize_relief::ReliefProduct>()
            .executions(),
        1
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<vize_croquis::CroquisDocumentProduct>()
            .executions(),
        1
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<vize_patina::PatinaDocumentReportProduct>()
            .executions(),
        1
    );
    assert_eq!(
        state.artifact_sources.get(&uri).map(|entry| *entry),
        Some(source)
    );
}
