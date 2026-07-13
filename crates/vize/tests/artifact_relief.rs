use vize::artifact_graph::{VizeGraphConfig, create_compilation};
use vize::atelier_sfc::SfcTemplateProduct;
use vize::flow::FlowProduct;
use vize::relief::{ErrorCode, ReliefProduct, TransformedReliefProduct};
use vize::rendu::RenduProduct;
use vize_croquis::CroquisSemanticProduct;

#[test]
fn relief_syntax_is_cached_before_transform_and_keeps_parse_diagnostics() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source = compilation
        .add_source(
            "App.vue",
            r#"<template><div id="first" id="second" v-if="ready" /></template>"#,
        )
        .unwrap();

    let syntax = compilation.query::<ReliefProduct>(source).unwrap();
    let syntax = syntax.value().as_ref().expect("template syntax");
    assert!(!syntax.snapshot().transformed());
    assert!(
        syntax
            .parse_diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code == ErrorCode::DuplicateAttribute)
    );
    assert!(
        !compilation
            .cache()
            .contains::<TransformedReliefProduct>(source)
    );

    let transformed = compilation
        .query::<TransformedReliefProduct>(source)
        .unwrap();
    assert!(
        transformed
            .value()
            .as_ref()
            .expect("transformed template")
            .snapshot()
            .transformed()
    );
    assert_eq!(
        compilation
            .counters()
            .for_product::<ReliefProduct>()
            .executions(),
        1
    );
}

#[test]
fn fatal_parse_diagnostics_are_cached_before_consumers_reject_them() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source = compilation
        .add_source("Broken.vue", "<template><div></template>")
        .unwrap();

    let syntax = compilation.query::<ReliefProduct>(source).unwrap();
    let syntax = syntax.value().as_ref().expect("recovered template syntax");
    assert!(syntax.has_fatal_diagnostics());
    assert!(
        syntax
            .parse_diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code == ErrorCode::MissingEndTag)
    );
    assert!(
        !compilation
            .cache()
            .contains::<TransformedReliefProduct>(source)
    );
    let transformed = compilation
        .query::<TransformedReliefProduct>(source)
        .unwrap();
    let transformed = transformed.value().as_ref().expect("recovered transform");
    assert!(transformed.has_fatal_diagnostics());
    assert!(
        transformed
            .parse_diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code == ErrorCode::MissingEndTag)
    );
    assert!(compilation.query::<RenduProduct>(source).is_err());
    assert!(compilation.query::<RenduProduct>(source).is_err());
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
            .for_product::<TransformedReliefProduct>()
            .executions(),
        1
    );
}

#[test]
fn transform_diagnostics_are_cached_before_consumers_reject_them() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source = compilation
        .add_source("Broken.vue", "<template><div v-if>always</div></template>")
        .unwrap();

    let transformed = compilation
        .query::<TransformedReliefProduct>(source)
        .unwrap();
    let transformed = transformed.value().as_ref().expect("transformed template");
    assert!(transformed.has_fatal_diagnostics());
    assert!(
        transformed
            .transform_diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code == ErrorCode::VIfNoExpression)
    );
    assert!(compilation.query::<RenduProduct>(source).is_err());
    assert!(compilation.query::<RenduProduct>(source).is_err());
    assert_eq!(
        compilation
            .counters()
            .for_product::<TransformedReliefProduct>()
            .executions(),
        1
    );
}

#[test]
fn template_less_sfc_has_optional_syntax_empty_render_and_script_flow() {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source = compilation
        .add_source(
            "ScriptOnly.vue",
            "<script setup lang=\"ts\">const ready = true</script>",
        )
        .unwrap();

    assert!(
        compilation
            .query::<SfcTemplateProduct>(source)
            .unwrap()
            .value()
            .is_none()
    );
    assert!(
        compilation
            .query::<ReliefProduct>(source)
            .unwrap()
            .value()
            .is_none()
    );
    assert!(
        compilation
            .query::<TransformedReliefProduct>(source)
            .unwrap()
            .value()
            .is_none()
    );

    let rendu = compilation.query::<RenduProduct>(source).unwrap();
    assert!(rendu.value().sources().is_empty());
    assert!(rendu.value().nodes().is_empty());
    assert!(rendu.value().entry().is_empty());

    let flow = compilation.query::<FlowProduct>(source).unwrap();
    assert!(flow.value().blocks().len() > 1);
    assert!(
        flow.value()
            .sources()
            .any(|source| source.name().ends_with("#script-setup"))
    );
    flow.value().validate().unwrap();
    compilation.query::<CroquisSemanticProduct>(source).unwrap();
}
