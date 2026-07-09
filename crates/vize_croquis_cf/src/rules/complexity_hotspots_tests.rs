use super::{
    ComplexityDimension, CrossFileReactivityIssue, CrossFileReactivityIssueKind, FallthroughInfo,
    ProvideInjectMatch, ReactivityIssue, ReactivityIssueKind,
    summarize_complexity_hotspots_with_effect_graphs,
};
use crate::analyzer::CrossFileResult;
use crate::diagnostics::DiagnosticSeverity;
use crate::registry::ModuleRegistry;
use vize_carton::{CompactString, FxHashSet, smallvec};
use vize_croquis::analysis::{ComponentUsage, PassedProp, SlotUsage, TemplateExpression};
use vize_croquis::reactivity::ReactiveKind;
use vize_croquis::{Croquis, EffectGraphSummary, ScopeId, TemplateExpressionKind, VForScopeData};

#[test]
fn ranks_hotspots_with_dimension_inputs_and_json_shape() {
    let mut parent = Croquis::new();
    parent.template_expressions.push(TemplateExpression {
        content: CompactString::new("ready && active"),
        kind: TemplateExpressionKind::VIf,
        start: 0,
        end: 15,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    });
    parent.scopes.enter_v_for_scope(v_for_data(), 16, 40);
    parent.component_usages.push(ComponentUsage {
        name: CompactString::new("Child"),
        start: 41,
        end: 90,
        props: smallvec![PassedProp {
            name: CompactString::new("model"),
            value: Some(CompactString::new("state")),
            start: 45,
            end: 60,
            is_dynamic: true,
        }],
        events: smallvec![],
        slots: smallvec![SlotUsage {
            name: CompactString::new("default"),
            scope_vars: smallvec![CompactString::new("slotProps")],
            start: 61,
            end: 89,
            has_scope: true,
        }],
        has_spread_attrs: false,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    });
    parent
        .reactivity
        .register(CompactString::new("state"), ReactiveKind::Reactive, 10);

    let child = Croquis::new();
    let mut registry = ModuleRegistry::new();
    let (parent_id, _) = registry.register("Parent.vue", "", parent);
    let (child_id, _) = registry.register("Child.vue", "", child);

    let result = CrossFileResult {
        fallthrough_info: vec![FallthroughInfo {
            file_id: child_id,
            inherit_attrs_disabled: false,
            uses_attrs: false,
            binds_attrs: false,
            root_element_count: 2,
            passed_attrs: FxHashSet::from_iter([CompactString::new("trackingId")]),
            fallthrough_attrs: FxHashSet::from_iter([CompactString::new("trackingId")]),
            declared_props: FxHashSet::default(),
            declared_events: FxHashSet::default(),
            template_start: 0,
            template_end: 10,
        }],
        provide_inject_matches: vec![ProvideInjectMatch {
            provider: parent_id,
            consumer: child_id,
            key: CompactString::new("theme"),
            key_identity: CompactString::new("string:theme"),
            path: vec![parent_id, child_id],
            type_match: Some(true),
            provide_offset: 20,
            inject_offset: 5,
        }],
        reactivity_issues: vec![ReactivityIssue {
            file_id: parent_id,
            kind: ReactivityIssueKind::ShouldUseStoreToRefs {
                store_name: CompactString::new("useUserStore"),
            },
            offset: 1,
            source: None,
        }],
        cross_file_reactivity_issues: vec![
            CrossFileReactivityIssue {
                file_id: child_id,
                kind: CrossFileReactivityIssueKind::StoreDestructured {
                    store_name: CompactString::new("useCartStore"),
                    destructured_props: vec![CompactString::new("items")],
                },
                offset: 2,
                related_file: Some(parent_id),
                severity: DiagnosticSeverity::Warning,
            },
            CrossFileReactivityIssue {
                file_id: child_id,
                kind: CrossFileReactivityIssueKind::CircularReactiveDependency {
                    cycle: vec![CompactString::new("left"), CompactString::new("right")],
                },
                offset: 3,
                related_file: None,
                severity: DiagnosticSeverity::Error,
            },
        ],
        ..CrossFileResult::default()
    };

    let effect_graphs = vize_carton::FxHashMap::from_iter([
        (
            parent_id,
            EffectGraphSummary {
                node_count: 2,
                edge_count: 1,
                cycle_count: 0,
                cycle_node_count: 0,
            },
        ),
        (
            child_id,
            EffectGraphSummary {
                node_count: 3,
                edge_count: 2,
                cycle_count: 1,
                cycle_node_count: 2,
            },
        ),
    ]);
    let hotspots =
        summarize_complexity_hotspots_with_effect_graphs(&registry, &effect_graphs, &result);

    assert_eq!(hotspots.len(), 2);
    assert_eq!(hotspots[0].file_id, child_id);
    assert_eq!(hotspots[0].file_name, "Child.vue");
    assert_eq!(hotspots[0].component_name.as_deref(), Some("Child"));
    assert_eq!(hotspots[0].input.fallthrough_risk_count, 2);
    assert_eq!(hotspots[0].input.reactive_edge_count, 2);
    assert_eq!(hotspots[0].input.reactive_cycle_count, 1);
    assert_eq!(hotspots[0].input.provide_inject_max_depth, 2);
    assert_eq!(hotspots[0].input.provide_inject_reference_count, 1);
    assert_eq!(hotspots[0].dimensions.fallthrough_attrs, 8);
    assert_eq!(hotspots[0].dimensions.reactive_graph, 14);
    assert_eq!(hotspots[0].total_score, 28);
    assert_eq!(
        hotspots[0].dominant_dimension.unwrap().dimension,
        ComplexityDimension::ReactiveGraph
    );

    assert_eq!(hotspots[1].file_id, parent_id);
    assert_eq!(hotspots[1].input.template_if_count, 1);
    assert_eq!(hotspots[1].input.template_for_count, 1);
    assert_eq!(hotspots[1].input.template_logical_operator_count, 1);
    assert_eq!(hotspots[1].input.slot_count, 1);
    assert_eq!(hotspots[1].input.prop_drilling_edge_count, 1);
    assert_eq!(hotspots[1].input.provide_inject_reference_count, 1);
    assert_eq!(hotspots[1].input.provide_inject_fanout_count, 1);
    assert_eq!(hotspots[1].dimensions.template_control_flow, 4);
    assert_eq!(hotspots[1].total_score, 17);

    let json = serde_json::to_value(&hotspots[0]).unwrap();
    assert_eq!(json["fileName"], "Child.vue");
    assert_eq!(json["componentName"], "Child");
    assert_eq!(json["input"]["fallthroughRiskCount"], 2);
    assert_eq!(json["dominantDimension"]["dimension"], "reactive-graph");
}

