use vize_croquis_cf::{ComplexityHotspot, ComplexityReport};

pub(crate) fn complexity_report_json(report: &ComplexityReport) -> serde_json::Value {
    serde_json::to_value(report).expect("complexity report should serialize")
}

pub(crate) fn complexity_hotspots_json(hotspots: &[ComplexityHotspot]) -> serde_json::Value {
    serde_json::to_value(hotspots).expect("complexity hotspots should serialize")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use vize_croquis_cf::{
        ComplexityDimension, ComplexityDimensionBreakdown, ComplexityDimensionScores,
        ComplexityInput, FileId,
    };

    #[test]
    fn complexity_hotspots_json_preserves_js_shape() {
        let hotspots = vec![ComplexityHotspot {
            file_id: FileId::new(7),
            file_name: "App.vue".into(),
            component_name: Some("App".into()),
            input: ComplexityInput {
                component_count: 1,
                template_if_count: 2,
                prop_drilling_edge_count: 1,
                provide_inject_fanout_count: 4,
                ..ComplexityInput::default()
            },
            dimensions: ComplexityDimensionScores {
                template_control_flow: 3,
                prop_drilling: 3,
                ..ComplexityDimensionScores::default()
            },
            total_score: 6,
            dominant_dimension: Some(ComplexityDimensionBreakdown {
                dimension: ComplexityDimension::TemplateControlFlow,
                score: 3,
            }),
        }];

        assert_eq!(
            complexity_hotspots_json(&hotspots),
            json!([
                {
                    "fileId": 7,
                    "fileName": "App.vue",
                    "componentName": "App",
                    "input": {
                        "componentCount": 1,
                        "templateIfCount": 2,
                        "templateForCount": 0,
                        "templateLogicalOperatorCount": 0,
                        "componentTreeVIfMaxDepth": 0,
                        "componentTreeVForMaxDepth": 0,
                        "componentTreeScopedSlotMaxDepth": 0,
                        "componentTreeTemplateNestingScore": 0,
                        "slotCount": 0,
                        "propDrillingEdgeCount": 1,
                        "globalStateReferenceCount": 0,
                        "provideInjectMaxDepth": 0,
                        "provideInjectReferenceCount": 0,
                        "provideInjectFanoutCount": 4,
                        "fallthroughRiskCount": 0,
                        "reactiveNodeCount": 0,
                        "reactiveEdgeCount": 0,
                        "reactiveCycleCount": 0
                    },
                    "dimensions": {
                        "templateControlFlow": 3,
                        "slotUsage": 0,
                        "propDrilling": 3,
                        "globalState": 0,
                        "provideInject": 0,
                        "fallthroughAttrs": 0,
                        "reactiveGraph": 0
                    },
                    "totalScore": 6,
                    "dominantDimension": {
                        "dimension": "template-control-flow",
                        "score": 3
                    }
                }
            ])
        );
    }

    #[test]
    fn complexity_hotspots_json_serializes_empty_list() {
        assert_eq!(complexity_hotspots_json(&[]), json!([]));
    }

    #[test]
    fn parser_effect_graph_counts_reach_wasm_complexity_json() {
        const SOURCE: &str = r#"
import { computed, ref, watch, watchEffect } from 'vue'
const count = ref(0)
const doubled = computed(() => count.value * 2)
watchEffect(() => console.log(doubled.value))
watch(count, () => {})
"#;

        let mut drawer =
            vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
        drawer.analyze_script_setup(SOURCE);
        let mut analyzer =
            vize_croquis_cf::CrossFileAnalyzer::new(vize_croquis_cf::CrossFileOptions::minimal());
        analyzer.add_file_with_analysis("Reactive.vue", SOURCE, drawer.finish());
        let result = analyzer.analyze();

        assert_eq!(
            complexity_report_json(&result.complexity_report),
            json!({
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
                "cyclomaticScore": 1,
                "cognitiveScore": 0,
                "totalScore": 9,
                "band": "low"
            })
        );
        assert_eq!(
            complexity_hotspots_json(&result.complexity_hotspots),
            json!([{
                "fileId": 0,
                "fileName": "Reactive.vue",
                "componentName": "Reactive",
                "input": result.complexity_report.input,
                "dimensions": result.complexity_report.dimensions,
                "totalScore": 9,
                "dominantDimension": { "dimension": "reactive-graph", "score": 8 }
            }])
        );
    }
}
