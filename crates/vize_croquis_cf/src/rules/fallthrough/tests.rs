use super::{FallthroughInfo, analyze_fallthrough};
use crate::graph::{DependencyEdge, DependencyGraph, ModuleNode};
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{CompactString, FxHashSet, smallvec};
use vize_croquis::analysis::{ComponentUsage, PassedProp};
use vize_croquis::{Croquis, ScopeId};

fn passed_prop(name: &str) -> PassedProp {
    PassedProp {
        name: CompactString::new(name),
        value: None,
        start: 0,
        end: 0,
        is_dynamic: false,
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

fn graph_node(id: FileId, path: &str, component: &str) -> ModuleNode {
    let mut node = ModuleNode::new(id, path);
    node.component_name = Some(CompactString::new(component));
    node
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