#[test]
fn analyzer_result_stores_complexity_hotspots() {
    let mut analyzer = crate::CrossFileAnalyzer::new(
        crate::CrossFileOptions::default()
            .with_fallthrough_attrs(true)
            .with_reactivity_tracking(true),
    );

    let mut parent = Croquis::new();
    parent.used_components.insert(CompactString::new("Child"));
    parent.component_usages.push(ComponentUsage {
        name: CompactString::new("Child"),
        start: 0,
        end: 30,
        props: smallvec![PassedProp {
            name: CompactString::new("trackingId"),
            value: Some(CompactString::new("id")),
            start: 8,
            end: 24,
            is_dynamic: true,
        }],
        events: smallvec![],
        slots: smallvec![],
        has_spread_attrs: false,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    });

    let mut child = Croquis::new();
    child.template_info.root_element_count = 2;

    analyzer.add_file_with_analysis("Parent.vue", "const id = 'abc'", parent);
    analyzer.add_file_with_analysis("Child.vue", "", child);
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();

    assert!(!result.complexity_hotspots.is_empty());
    assert!(result.complexity_hotspots.iter().any(
        |hotspot| hotspot.file_name == "Child.vue" && hotspot.input.fallthrough_risk_count >= 1
    ));
}

fn v_for_data() -> VForScopeData {
    VForScopeData {
        value_alias: CompactString::new("item"),
        value_bindings: smallvec![CompactString::new("item")],
        key_alias: None,
        index_alias: None,
        source: CompactString::new("items"),
        key_expression: None,
    }
}
