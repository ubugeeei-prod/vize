use super::{
    FallthroughInfo, FallthroughUsageAttrKind, analyze_fallthrough,
    collect_fallthrough_usage_facts, summarize_fallthrough,
};
use crate::graph::{DependencyEdge, DependencyGraph, ModuleNode};
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{CompactString, FxHashSet, smallvec};
use vize_croquis::analysis::{ComponentUsage, EventListener, PassedProp};
use vize_croquis::macros::PropDefinition;
use vize_croquis::{Croquis, ScopeId};

fn passed_prop(name: &str) -> PassedProp {
    passed_prop_at(name, 0, 0, false)
}

fn passed_prop_at(name: &str, start: u32, end: u32, is_dynamic: bool) -> PassedProp {
    PassedProp {
        name: CompactString::new(name),
        value: None,
        start,
        end,
        is_dynamic,
    }
}

fn usage_with_prop(name: &str, prop: &str) -> ComponentUsage {
    ComponentUsage {
        name: CompactString::new(name),
        start: 0,
        end: 0,
        props: smallvec![passed_prop(prop)],
        events: smallvec![],
        slots: smallvec![],
        has_spread_attrs: false,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    }
}

fn event_listener_at(name: &str, start: u32, end: u32) -> EventListener {
    EventListener {
        name: CompactString::new(name),
        handler: None,
        modifiers: smallvec![],
        start,
        end,
    }
}

fn declare_prop(analysis: &mut Croquis, name: &str) {
    analysis.macros.add_prop(PropDefinition {
        name: CompactString::new(name),
        prop_type: None,
        required: false,
        default_value: None,
    });
}

fn graph_node(id: FileId, path: &str, component: &str) -> ModuleNode {
    let mut node = ModuleNode::new(id, path);
    node.component_name = Some(CompactString::new(component));
    node
}

#[test]
fn usage_facts_keep_parent_source_ranges_and_attr_classification() {
    let mut registry = ModuleRegistry::new();

    let parent_analysis = {
        let mut analysis = Croquis::new();
        analysis.component_usages.push(ComponentUsage {
            name: CompactString::new("Child"),
            start: 10,
            end: 90,
            props: smallvec![
                passed_prop_at("kind", 18, 29, false),
                passed_prop_at("trackingId", 34, 58, true)
            ],
            events: smallvec![event_listener_at("click", 60, 76)],
            slots: smallvec![],
            has_spread_attrs: true,
            scope_id: ScopeId::ROOT,
            vif_guard: None,
        });
        analysis
    };
    let mut child_analysis = Croquis::new();
    declare_prop(&mut child_analysis, "kind");

    let (parent_id, _) = registry.register("Parent.vue", "", parent_analysis);
    let (child_id, _) = registry.register("Child.vue", "", child_analysis);

    let mut graph = DependencyGraph::new();
    graph.add_node(graph_node(parent_id, "Parent.vue", "Parent"));
    graph.add_node(graph_node(child_id, "Child.vue", "Child"));
    graph.add_edge(parent_id, child_id, DependencyEdge::ComponentUsage);

    let facts = collect_fallthrough_usage_facts(&registry, &graph);
    assert_eq!(facts.len(), 1);

    let fact = &facts[0];
    assert_eq!(fact.parent_file_id, parent_id);
    assert_eq!(fact.child_file_id, child_id);
    assert_eq!(fact.component_name, "Child");
    assert_eq!(fact.usage_start, 10);
    assert_eq!(fact.usage_end, 90);
    assert!(fact.has_spread_attrs);
    assert_eq!(fact.attrs.len(), 3);

    let kind = fact.attrs.iter().find(|attr| attr.name == "kind").unwrap();
    assert_eq!(kind.kind, FallthroughUsageAttrKind::Prop);
    assert_eq!((kind.source_start, kind.source_end), (18, 29));
    assert!(kind.declared_prop);
    assert!(!kind.fallthrough);

    let tracking = fact
        .attrs
        .iter()
        .find(|attr| attr.name == "trackingId")
        .unwrap();
    assert!(tracking.dynamic);
    assert!(tracking.fallthrough);
    assert!(!tracking.standard_html_attr);

    let listener = fact
        .attrs
        .iter()
        .find(|attr| attr.name == "onClick")
        .unwrap();
    assert_eq!(listener.kind, FallthroughUsageAttrKind::Listener);
    assert_eq!((listener.source_start, listener.source_end), (60, 76));
    assert!(listener.dynamic);
    assert!(listener.fallthrough);
    assert!(listener.standard_html_attr);

    let json = serde_json::to_value(fact).unwrap();
    assert_eq!(json["componentName"], "Child");
    assert_eq!(json["attrs"][0]["sourceStart"], 18);
    assert_eq!(json["attrs"][2]["kind"], "listener");
}

