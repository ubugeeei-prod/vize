//! Canon/Corsa regressions over Maestro's persistent editor compilation.

use tower_lsp::lsp_types::Url;

use crate::server::ServerState;

#[test]
fn corsa_document_reuses_the_lint_and_virtual_document_frontend() {
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(&serde_json::json!({
        "lint": true,
        "typecheck": false
    })));
    let uri = Url::parse("file:///tmp/SharedCorsa.vue").unwrap();
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
    let generated = state
        .canon_vue_document_for(
            &uri,
            content,
            vize_canon::CorsaVueVirtualDocumentOptions::default(),
        )
        .expect("persistent Canon virtual document");
    assert!(generated.code.contains("count"));

    let compilation = state.artifact_compilation.read();
    assert_eq!(
        compilation
            .counters()
            .for_product::<vize_atelier_sfc::SfcDescriptorProduct>()
            .executions(),
        1
    );
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
            .for_product::<vize_canon::batch::CanonVueDocumentProduct>()
            .executions(),
        1
    );
    assert_eq!(
        state.artifact_sources.get(&uri).map(|entry| *entry),
        Some(source)
    );
}

#[test]
fn normal_vue_corsa_routes_only_sync_the_persistent_product() {
    let diagnostics = include_str!("../../../../ide/diagnostics/corsa/collect.rs");
    let canonical = include_str!("../../../../ide/corsa_support/canonical.rs");
    assert!(diagnostics.contains("canon_vue_document_for"));
    assert!(diagnostics.contains("open_prebuilt_vue_virtual_document_with_overlays"));
    assert!(!diagnostics.contains(".open_vue_virtual_document_with_overlays("));
    assert!(canonical.contains("canon_vue_document_for"));
    assert!(canonical.contains("open_prebuilt_vue_virtual_document_with_overlays"));
    assert!(!canonical.contains(".open_vue_virtual_document("));
    assert!(!canonical.contains(".open_vue_virtual_document_with_overlays("));

    let provider =
        include_str!("../../../../../../vize_canon/src/batch/virtual_project/document_product.rs");
    assert!(!provider.contains("Compilation::new"));
    for product in [
        "SfcDescriptorProduct",
        "ReliefProduct",
        "CroquisDocumentProduct",
        "ModuleSyntaxProduct",
        "SfcScriptSyntaxProduct",
    ] {
        assert!(
            provider.contains(product),
            "missing shared product {product}"
        );
    }
}

#[test]
fn nested_script_vue_dependencies_stay_in_the_persistent_compilation() {
    let project = tempfile::TempDir::new().expect("temp project");
    let host_path = project.path().join("Host.vue");
    let helper_path = project.path().join("helper.ts");
    let dependency_path = project.path().join("Dependency.vue");
    let host = r#"<script setup lang="ts">
import { Dependency } from "./helper"
const component = Dependency
</script>
<template><component :is="component" /></template>"#;
    std::fs::write(&host_path, host).expect("host");
    std::fs::write(
        &helper_path,
        "import Dependency from './Dependency.vue'; export { Dependency };\n",
    )
    .expect("helper");
    std::fs::write(
        &dependency_path,
        r#"<script setup lang="ts">defineProps<{ label?: string }>()</script>
<template><span /></template>"#,
    )
    .expect("dependency");

    let state = ServerState::new();
    let uri = Url::from_file_path(&host_path).unwrap();
    state
        .documents
        .open(uri.clone(), host.to_string(), 1, "vue".to_string());
    let options = vize_canon::CorsaVueVirtualDocumentOptions::default();
    let host_document = state
        .canon_vue_document_for(&uri, host, options)
        .expect("host document");
    let first = state.canon_vue_overlays(&uri, &host_document, options);
    assert!(
        first.generated.iter().any(|(path, _)| {
            vize_carton::path::canonicalize_non_verbatim(path)
                == vize_carton::path::canonicalize_non_verbatim(&dependency_path)
        }),
        "nested Vue dependency must be supplied as a prebuilt overlay",
    );
    let dependency_uri = Url::from_file_path(vize_carton::path::canonicalize_non_verbatim(
        &dependency_path,
    ))
    .unwrap();
    let dependency_source = *state
        .artifact_sources
        .get(&dependency_uri)
        .expect("persistent dependency source");

    let second = state.canon_vue_overlays(&uri, &host_document, options);
    assert_eq!(second.generated.len(), first.generated.len());
    assert_eq!(
        state
            .artifact_sources
            .get(&dependency_uri)
            .map(|source| *source),
        Some(dependency_source),
    );
    let compilation = state.artifact_compilation.read();
    for executions in [
        compilation
            .counters()
            .for_product::<vize_atelier_sfc::SfcDescriptorProduct>()
            .executions(),
        compilation
            .counters()
            .for_product::<vize_atelier_sfc::SfcScriptSyntaxProduct>()
            .executions(),
        compilation
            .counters()
            .for_product::<vize_module::ModuleSyntaxProduct>()
            .executions(),
        compilation
            .counters()
            .for_product::<vize_relief::ReliefProduct>()
            .executions(),
        compilation
            .counters()
            .for_product::<vize_croquis::CroquisDocumentProduct>()
            .executions(),
        compilation
            .counters()
            .for_product::<vize_canon::batch::CanonVueDocumentProduct>()
            .executions(),
    ] {
        assert_eq!(
            executions, 2,
            "host and dependency should execute once each"
        );
    }
}
