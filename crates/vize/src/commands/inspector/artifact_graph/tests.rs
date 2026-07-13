use vize_atelier_sfc::{SfcCompileProduct, SfcDescriptorProduct};
use vize_atlas::{ObservationKind, ProductStatus};
use vize_carton::cstr;
use vize_rendu::RenduProduct;

use super::*;
use crate::commands::inspector::{InspectorOutputFormat, InspectorTemplateSyntaxArg};

fn args(target: InspectorTarget) -> InspectorArgs {
    InspectorArgs {
        target,
        format: InspectorOutputFormat::Compare,
        template_syntax: InspectorTemplateSyntaxArg::Standard,
        ..Default::default()
    }
}

fn file(source: &str) -> curator_inspector::InspectorSourceFile {
    curator_inspector::InspectorSourceFile {
        path: cstr!("src/App.vue"),
        source: source.into(),
    }
}

#[test]
fn production_query_executes_one_parse_then_uses_the_shared_backend_plan() {
    let file = file("<template><main>{{ msg }}</main></template>");
    let graph =
        InspectorArtifactGraph::new(std::slice::from_ref(&file), &args(InspectorTarget::Dom))
            .unwrap();
    let source = graph.sources[&file.path];
    let mut session = graph.snapshot.query_session();
    let descriptor = session.query::<SfcDescriptorProduct>(source).unwrap();
    let compiled = session.query::<SfcCompileProduct>(source).unwrap();

    assert_eq!(descriptor.status(), ProductStatus::Executed);
    assert_eq!(compiled.status(), ProductStatus::Executed);
    assert!(compiled.trace().cache_hit::<SfcDescriptorProduct>());
    assert!(!compiled.trace().executed::<SfcDescriptorProduct>());
    assert!(compiled.plan().contains::<RenduProduct>());
    assert_eq!(
        compiled
            .execution()
            .observations()
            .iter()
            .filter(|observation| observation.kind() == ObservationKind::Fallback)
            .count(),
        0
    );
}

#[test]
fn source_aware_graph_compile_preserves_typescript_configuration() {
    let source = "<script setup lang=\"ts\">const msg: string='hi'</script>\n<template><p>{{msg}}</p></template>";
    let file = file(source);
    let graph =
        InspectorArtifactGraph::new(std::slice::from_ref(&file), &args(InspectorTarget::Dom))
            .unwrap();
    let actual = graph.query(&file).unwrap();

    assert!(actual.plan().contains::<RenduProduct>());
    assert!(actual.value().code.contains("msg"));
    assert!(actual.value().errors.is_empty());
}

#[test]
fn graph_product_preserves_target_semantics_and_serialized_outputs() {
    let source = "<script setup lang=\"ts\">const msg: string='hi'</script>\n<template><p class=\"x\">{{msg}}</p></template>\n<style scoped>.x{color:red}</style>";
    let file = file(source);
    for (target, marker) in [
        (InspectorTarget::Dom, "export function render"),
        (InspectorTarget::Ssr, "export function ssrRender"),
        (InspectorTarget::Vapor, "_sfc_main.__vapor = true"),
    ] {
        let graph =
            InspectorArtifactGraph::new(std::slice::from_ref(&file), &args(target)).unwrap();
        let source_id = graph.sources[&file.path];
        let mut session = graph.snapshot.query_session();
        let descriptor = session.query::<SfcDescriptorProduct>(source_id).unwrap();
        let compiled = session.query::<SfcCompileProduct>(source_id).unwrap();
        let serialized = serde_json::to_value(compiled.value()).unwrap();

        assert_eq!(descriptor.status(), ProductStatus::Executed);
        assert!(compiled.trace().cache_hit::<SfcDescriptorProduct>());
        assert!(compiled.plan().contains::<RenduProduct>());
        assert_eq!(serialized["css"], ".x[data-v-996667b8]{color:red}");
        assert_eq!(serialized["errors"], serde_json::json!([]));
        assert_eq!(serialized["warnings"], serde_json::json!([]));
        assert_eq!(serialized["bindings"]["bindings"]["msg"], "literal-const");
        assert!(compiled.value().code.contains(marker));
        assert_eq!(
            compiled
                .execution()
                .observations()
                .iter()
                .filter(|observation| observation.kind() == ObservationKind::Fallback)
                .count(),
            0
        );
    }
}

#[test]
fn malformed_sfc_diagnostic_is_cached_without_a_compile_fallback() {
    let file = file("<template /><template />");
    let graph =
        InspectorArtifactGraph::new(std::slice::from_ref(&file), &args(InspectorTarget::Dom))
            .unwrap();
    let source = graph.sources[&file.path];
    let mut session = graph.snapshot.query_session();
    let first = session.query::<SfcDescriptorProduct>(source).unwrap();
    let second = session.query::<SfcDescriptorProduct>(source).unwrap();

    assert!(first.value().diagnostic().is_some());
    assert_eq!(second.status(), ProductStatus::CacheHit);
    assert!(session.query::<SfcCompileProduct>(source).is_err());
}

#[test]
fn script_lang_detection_handles_quotes_and_unquoted_values() {
    assert!(source_uses_type_script(
        "<script setup lang='tsx'>x</script>"
    ));
    assert!(source_uses_type_script("<script lang=ts>x</script>"));
    assert!(source_uses_type_script("<script lang = \"ts\">x</script>"));
    assert!(!source_uses_type_script("<script setup>x</script>"));
}