#[test]
fn usage_facts_feed_listener_attrs_into_component_aggregates() {
    let mut registry = ModuleRegistry::new();

    let parent_analysis = {
        let mut analysis = Croquis::new();
        analysis.component_usages.push(ComponentUsage {
            name: CompactString::new("Dialog"),
            start: 5,
            end: 40,
            props: smallvec![],
            events: smallvec![event_listener_at("close", 12, 28)],
            slots: smallvec![],
            has_spread_attrs: false,
            scope_id: ScopeId::ROOT,
            vif_guard: None,
        });
        analysis
    };
    let mut child_analysis = Croquis::new();
    child_analysis.template_info.root_element_count = 2;

    let (parent_id, _) = registry.register("Parent.vue", "", parent_analysis);
    let (dialog_id, _) = registry.register("Dialog.vue", "", child_analysis);

    let mut graph = DependencyGraph::new();
    graph.add_node(graph_node(parent_id, "Parent.vue", "Parent"));
    graph.add_node(graph_node(dialog_id, "Dialog.vue", "Dialog"));
    graph.add_edge(parent_id, dialog_id, DependencyEdge::ComponentUsage);

    let (infos, diagnostics) = analyze_fallthrough(&registry, &graph);
    let dialog = infos.iter().find(|info| info.file_id == dialog_id).unwrap();
    assert!(dialog.passed_attrs.contains("onClose"));
    assert_eq!(dialog.fallthrough_attr_count(), 1);
    assert_eq!(dialog.safe_standard_fallthrough_attr_count(), 1);
    assert_eq!(dialog.risky_unconsumed_fallthrough_attr_count(), 0);

    let summary = summarize_fallthrough(&infos);
    assert_eq!(summary.passed_attr_count, 1);
    assert_eq!(summary.safe_standard_fallthrough_attr_count, 1);
    assert_eq!(summary.risky_unconsumed_fallthrough_attr_count, 0);

    assert!(diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic.kind,
            crate::diagnostics::CrossFileDiagnosticKind::MultiRootMissingAttrs
        )
    }));
}

/// A parent that uses two distinct child components, each receiving a
/// different prop, must attribute each prop only to the child it was passed
/// to. Previously `extract_passed_attrs` ignored the child identity and
/// merged every passed prop onto every child.
#[test]
fn passed_attrs_are_attributed_per_child() {
    let mut registry = ModuleRegistry::new();

    let parent_analysis = {
        let mut analysis = Croquis::new();
        analysis
            .component_usages
            .push(usage_with_prop("ChildA", "foo"));
        analysis
            .component_usages
            .push(usage_with_prop("ChildB", "bar"));
        analysis
    };

    let (parent_id, _) = registry.register("Parent.vue", "", parent_analysis);
    let (child_a, _) = registry.register("ChildA.vue", "", Croquis::new());
    let (child_b, _) = registry.register("ChildB.vue", "", Croquis::new());

    let mut graph = DependencyGraph::new();
    graph.add_node(graph_node(parent_id, "Parent.vue", "Parent"));
    graph.add_node(graph_node(child_a, "ChildA.vue", "ChildA"));
    graph.add_node(graph_node(child_b, "ChildB.vue", "ChildB"));
    graph.add_edge(parent_id, child_a, DependencyEdge::ComponentUsage);
    graph.add_edge(parent_id, child_b, DependencyEdge::ComponentUsage);

    let (infos, _diags) = analyze_fallthrough(&registry, &graph);

    let attrs_for = |id: FileId| -> Vec<&str> {
        let info = infos.iter().find(|i| i.file_id == id).unwrap();
        let mut names: Vec<&str> = info.passed_attrs.iter().map(|s| s.as_str()).collect();
        names.sort_unstable();
        names
    };

    // Each child only sees the prop passed to its own usage site.
    assert_eq!(attrs_for(child_a), vec!["foo"]);
    assert_eq!(attrs_for(child_b), vec!["bar"]);

    let summary = summarize_fallthrough(&infos);
    assert_eq!(summary.component_count, 3);
    assert_eq!(summary.components_with_passed_attrs, 2);
    assert_eq!(summary.passed_attr_count, 2);
    assert_eq!(summary.undeclared_passed_attr_count, 2);
    assert_eq!(summary.unconsumed_fallthrough_attr_count, 2);
    assert_eq!(summary.max_passed_attrs, 1);
}

