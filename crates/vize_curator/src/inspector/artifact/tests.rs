use vize_atelier_sfc::SfcDescriptorProduct;
use vize_atlas::Compilation;
use vize_croquis::CroquisDocumentProduct;
use vize_module::ModuleSyntaxProduct;
use vize_relief::ReliefProduct;

use super::*;
use crate::inspector::{
    InspectorOptions, InspectorSourceFile, InspectorTarget, build_payload, serialize_agent_report,
};

const APP: &str = r#"<script setup lang="ts">
import ChildCard from './Child.vue'
</script>
<template><ChildCard /></template>"#;
const CHILD: &str =
    "<script setup>defineProps<{ title?: string }>()</script><template><p /></template>";

#[test]
fn agent_report_is_the_multi_source_root() {
    let files = vec![
        InspectorSourceFile {
            path: "src/App.vue".into(),
            source: APP.into(),
        },
        InspectorSourceFile {
            path: "src/Child.vue".into(),
            source: CHILD.into(),
        },
    ];
    let payload = build_payload(
        InspectorTarget::Dom,
        InspectorOptions {
            custom_renderer: false,
            template_syntax: Default::default(),
        },
        files.clone(),
    );
    let mut graph =
        InspectorReportGraph::new(payload, "https://example.test".into(), &files).unwrap();
    let outcome = graph.query().unwrap();

    assert!(outcome.plan().contains::<InspectorAgentReportProduct>());
    assert!(outcome.plan().contains::<InspectorSourceAnalysisProduct>());
    assert!(outcome.plan().contains::<SfcDescriptorProduct>());
    assert!(outcome.plan().contains::<ReliefProduct>());
    assert!(outcome.plan().contains::<CroquisDocumentProduct>());
    let json = serialize_agent_report(outcome.value()).unwrap();
    assert!(json.contains("\"kind\": \"component\""));
    assert!(json.contains("\"semanticFiles\""));
}

#[test]
fn source_analysis_does_not_execute_report_or_vue_products_for_ts() {
    let mut compilation = Compilation::new();
    register_inspector_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source("src/main.ts", "import './side-effect.js'")
        .unwrap();
    let outcome = compilation
        .query::<InspectorSourceAnalysisProduct>(source)
        .unwrap();

    assert!(!outcome.plan().contains::<InspectorAgentReportProduct>());
    assert!(!outcome.plan().contains::<SfcDescriptorProduct>());
    assert!(!outcome.plan().contains::<ReliefProduct>());
    assert_eq!(outcome.value().graph.imports.len(), 1);
}

#[test]
fn source_analysis_uses_the_jsx_frontend_for_tsx() {
    let mut compilation = Compilation::new();
    register_inspector_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source(
            "src/Card.tsx",
            "import Child from './Child.vue'; export const Card = () => <Child />;",
        )
        .unwrap();
    let outcome = compilation
        .query::<InspectorSourceAnalysisProduct>(source)
        .unwrap();

    assert!(
        outcome
            .plan()
            .contains::<vize_atelier_jsx::JsxSyntaxProduct>()
    );
    assert!(outcome.plan().contains::<ModuleSyntaxProduct>());
    assert!(outcome.plan().contains::<CroquisDocumentProduct>());
    assert!(!outcome.plan().contains::<SfcDescriptorProduct>());
    assert!(!outcome.plan().contains::<ReliefProduct>());
    assert_eq!(outcome.value().graph.imports.len(), 1);
    assert!(outcome.value().graph.template_used_ids.contains("Child"));
    assert!(outcome.value().semantic.is_some());
}

#[test]
fn source_analysis_accepts_virtual_source_suffixes() {
    for (name, source) in [
        (
            "src/App.vue?vue&type=script",
            "<template><main /></template>",
        ),
        (
            "src/Card.tsx#component",
            "type Props = { label: string }; export const Card = (props: Props) => <main>{props.label}</main>;",
        ),
    ] {
        let mut compilation = Compilation::new();
        register_inspector_atlas_providers(&mut compilation).unwrap();
        let source = compilation.add_source(name, source).unwrap();
        let outcome = compilation
            .query::<InspectorSourceAnalysisProduct>(source)
            .unwrap();

        assert!(outcome.value().semantic.is_some(), "{name}");
    }
}

#[test]
fn source_analysis_reports_recovered_tsx_parse_errors() {
    let mut compilation = Compilation::new();
    register_inspector_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source("src/Broken.tsx", "export const Broken = () => <main")
        .unwrap();
    let outcome = compilation
        .query::<InspectorSourceAnalysisProduct>(source)
        .unwrap();

    assert!(outcome.value().jsx_parse_error);
}

#[test]
fn production_inspector_sources_do_not_reparse_sfc_or_template() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/inspector");
    for file in ["payload.rs", "imports.rs", "artifact/source.rs"] {
        let source = std::fs::read_to_string(root.join(file)).unwrap();
        assert!(!source.contains("parse_sfc("), "{file} reparses the SFC");
        assert!(
            !source.contains("vize_armature::parse"),
            "{file} reparses the template"
        );
    }
}
