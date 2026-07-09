use super::complexity::summarize_complexity;
use super::{
    ComplexityBand, ComplexityDimension, ComplexityInput, ComplexityReport, FallthroughInfo,
    ProvideInjectTreeSummary, ReactivityIssue, ReactivityIssueKind,
    summarize_complexity_with_graph,
};
use crate::analyzer::CrossFileResult;
use crate::graph::{DependencyEdge, DependencyGraph, ModuleNode};
use crate::registry::ModuleRegistry;
use vize_carton::{CompactString, FxHashSet, smallvec};
use vize_croquis::croquis::{
    ComponentUsage, EventListener, PassedProp, SlotUsage, TemplateExpression,
    TemplateExpressionKind,
};
use vize_croquis::{Croquis, ScopeId, VForScopeData, VSlotScopeData};

#[test]
fn scores_each_complexity_dimension() {
    let report = ComplexityReport::from_input(ComplexityInput {
        component_count: 1,
        template_if_count: 2,
        template_for_count: 1,
        template_logical_operator_count: 2,
        component_tree_v_if_max_depth: 3,
        component_tree_v_for_max_depth: 2,
        component_tree_scoped_slot_max_depth: 3,
        component_tree_template_nesting_score: 9,
        slot_count: 3,
        prop_drilling_edge_count: 2,
        global_state_reference_count: 4,
        provide_inject_max_depth: 3,
        provide_inject_reference_count: 5,
        fallthrough_risk_count: 2,
        reactive_node_count: 6,
        reactive_edge_count: 7,
        reactive_cycle_count: 1,
    });

    assert_eq!(report.cyclomatic_score, 6);
    assert_eq!(report.cognitive_score, 9);
    assert_eq!(report.dimensions.template_control_flow, 15);
    assert_eq!(report.dimensions.slot_usage, 14);
    assert_eq!(report.dimensions.prop_drilling, 6);
    assert_eq!(report.dimensions.global_state, 8);
    assert_eq!(report.dimensions.provide_inject, 9);
    assert_eq!(report.dimensions.fallthrough_attrs, 8);
    assert_eq!(report.dimensions.reactive_graph, 30);
    assert_eq!(report.total_score, 90);
    assert_eq!(report.band, ComplexityBand::Extreme);

    let dominant = report
        .dominant_dimension()
        .expect("non-zero report should expose a dominant dimension");
    assert_eq!(dominant.dimension, ComplexityDimension::ReactiveGraph);
    assert_eq!(dominant.dimension.as_str(), "reactive-graph");
    assert_eq!(dominant.score, 30);
}

#[test]
fn dominant_dimension_is_none_for_zero_score() {
    let report = ComplexityReport::from_input(ComplexityInput::default());

    assert!(report.dominant_dimension().is_none());
}

#[test]
fn summarizes_complexity_from_registry_and_result() {
    let mut analysis = Croquis::new();
    analysis.template_expressions.push(TemplateExpression {
        content: CompactString::new("ready && active"),
        kind: TemplateExpressionKind::VIf,
        start: 0,
        end: 5,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    });
    analysis.scopes.enter_v_for_scope(
        vize_croquis::VForScopeData {
            value_alias: CompactString::new("item"),
            value_bindings: smallvec![CompactString::new("item")],
            key_alias: None,
            index_alias: None,
            source: CompactString::new("items"),
            key_expression: None,
        },
        6,
        20,
    );
    analysis.component_usages.push(ComponentUsage {
        name: CompactString::new("Child"),
        start: 21,
        end: 40,
        props: smallvec![PassedProp {
            name: CompactString::new("value"),
            value: Some(CompactString::new("item")),
            start: 22,
            end: 30,
            is_dynamic: true,
        }],
        events: smallvec![EventListener {
            name: CompactString::new("save"),
            handler: None,
            modifiers: smallvec![],
            start: 31,
            end: 35,
        }],
        slots: smallvec![SlotUsage {
            name: CompactString::new("default"),
            scope_vars: smallvec![CompactString::new("row")],
            start: 36,
            end: 40,
            has_scope: true,
        }],
        has_spread_attrs: false,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    });
    analysis.reactivity.register(
        CompactString::new("state"),
        vize_croquis::reactivity::ReactiveKind::Ref,
        0,
    );

    let mut registry = ModuleRegistry::new();
    let (file_id, _) = registry.register("Parent.vue", "", analysis);
    let result = CrossFileResult {
        fallthrough_info: vec![FallthroughInfo {
            file_id,
            inherit_attrs_disabled: false,
            uses_attrs: false,
            binds_attrs: false,
            root_element_count: 2,
            passed_attrs: FxHashSet::from_iter([CompactString::new("class")]),
            declared_props: FxHashSet::default(),
            template_start: 0,
            template_end: 10,
        }],
        provide_inject_tree_summary: Some(ProvideInjectTreeSummary {
            max_depth: 3,
            provide_count: 1,
            inject_count: 2,
            ..ProvideInjectTreeSummary::default()
        }),
        reactivity_issues: vec![ReactivityIssue {
            file_id,
            kind: ReactivityIssueKind::ShouldUseStoreToRefs {
                store_name: CompactString::new("useUserStore"),
            },
            offset: 1,
            source: None,
        }],
        ..CrossFileResult::default()
    };

    let report = summarize_complexity(&registry, &result);

    assert_eq!(report.input.template_if_count, 1);
    assert_eq!(report.input.template_for_count, 1);
    assert_eq!(report.input.template_logical_operator_count, 1);
    assert_eq!(report.cyclomatic_score, 4);
    assert_eq!(report.cognitive_score, 0);
    assert_eq!(report.input.slot_count, 1);
    assert_eq!(report.input.prop_drilling_edge_count, 1);
    assert_eq!(report.input.global_state_reference_count, 1);
    assert_eq!(report.input.provide_inject_max_depth, 3);
    assert_eq!(report.input.provide_inject_reference_count, 3);
    assert_eq!(report.input.fallthrough_risk_count, 1);
    assert_eq!(report.input.reactive_node_count, 1);
    assert_eq!(report.input.reactive_edge_count, 1);
    assert_eq!(report.total_score, 25);
    assert_eq!(report.band, ComplexityBand::Moderate);
}

