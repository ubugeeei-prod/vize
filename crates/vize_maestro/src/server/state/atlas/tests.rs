use super::*;
use vize_atlas::ProductStatus;

#[test]
fn open_update_query_and_close_preserve_one_source_identity() {
    let state = ServerState::new();
    let uri = Url::parse("file:///tmp/App.tsx").unwrap();
    let source = state
        .upsert_artifact_source(&uri, "const App = () => <div>{one}</div>")
        .unwrap();

    let first = state
        .artifact_compilation
        .write()
        .query::<vize_atelier_jsx::JsxSyntaxProduct>(source)
        .unwrap();
    assert_eq!(first.status(), ProductStatus::Executed);
    let cached = state
        .artifact_compilation
        .write()
        .query::<vize_atelier_jsx::JsxSyntaxProduct>(source)
        .unwrap();
    assert_eq!(cached.status(), ProductStatus::CacheHit);

    assert_eq!(
        state.upsert_artifact_source(&uri, "const App = () => <div>{two}</div>"),
        Some(source)
    );
    let revised = state.jsx_syntax(&uri).unwrap();
    assert!(revised.source.contains("two"));

    state.remove_artifact_source(&uri);
    assert!(state.jsx_syntax(&uri).is_none());
}

#[test]
fn production_sfc_request_projections_have_zero_compatibility_bypasses() {
    crate::ide::reset_sfc_compatibility_queries();
    let state = ServerState::new();
    let uri = Url::parse("file:///tmp/App.vue").unwrap();
    let content = r#"<script setup>const count = 1</script>
<template><button>{{ count }}</button></template>
<style scoped>.button { color: red }</style>"#;
    let source = state.upsert_artifact_source(&uri, content).unwrap();
    state.update_virtual_docs(&uri, content);
    let context = crate::ide::IdeContext::with_content(&state, &uri, 50, content.to_string());
    let descriptor = context.sfc_descriptor().unwrap();
    crate::ide::SfcDocumentStructureService::symbols(descriptor);
    crate::ide::SfcDocumentStructureService::folding_ranges(descriptor);
    crate::ide::CodeLensService::get_lenses_from_descriptor(descriptor);
    crate::ide::DocumentLinkService::get_links_from_descriptor(content, &uri, descriptor);
    crate::ide::SemanticTokensService::get_tokens_from_descriptor(content, &uri, descriptor);
    assert!(context.sfc_croquis().is_some());

    assert_eq!(crate::ide::sfc_compatibility_queries(), 0);
    assert_eq!(
        state
            .artifact_compilation
            .read()
            .counters()
            .for_product::<vize_atelier_sfc::SfcDescriptorProduct>()
            .executions(),
        1
    );
    assert_eq!(
        state.artifact_sources.get(&uri).map(|entry| *entry),
        Some(source)
    );
}

#[test]
fn type_diagnostics_share_one_frontend_artifact_plan() {
    let state = ServerState::new();
    let uri = Url::parse("file:///tmp/Typed.vue").unwrap();
    let content = r#"<script setup lang="ts">
const count = 1
</script><template>{{ count }} {{ missing }}</template>"#;
    let request = vize_canon::SfcTypeCheckRequest::new(
        vize_canon::SfcTypeCheckOptions::new(uri.path()),
        vize_atelier_sfc::SfcCroquisMode::Full,
    );
    let result = state
        .sfc_typecheck_for(&uri, content, request.clone())
        .unwrap();
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code.as_deref() == Some("undefined-binding") })
    );
    let source = *state.artifact_sources.get(&uri).unwrap();
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
    drop(compilation);
    assert!(state.sfc_typecheck_for(&uri, content, request).is_some());
    assert_eq!(
        state
            .artifact_compilation
            .read()
            .counters()
            .for_product::<vize_canon::SfcTypeCheckProduct>()
            .cache_hits(),
        1
    );
    assert_eq!(
        state.artifact_sources.get(&uri).map(|entry| *entry),
        Some(source)
    );
}

#[test]
fn croquis_mode_refresh_invalidates_only_dependent_open_source_products() {
    let state = ServerState::new();
    let vue_uri = Url::parse("file:///tmp/App.vue").unwrap();
    let jsx_uri = Url::parse("file:///tmp/Widget.tsx").unwrap();
    let vue = state
        .upsert_artifact_source(
            &vue_uri,
            "<script>export default { data: () => ({ msg: 'hi' }) }</script><template>{{ msg }}</template>",
        )
        .unwrap();
    let jsx = state
        .upsert_artifact_source(&jsx_uri, "const Widget = () => <p>stable</p>")
        .unwrap();

    let mut compilation = state.artifact_compilation.write();
    assert_eq!(
        compilation
            .source_input::<vize_atelier_sfc::SfcCroquisSettingsInput>(vue)
            .unwrap()
            .mode,
        vize_atelier_sfc::SfcCroquisMode::OptionsApi
    );
    assert_eq!(
        compilation
            .query::<vize_croquis::CroquisDocumentProduct>(vue)
            .unwrap()
            .status(),
        ProductStatus::Executed
    );
    assert_eq!(
        compilation
            .query::<vize_atelier_jsx::JsxSyntaxProduct>(jsx)
            .unwrap()
            .status(),
        ProductStatus::Executed
    );
    drop(compilation);

    *state.type_checker_options_api.write() = false;
    state.refresh_artifact_croquis_mode();

    let mut compilation = state.artifact_compilation.write();
    assert_eq!(
        compilation
            .source_input::<vize_atelier_sfc::SfcCroquisSettingsInput>(vue)
            .unwrap()
            .mode,
        vize_atelier_sfc::SfcCroquisMode::Full
    );
    assert_eq!(
        compilation
            .query::<vize_croquis::CroquisDocumentProduct>(vue)
            .unwrap()
            .status(),
        ProductStatus::Executed
    );
    assert_eq!(
        compilation
            .query::<vize_atelier_jsx::JsxSyntaxProduct>(jsx)
            .unwrap()
            .status(),
        ProductStatus::CacheHit
    );
}
