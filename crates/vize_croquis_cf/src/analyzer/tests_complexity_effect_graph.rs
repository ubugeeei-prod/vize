use super::{CrossFileAnalyzer, CrossFileOptions};
use serde_json::json;
use vize_carton::cstr;
use vize_croquis::{
    Analyzer, AnalyzerOptions, Croquis, EffectGraphSummary, build_effect_graph_from_script,
    build_effect_graph_from_script_setup,
};

const ALPHA_SOURCE: &str = r#"
import { computed, ref } from 'vue'
const count = ref(0)
const doubled = computed(() => count.value * 2)
"#;

const BETA_SOURCE: &str = r#"
import { computed, ref, watch, watchEffect } from 'vue'
const count = ref(0)
const doubled = computed(() => count.value * 2)
watchEffect(() => console.log(doubled.value))
watch(count, () => {})
"#;

#[test]
fn parser_effect_graphs_drive_global_and_hotspot_complexity() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());
    add_vue_script(&mut analyzer, "Alpha.vue", ALPHA_SOURCE);
    add_vue_script(&mut analyzer, "Beta.vue", BETA_SOURCE);

    let result = analyzer.analyze();
    let input = result.complexity_report.input;

    assert_eq!(input.reactive_node_count, 4);
    assert_eq!(input.reactive_edge_count, 4);
    assert_eq!(input.reactive_cycle_count, 0);
    assert_eq!(result.complexity_report.dimensions.reactive_graph, 12);
    assert_eq!(result.complexity_hotspots.len(), 2);
    assert_eq!(result.complexity_hotspots[0].file_name, "Beta.vue");
    assert_eq!(result.complexity_hotspots[1].file_name, "Alpha.vue");

    let hotspot_input = result
        .complexity_hotspots
        .iter()
        .fold((0, 0, 0), |counts, hotspot| {
            (
                counts.0 + hotspot.input.reactive_node_count,
                counts.1 + hotspot.input.reactive_edge_count,
                counts.2 + hotspot.input.reactive_cycle_count,
            )
        });
    assert_eq!(hotspot_input, (4, 4, 0));

    assert_eq!(
        serde_json::to_value(&result.complexity_hotspots).unwrap(),
        json!([
            {
                "fileId": 1,
                "fileName": "Beta.vue",
                "componentName": "Beta",
                "input": {
                    "componentCount": 1,
                    "templateIfCount": 0,
                    "templateForCount": 0,
                    "templateLogicalOperatorCount": 0,
                    "componentTreeVIfMaxDepth": 0,
                    "componentTreeVForMaxDepth": 0,
                    "componentTreeScopedSlotMaxDepth": 0,
                    "componentTreeTemplateNestingScore": 0,
                    "slotCount": 0,
                    "propDrillingEdgeCount": 0,
                    "globalStateReferenceCount": 0,
                    "provideInjectMaxDepth": 0,
                    "provideInjectReferenceCount": 0,
                    "provideInjectFanoutCount": 0,
                    "fallthroughRiskCount": 0,
                    "reactiveNodeCount": 2,
                    "reactiveEdgeCount": 3,
                    "reactiveCycleCount": 0
                },
                "dimensions": {
                    "templateControlFlow": 1,
                    "slotUsage": 0,
                    "propDrilling": 0,
                    "globalState": 0,
                    "provideInject": 0,
                    "fallthroughAttrs": 0,
                    "reactiveGraph": 8
                },
                "totalScore": 9,
                "dominantDimension": { "dimension": "reactive-graph", "score": 8 }
            },
            {
                "fileId": 0,
                "fileName": "Alpha.vue",
                "componentName": "Alpha",
                "input": {
                    "componentCount": 1,
                    "templateIfCount": 0,
                    "templateForCount": 0,
                    "templateLogicalOperatorCount": 0,
                    "componentTreeVIfMaxDepth": 0,
                    "componentTreeVForMaxDepth": 0,
                    "componentTreeScopedSlotMaxDepth": 0,
                    "componentTreeTemplateNestingScore": 0,
                    "slotCount": 0,
                    "propDrillingEdgeCount": 0,
                    "globalStateReferenceCount": 0,
                    "provideInjectMaxDepth": 0,
                    "provideInjectReferenceCount": 0,
                    "provideInjectFanoutCount": 0,
                    "fallthroughRiskCount": 0,
                    "reactiveNodeCount": 2,
                    "reactiveEdgeCount": 1,
                    "reactiveCycleCount": 0
                },
                "dimensions": {
                    "templateControlFlow": 1,
                    "slotUsage": 0,
                    "propDrilling": 0,
                    "globalState": 0,
                    "provideInject": 0,
                    "fallthroughAttrs": 0,
                    "reactiveGraph": 4
                },
                "totalScore": 5,
                "dominantDimension": { "dimension": "reactive-graph", "score": 4 }
            }
        ])
    );
}

