use super::analyze_fallthrough;
use crate::diagnostics::CrossFileDiagnosticKind;
use crate::graph::{DependencyEdge, DependencyGraph, ModuleNode};
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{CompactString, smallvec};
use vize_croquis::analysis::{ComponentUsage, PassedProp};
use vize_croquis::{Croquis, ScopeId};

fn passed_prop_at(name: &str, start: u32, end: u32, is_dynamic: bool) -> PassedProp {
    PassedProp {
        name: CompactString::new(name),
        value: None,
        start,
        end,
        is_dynamic,
    }
}

fn graph_node(id: FileId, path: &str, component: &str) -> ModuleNode {
    let mut node = ModuleNode::new(id, path);
    node.component_name = Some(CompactString::new(component));
    node
}

#[test]
fn diagnostics_include_parent_usage_related_locations() {
    let mut registry = ModuleRegistry::new();

    let parent_analysis = {
        let mut analysis = Croquis::new();
        analysis.component_usages.push(ComponentUsage {
            name: CompactString::new("Panel"),
            start: 10,
            end: 80,
            props: smallvec![passed_prop_at("trackingId", 34, 58, true)],
            events: smallvec![],
            slots: smallvec![],
            has_spread_attrs: false,
            scope_id: ScopeId::ROOT,
            vif_guard: None,
        });
        analysis
    };
    let mut panel_analysis = Croquis::new();
    panel_analysis.template_info.root_element_count = 2;

    let (parent_id, _) = registry.register("Parent.vue", "", parent_analysis);
    let (panel_id, _) = registry.register("Panel.vue", "", panel_analysis);

    let mut graph = DependencyGraph::new();
    graph.add_node(graph_node(parent_id, "Parent.vue", "Parent"));
    graph.add_node(graph_node(panel_id, "Panel.vue", "Panel"));
    graph.add_edge(parent_id, panel_id, DependencyEdge::ComponentUsage);

    let (_infos, diagnostics) = analyze_fallthrough(&registry, &graph);
    let multi_root = diagnostics
        .iter()
        .find(|diagnostic| {
            matches!(
                diagnostic.kind,
                CrossFileDiagnosticKind::MultiRootMissingAttrs
            )
        })
        .expect("multi-root diagnostic should be emitted");
    assert_eq!(multi_root.primary_file, panel_id);
    assert_eq!(multi_root.related_files.len(), 1);
    assert_eq!(multi_root.related_files[0].0, parent_id);
    assert_eq!(multi_root.related_files[0].1, 34);
    assert!(
        multi_root.related_files[0]
            .2
            .contains("trackingId passed to <Panel>")
    );

    let unused = diagnostics
        .iter()
        .find(|diagnostic| {
            matches!(
                diagnostic.kind,
                CrossFileDiagnosticKind::UnusedFallthroughAttrs { .. }
            )
        })
        .expect("unused fallthrough diagnostic should be emitted");
    assert_eq!(unused.related_files.len(), 1);
    assert_eq!(unused.related_files[0].0, parent_id);
    assert_eq!(unused.related_files[0].1, 34);
}

#[test]
fn spread_attrs_report_multi_root_with_parent_usage_location() {
    let mut registry = ModuleRegistry::new();

    let parent_analysis = {
        let mut analysis = Croquis::new();
        analysis.component_usages.push(ComponentUsage {
            name: CompactString::new("Panel"),
            start: 23,
            end: 54,
            props: smallvec![],
            events: smallvec![],
            slots: smallvec![],
            has_spread_attrs: true,
            scope_id: ScopeId::ROOT,
            vif_guard: None,
        });
        analysis
    };
    let mut panel_analysis = Croquis::new();
    panel_analysis.template_info.root_element_count = 2;

    let (parent_id, _) = registry.register("Parent.vue", "", parent_analysis);
    let (panel_id, _) = registry.register("Panel.vue", "", panel_analysis);

    let mut graph = DependencyGraph::new();
    graph.add_node(graph_node(parent_id, "Parent.vue", "Parent"));
    graph.add_node(graph_node(panel_id, "Panel.vue", "Panel"));
    graph.add_edge(parent_id, panel_id, DependencyEdge::ComponentUsage);

    let (_infos, diagnostics) = analyze_fallthrough(&registry, &graph);
    let multi_root = diagnostics
        .iter()
        .find(|diagnostic| {
            matches!(
                diagnostic.kind,
                CrossFileDiagnosticKind::MultiRootMissingAttrs
            )
        })
        .expect("spread attrs should trigger a multi-root diagnostic");

    assert_eq!(multi_root.primary_file, panel_id);
    assert_eq!(multi_root.related_files.len(), 1);
    assert_eq!(multi_root.related_files[0].0, parent_id);
    assert_eq!(multi_root.related_files[0].1, 23);
    assert_eq!(
        multi_root.related_files[0].2,
        "v-bind spread passed to <Panel>"
    );
    assert!(diagnostics.iter().all(|diagnostic| !matches!(
        diagnostic.kind,
        CrossFileDiagnosticKind::UnusedFallthroughAttrs { .. }
    )));
}

#[test]
fn spread_attrs_are_safe_when_multi_root_component_binds_attrs() {
    let mut registry = ModuleRegistry::new();

    let mut parent_analysis = Croquis::new();
    parent_analysis.component_usages.push(ComponentUsage {
        name: CompactString::new("Panel"),
        start: 10,
        end: 40,
        props: smallvec![],
        events: smallvec![],
        slots: smallvec![],
        has_spread_attrs: true,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    });
    let mut panel_analysis = Croquis::new();
    panel_analysis.template_info.root_element_count = 2;
    panel_analysis.template_info.binds_attrs_explicitly = true;

    let (parent_id, _) = registry.register("Parent.vue", "", parent_analysis);
    let (panel_id, _) = registry.register("Panel.vue", "", panel_analysis);

    let mut graph = DependencyGraph::new();
    graph.add_node(graph_node(parent_id, "Parent.vue", "Parent"));
    graph.add_node(graph_node(panel_id, "Panel.vue", "Panel"));
    graph.add_edge(parent_id, panel_id, DependencyEdge::ComponentUsage);

    let (_infos, diagnostics) = analyze_fallthrough(&registry, &graph);

    assert!(diagnostics.iter().all(|diagnostic| !matches!(
        diagnostic.kind,
        CrossFileDiagnosticKind::MultiRootMissingAttrs
    )));
}
