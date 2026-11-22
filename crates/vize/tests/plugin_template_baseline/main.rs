//! Original public SDK and Doctor contracts for a scoped Vue template.
use davinci_harness::alloc::{self, CountingAllocator};
use std::{alloc::System, hint::black_box};
use vize_croquis_cf::TemplateComplexity;
use vize_l0::fact::FactManager;
use vize_patina::plugin_batch::{
    batch::{PluginSpec, build_batch, diagnostics},
    document::PluginDocument,
    facts::REGISTRY,
};

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

#[test]
fn original_component_facts_plugin_scopes_and_report_positions_share_the_authored_file() {
    const SOURCE: &str = "<script setup>/* ok */</script>\r\n<template>\r\n  <ul title=\"🌱\" v-if=\"enabled\"><li v-for=\"(item, i) in rows\" :key=\"i\">{{ item }}</li></ul>\r\n</template>";
    let demands = ["templateScopes".into()];
    let mut spec = PluginSpec {
        name: "contract",
        version: "1",
        fingerprint: "fixture",
        visit: None,
        demands: &demands,
    };
    let read = || {
        let doc =
            PluginDocument::build(black_box(SOURCE), "Original.vue").expect("valid component");
        let batch =
            build_batch(&doc, &spec, &mut FactManager::new(&REGISTRY)).expect("public demand");
        let facts = TemplateComplexity::from_sfc(black_box(SOURCE)).expect("inline HTML");
        (doc, batch, facts)
    };
    alloc::mark_installed();
    let ((document, batch, facts), allocation) = alloc::measure_returning(read);
    let allocation = allocation.expect("global counting allocator");
    let loop_node = document
        .nodes
        .iter()
        .find(|n| n.kind == "ui.for")
        .expect("loop owner");
    assert_eq!((facts.own.cyclomatic, facts.own.cognitive), (3, 3));
    let loop_fact = facts
        .contributors
        .iter()
        .find(|r| r.kind == "v-for")
        .expect("loop contributor");
    assert_eq!(loop_fact.line, 3);
    let payload: serde_json::Value = serde_json::from_str(&batch.json).expect("SDK JSON");
    let reports = format!(
        r#"[{{"node":{},"rule":"loop","message":"original","fix":"<li/>"}}]"#,
        loop_node.id
    );
    let mapped = diagnostics(&document, spec.name, &reports).expect("mapped report");
    assert_eq!(
        (mapped[0].start, mapped[0].end),
        (loop_node.start, loop_node.end)
    );
    assert_eq!(
        (
            mapped[0].line,
            mapped[0].column,
            mapped[0].end_line,
            mapped[0].end_column
        ),
        (3, 33, 3, 87)
    );
    assert_eq!(
        payload["facts"],
        serde_json::json!({"templateScopes": [[loop_node.id, [
            {"name": "item", "position": "value"},
            {"name": "i", "position": "key"}
        ]]]})
    );
    assert_eq!(
        payload["parents"],
        serde_json::json!(
            document
                .nodes
                .iter()
                .map(|node| node.parent.map_or(-1, i64::from))
                .collect::<Vec<_>>()
        )
    );
    assert!(allocation.calls > 0 && allocation.peak_bytes_over_start > 0);
    eprintln!(
        "original_plugin_template nodes={} bytes={} contributors={} calls={} peak={}",
        batch.nodes,
        batch.json.len(),
        facts.contributors.len(),
        allocation.calls,
        allocation.peak_bytes_over_start
    );
    spec.demands = &[];
    let without =
        build_batch(&document, &spec, &mut FactManager::new(&REGISTRY)).expect("no demands");
    let mut expected = payload;
    expected["facts"] = serde_json::json!({});
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&without.json).expect("SDK JSON"),
        expected
    );
}