/// The same child component used at two sites with different props must not
/// have its attributes conflated with a sibling child's attributes.
#[test]
fn same_child_used_twice_does_not_leak_sibling_attrs() {
    let mut registry = ModuleRegistry::new();

    let parent_analysis = {
        let mut analysis = Croquis::new();
        // Two usages of the same child, each passing its own prop.
        analysis
            .component_usages
            .push(usage_with_prop("Card", "title"));
        analysis
            .component_usages
            .push(usage_with_prop("Card", "subtitle"));
        // An unrelated sibling child receiving a different prop.
        analysis
            .component_usages
            .push(usage_with_prop("Banner", "color"));
        analysis
    };

    let (parent_id, _) = registry.register("Parent.vue", "", parent_analysis);
    let (card_id, _) = registry.register("Card.vue", "", Croquis::new());
    let (banner_id, _) = registry.register("Banner.vue", "", Croquis::new());

    let mut graph = DependencyGraph::new();
    graph.add_node(graph_node(parent_id, "Parent.vue", "Parent"));
    graph.add_node(graph_node(card_id, "Card.vue", "Card"));
    graph.add_node(graph_node(banner_id, "Banner.vue", "Banner"));
    graph.add_edge(parent_id, card_id, DependencyEdge::ComponentUsage);
    graph.add_edge(parent_id, banner_id, DependencyEdge::ComponentUsage);

    let (infos, _diags) = analyze_fallthrough(&registry, &graph);

    let attrs_for = |id: FileId| -> Vec<&str> {
        let info = infos.iter().find(|i| i.file_id == id).unwrap();
        let mut names: Vec<&str> = info.passed_attrs.iter().map(|s| s.as_str()).collect();
        names.sort_unstable();
        names
    };

    // Card aggregates props from both of its usage sites, but never picks
    // up `color`, which only the Banner sibling received.
    assert_eq!(attrs_for(card_id), vec!["subtitle", "title"]);
    assert_eq!(attrs_for(banner_id), vec!["color"]);
}

#[test]
fn test_fallthrough_info_issues() {
    // Single root element - no issue
    let mut info = FallthroughInfo {
        file_id: FileId::new(0),
        inherit_attrs_disabled: false,
        uses_attrs: false,
        binds_attrs: false,
        root_element_count: 1,
        passed_attrs: FxHashSet::default(),
        declared_props: FxHashSet::default(),
        template_start: 0,
        template_end: 0,
    };
    assert!(!info.has_potential_issues());

    // Multiple roots without binds_attrs - this IS an issue
    info.root_element_count = 2;
    assert!(info.has_potential_issues());

    // Multiple roots WITH binds_attrs - no issue
    info.binds_attrs = true;
    assert!(!info.has_potential_issues());

    // Reset and test inheritAttrs disabled without using $attrs
    info.binds_attrs = false;
    info.root_element_count = 1;
    info.inherit_attrs_disabled = true;
    assert!(info.has_potential_issues());

    // inheritAttrs disabled but $attrs is used - no issue
    info.uses_attrs = true;
    assert!(!info.has_potential_issues());
}
