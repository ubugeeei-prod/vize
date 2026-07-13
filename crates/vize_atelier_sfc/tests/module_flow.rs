use vize_atelier_sfc::register_atlas_providers;
use vize_atlas::Compilation;
use vize_flow::{ControlEdgeKind, FlowProduct, TerminatorKind};
use vize_module::ModuleSyntaxProduct;

#[test]
fn sfc_flow_combines_script_cfg_and_template_cfg() {
    let mut compilation = Compilation::new();
    register_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source(
            "Combined.vue",
            r#"<script setup lang="ts">
import Card from './Card.vue'
for (let i = 0; i < 2; i++) { if (i) continue }
</script>
<template><Card v-if="ok" v-for="item in items">{{ item }}</Card></template>"#,
        )
        .unwrap();
    let modules = compilation.query::<ModuleSyntaxProduct>(source).unwrap();
    assert_eq!(modules.value().modules.len(), 1);
    assert_eq!(
        modules.value().modules[0].imports[0].specifier.as_ref(),
        "./Card.vue"
    );
    let flow = compilation.query::<FlowProduct>(source).unwrap();
    let graph = flow.value();
    assert!(
        graph
            .sources()
            .any(|source| source.name().ends_with("#script-setup"))
    );
    assert!(
        graph
            .sources()
            .any(|source| source.name() == "sfc-template")
    );
    assert!(
        graph
            .control_edges()
            .any(|edge| edge.kind() == ControlEdgeKind::LoopBack)
    );
    assert!(graph.nodes().any(|node| {
        matches!(
            node.kind(),
            vize_flow::NodeKind::Terminator(TerminatorKind::Branch)
        )
    }));
}
