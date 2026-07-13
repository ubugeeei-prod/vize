use vize_atelier_sfc::{SfcCroquisMode, SfcDescriptorProduct, SfcScriptSyntaxProduct};
use vize_atlas::{Compilation, ObservationKind, ProductStatus};
use vize_croquis::CroquisDocumentProduct;
use vize_flow::FlowProduct;
use vize_module::ModuleSyntaxProduct;
use vize_relief::{ReliefProduct, TransformedReliefProduct};

use super::{
    SfcTypeCheckOptions, SfcTypeCheckProduct, SfcTypeCheckRequest, install_sfc_typecheck_request,
    register_sfc_typecheck_provider,
};

fn compilation(source: &str) -> (Compilation, vize_atlas::SourceId) {
    let mut compilation = Compilation::new();
    vize_atelier_sfc::register_atlas_providers(&mut compilation).unwrap();
    register_sfc_typecheck_provider(&mut compilation).unwrap();
    let id = compilation.add_source("Fixture.vue", source).unwrap();
    install_sfc_typecheck_request(
        &mut compilation,
        id,
        SfcTypeCheckRequest::new(
            SfcTypeCheckOptions::new("Fixture.vue").with_virtual_ts(),
            SfcCroquisMode::Full,
        ),
    )
    .unwrap();
    (compilation, id)
}

#[test]
fn production_plan_executes_each_shared_frontend_artifact_once() {
    let source = r#"<script setup lang="ts">
const count = 1
</script>
<template><button v-if="count">{{ count }}</button></template>"#;
    crate::virtual_ts::reset_authored_script_fallback_parse_invocations();
    let (mut compilation, source) = compilation(source);
    let first = compilation.query::<SfcTypeCheckProduct>(source).unwrap();

    assert!(first.plan().contains::<SfcDescriptorProduct>());
    assert!(first.plan().contains::<ReliefProduct>());
    assert!(first.plan().contains::<CroquisDocumentProduct>());
    assert!(first.plan().contains::<SfcScriptSyntaxProduct>());
    assert!(first.plan().contains::<ModuleSyntaxProduct>());
    assert!(!first.plan().contains::<TransformedReliefProduct>());
    assert!(!first.plan().contains::<FlowProduct>());
    assert!(first.value().virtual_ts.is_some());
    assert_eq!(
        first
            .execution()
            .observations()
            .iter()
            .filter(|observation| observation.kind() == ObservationKind::Fallback)
            .count(),
        0
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<SfcDescriptorProduct>()
            .executions(),
        1
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<SfcScriptSyntaxProduct>()
            .executions(),
        1
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<ModuleSyntaxProduct>()
            .executions(),
        1
    );
    assert_eq!(
        crate::virtual_ts::authored_script_fallback_parse_invocations(),
        0
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<ReliefProduct>()
            .executions(),
        1
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<CroquisDocumentProduct>()
            .executions(),
        1
    );

    let cached = compilation.query::<SfcTypeCheckProduct>(source).unwrap();
    assert_eq!(cached.status(), ProductStatus::CacheHit);
}

#[test]
fn production_runner_has_no_shadow_sfc_template_or_croquis_analysis() {
    let runner = include_str!("../runner.rs");
    let engine = include_str!("../engine.rs");
    let production = format!("{runner}\n{engine}");

    assert!(!production.contains("parse_sfc("));
    assert!(!production.contains("vize_armature::parse"));
    assert!(!production.contains("analyze_sfc_descriptor"));
}

#[test]
fn malformed_sources_remain_diagnostic_artifacts_without_render_or_flow_work() {
    let (mut compilation, source) = compilation("<template><div></div>");
    let outcome = compilation.query::<SfcTypeCheckProduct>(source).unwrap();

    assert_eq!(outcome.value().error_count, 1);
    assert_eq!(
        outcome.value().diagnostics[0].code.as_deref(),
        Some("parse-error")
    );
    assert!(outcome.value().virtual_ts.is_none());
    assert!(!outcome.trace().executed::<FlowProduct>());
    assert!(outcome.trace().executed::<SfcTypeCheckProduct>());
}

#[test]
fn malformed_template_keeps_canon_checks_without_requesting_flow() {
    let source = r#"<script setup>
const props = defineProps(['count'])
</script>
<template><div>{{ missing }}</template>"#;
    let (mut compilation, source) = compilation(source);
    let outcome = compilation.query::<SfcTypeCheckProduct>(source).unwrap();

    assert!(
        outcome
            .value()
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code.as_deref() == Some("template-parse-error") })
    );
    assert!(
        outcome
            .value()
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code.as_deref() == Some("untyped-prop") })
    );
    assert!(!outcome.trace().executed::<FlowProduct>());
}