#[test]
fn summarizes_component_tree_template_nesting() {
    let mut parent = Croquis::new();
    let parent_loop = parent
        .scopes
        .enter_v_for_scope(v_for_data("row", "rows"), 0, 20);
    parent.component_usages.push(ComponentUsage {
        name: CompactString::new("Child"),
        start: 21,
        end: 60,
        props: smallvec![],
        events: smallvec![],
        slots: smallvec![SlotUsage {
            name: CompactString::new("default"),
            scope_vars: smallvec![CompactString::new("slotProps")],
            start: 35,
            end: 55,
            has_scope: true,
        }],
        has_spread_attrs: false,
        scope_id: parent_loop,
        vif_guard: Some(CompactString::new("ready")),
    });

    let mut child = Croquis::new();
    child.template_expressions.push(TemplateExpression {
        content: CompactString::new("expanded"),
        kind: TemplateExpressionKind::VIf,
        start: 0,
        end: 8,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    });
    child
        .scopes
        .enter_v_for_scope(v_for_data("item", "items"), 9, 30);
    child.scopes.exit_scope();
    child.scopes.enter_v_slot_scope(
        VSlotScopeData {
            name: CompactString::new("default"),
            props_pattern: None,
            prop_names: smallvec![CompactString::new("item")],
            component: Some(CompactString::new("GrandChild")),
        },
        31,
        50,
    );

    let mut registry = ModuleRegistry::new();
    let (parent_id, _) = registry.register("Parent.vue", "", parent);
    let (child_id, _) = registry.register("Child.vue", "", child);
    let mut graph = DependencyGraph::new();
    graph.add_node(component_node(parent_id, "Parent.vue", "Parent"));
    graph.add_node(component_node(child_id, "Child.vue", "Child"));
    graph.add_edge(parent_id, child_id, DependencyEdge::ComponentUsage);

    let report = summarize_complexity_with_graph(&registry, &graph, &CrossFileResult::default());

    assert_eq!(report.input.template_if_count, 1);
    assert_eq!(report.input.template_for_count, 2);
    assert_eq!(report.input.slot_count, 1);
    assert_eq!(report.input.component_tree_v_if_max_depth, 2);
    assert_eq!(report.input.component_tree_v_for_max_depth, 2);
    assert_eq!(report.input.component_tree_scoped_slot_max_depth, 2);
    assert_eq!(report.input.component_tree_template_nesting_score, 17);
    assert_eq!(report.cyclomatic_score, 5);
    assert_eq!(report.cognitive_score, 17);
    assert_eq!(report.dimensions.template_control_flow, 22);
    assert_eq!(report.dimensions.slot_usage, 6);
}

#[test]
fn score_saturates_instead_of_overflowing() {
    let report = ComplexityReport::from_input(ComplexityInput {
        reactive_cycle_count: usize::MAX,
        ..ComplexityInput::default()
    });

    assert_eq!(report.dimensions.reactive_graph, u32::MAX);
    assert_eq!(report.total_score, u32::MAX);
    assert_eq!(report.band, ComplexityBand::Extreme);
}

fn v_for_data(value_alias: &str, source: &str) -> VForScopeData {
    VForScopeData {
        value_alias: CompactString::new(value_alias),
        value_bindings: smallvec![CompactString::new(value_alias)],
        key_alias: None,
        index_alias: None,
        source: CompactString::new(source),
        key_expression: None,
    }
}

fn component_node(id: crate::FileId, path: &str, name: &str) -> ModuleNode {
    let mut node = ModuleNode::new(id, path);
    node.component_name = Some(CompactString::new(name));
    node
}
