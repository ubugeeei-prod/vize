use vize_croquis_cf::{
    ComplexityBand, ComplexityDimensionScores, ComplexityInput, ComplexityReport,
};

pub(crate) fn complexity_report_json(report: &ComplexityReport) -> serde_json::Value {
    serde_json::json!({
        "input": complexity_input_json(report.input),
        "dimensions": complexity_dimensions_json(report.dimensions),
        "cyclomaticScore": report.cyclomatic_score,
        "cognitiveScore": report.cognitive_score,
        "totalScore": report.total_score,
        "band": complexity_band_name(report.band),
    })
}

fn complexity_input_json(input: ComplexityInput) -> serde_json::Value {
    serde_json::json!({
        "componentCount": input.component_count,
        "templateIfCount": input.template_if_count,
        "templateForCount": input.template_for_count,
        "templateLogicalOperatorCount": input.template_logical_operator_count,
        "componentTreeVIfMaxDepth": input.component_tree_v_if_max_depth,
        "componentTreeVForMaxDepth": input.component_tree_v_for_max_depth,
        "componentTreeScopedSlotMaxDepth": input.component_tree_scoped_slot_max_depth,
        "componentTreeTemplateNestingScore": input.component_tree_template_nesting_score,
        "slotCount": input.slot_count,
        "propDrillingEdgeCount": input.prop_drilling_edge_count,
        "globalStateReferenceCount": input.global_state_reference_count,
        "provideInjectMaxDepth": input.provide_inject_max_depth,
        "provideInjectReferenceCount": input.provide_inject_reference_count,
        "fallthroughRiskCount": input.fallthrough_risk_count,
        "reactiveNodeCount": input.reactive_node_count,
        "reactiveEdgeCount": input.reactive_edge_count,
        "reactiveCycleCount": input.reactive_cycle_count,
    })
}

fn complexity_dimensions_json(dimensions: ComplexityDimensionScores) -> serde_json::Value {
    serde_json::json!({
        "templateControlFlow": dimensions.template_control_flow,
        "slotUsage": dimensions.slot_usage,
        "propDrilling": dimensions.prop_drilling,
        "globalState": dimensions.global_state,
        "provideInject": dimensions.provide_inject,
        "fallthroughAttrs": dimensions.fallthrough_attrs,
        "reactiveGraph": dimensions.reactive_graph,
    })
}

fn complexity_band_name(band: ComplexityBand) -> &'static str {
    match band {
        ComplexityBand::Low => "low",
        ComplexityBand::Moderate => "moderate",
        ComplexityBand::High => "high",
        ComplexityBand::Extreme => "extreme",
    }
}