#[test]
fn parser_effect_cycles_and_plain_controls_are_not_inferred_from_diagnostics() {
    let mut cyclic = CrossFileAnalyzer::new(CrossFileOptions::minimal());
    add_vue_script(
        &mut cyclic,
        "Cycle.vue",
        r#"
import { computed } from 'vue'
const left = computed(() => right.value)
const right = computed(() => left.value)
"#,
    );
    let cyclic_result = cyclic.analyze();
    assert_eq!(cyclic_result.complexity_report.input.reactive_node_count, 2);
    assert_eq!(cyclic_result.complexity_report.input.reactive_edge_count, 2);
    assert_eq!(
        cyclic_result.complexity_report.input.reactive_cycle_count,
        1
    );
    assert_eq!(
        cyclic_result.complexity_hotspots[0]
            .dimensions
            .reactive_graph,
        16
    );

    let mut plain = CrossFileAnalyzer::new(CrossFileOptions::all());
    add_vue_script(
        &mut plain,
        "Plain.vue",
        "const value = 1\nconsole.log(value)",
    );
    let plain_result = plain.analyze();
    assert_eq!(plain_result.complexity_report.input.reactive_node_count, 0);
    assert_eq!(plain_result.complexity_report.input.reactive_edge_count, 0);
    assert_eq!(plain_result.complexity_report.input.reactive_cycle_count, 0);
    assert_eq!(
        plain_result.complexity_hotspots[0]
            .dimensions
            .reactive_graph,
        0
    );
}

#[test]
fn equally_scored_effect_hotspots_use_filename_order() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());
    add_vue_script(&mut analyzer, "Zulu.vue", ALPHA_SOURCE);
    add_vue_script(&mut analyzer, "Alpha.vue", ALPHA_SOURCE);

    let names: Vec<_> = analyzer
        .analyze()
        .complexity_hotspots
        .into_iter()
        .map(|hotspot| hotspot.file_name)
        .collect();

    assert_eq!(names, ["Alpha.vue", "Zulu.vue"]);
}

#[test]
fn raw_module_add_file_builds_effect_summary_without_sfc_container_parsing() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());
    let alpha = analyzer.add_file("Alpha.ts", ALPHA_SOURCE);
    let beta = analyzer.add_file("Beta.ts", BETA_SOURCE);

    assert_eq!(
        analyzer.effect_graph_summary(alpha).unwrap(),
        build_effect_graph_from_script(ALPHA_SOURCE).summary()
    );
    assert_eq!(
        analyzer.effect_graph_summary(beta).unwrap(),
        build_effect_graph_from_script(BETA_SOURCE).summary()
    );
}

#[test]
fn explicit_sfc_effect_summary_drives_complexity_and_replaces_previous_value() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());
    let source = r#"<script setup lang="ts">const label = '日本語🙂'</script>
<template><p>{{ label }}</p></template>"#;
    let expected = EffectGraphSummary {
        node_count: 9,
        edge_count: 6,
        cycle_count: 2,
        cycle_node_count: 4,
    };
    let file_id = analyzer.add_file_with_analysis_and_effect_summary(
        "Explicit.vue",
        source,
        Croquis::default(),
        expected,
    );

    assert_eq!(analyzer.effect_graph_summary(file_id), Some(expected));
    let result = analyzer.analyze();
    assert_eq!(result.complexity_report.input.reactive_edge_count, 6);
    assert_eq!(result.complexity_report.input.reactive_cycle_count, 2);
    assert_eq!(
        serde_json::to_value(&result.complexity_hotspots).unwrap(),
        json!([{
            "fileId": 0,
            "fileName": "Explicit.vue",
            "componentName": "Explicit",
            "input": {
                "componentCount": 1,
                "templateIfCount": 0,
                "templateForCount": 0,
                "templateLogicalOperatorCount": 0,
                "componentTreeVIfMaxDepth": 0,
                "componentTreeVForMaxDepth": 0,
                "componentTreeScopedSlotMaxDepth": 0,
                "componentTreeTemplateNestingScore": 0,
                "slotCount": 0,
                "propDrillingEdgeCount": 0,
                "globalStateReferenceCount": 0,
                "provideInjectMaxDepth": 0,
                "provideInjectReferenceCount": 0,
                "provideInjectFanoutCount": 0,
                "fallthroughRiskCount": 0,
                "reactiveNodeCount": 0,
                "reactiveEdgeCount": 6,
                "reactiveCycleCount": 2
            },
            "dimensions": {
                "templateControlFlow": 1,
                "slotUsage": 0,
                "propDrilling": 0,
                "globalState": 0,
                "provideInject": 0,
                "fallthroughAttrs": 0,
                "reactiveGraph": 32
            },
            "totalScore": 33,
            "dominantDimension": { "dimension": "reactive-graph", "score": 32 }
        }])
    );

    let updated = analyzer.add_file_with_analysis_and_effect_summary(
        "Explicit.vue",
        source,
        Croquis::default(),
        EffectGraphSummary::default(),
    );
    assert_eq!(updated, file_id);
    assert_eq!(
        analyzer.effect_graph_summary(file_id),
        Some(EffectGraphSummary::default())
    );
    let updated_result = analyzer.analyze();
    assert_eq!(
        updated_result.complexity_report.input.reactive_edge_count,
        0
    );
    assert_eq!(
        updated_result.complexity_report.input.reactive_cycle_count,
        0
    );
}

fn add_vue_script(analyzer: &mut CrossFileAnalyzer, path: &str, script: &str) {
    let mut croquis = Analyzer::with_options(AnalyzerOptions::full());
    croquis.analyze_script_setup(script);
    let analysis = croquis.finish();
    let effect_summary = build_effect_graph_from_script_setup(script).summary();
    let source = cstr!("<script setup lang=\"ts\">\n{script}\n</script>");

    analyzer.add_file_with_analysis_and_effect_summary(path, &source, analysis, effect_summary);
}
