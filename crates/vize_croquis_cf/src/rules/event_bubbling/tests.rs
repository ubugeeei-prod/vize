use super::{analyze_event_bubbling, extract_modifiers, trace_event_propagation};
use crate::diagnostics::CrossFileDiagnosticKind;
use crate::graph::{DependencyEdge, DependencyGraph, ModuleNode};
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{CompactString, FxHashMap};
use vize_croquis::{Analyzer, AnalyzerOptions, Croquis, EventHandlerScopeData};

#[test]
fn test_extract_modifiers() {
    let modifiers = extract_modifiers("@click.stop.prevent");
    assert!(modifiers.contains(&CompactString::new("stop")));
    assert!(modifiers.contains(&CompactString::new("prevent")));
}

#[test]
fn test_multi_parent_propagation_reports_unhandled_branch() {
    let mut registry = ModuleRegistry::new();
    let mut graph = DependencyGraph::new();

    let child = add_vue_component(
        &mut registry,
        &mut graph,
        "Child.vue",
        script_analysis("const emit = defineEmits(['save'])"),
    );
    let parent_a = add_vue_component(
        &mut registry,
        &mut graph,
        "ParentA.vue",
        analysis_with_handler("save"),
    );
    let parent_b = add_vue_component(&mut registry, &mut graph, "ParentB.vue", Croquis::default());
    let grand_b = add_vue_component(&mut registry, &mut graph, "GrandB.vue", Croquis::default());
    let great_b = add_vue_component(&mut registry, &mut graph, "GreatB.vue", Croquis::default());

    graph.add_edge(parent_a, child, DependencyEdge::ComponentUsage);
    graph.add_edge(parent_b, child, DependencyEdge::ComponentUsage);
    graph.add_edge(grand_b, parent_b, DependencyEdge::ComponentUsage);
    graph.add_edge(great_b, grand_b, DependencyEdge::ComponentUsage);

    let (bubbles, diagnostics) = analyze_event_bubbling(&registry, &graph);

    assert_eq!(bubbles.len(), 2);
    assert_eq!(
        bubbles
            .iter()
            .map(|bubble| bubble.propagation_path.clone())
            .collect::<Vec<_>>(),
        vec![
            vec![child, parent_a],
            vec![child, parent_b, grand_b, great_b]
        ]
    );
    assert_eq!(bubbles[0].handler, Some(parent_a));
    assert_eq!(bubbles[1].handler, None);

    let unhandled = diagnostics
        .iter()
        .filter_map(|diagnostic| match &diagnostic.kind {
            CrossFileDiagnosticKind::UnhandledEvent { event_name, depth } => {
                Some((event_name.as_str(), *depth))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(unhandled, vec![("save", 4)]);
}

#[test]
fn test_trace_event_propagation_skips_cycles() {
    let child = FileId::new(0);
    let parent = FileId::new(1);
    let mut graph = DependencyGraph::new();
    graph.add_node(ModuleNode::new(child, "Child.vue"));
    graph.add_node(ModuleNode::new(parent, "Parent.vue"));
    graph.add_edge(parent, child, DependencyEdge::ComponentUsage);
    graph.add_edge(child, parent, DependencyEdge::ComponentUsage);

    let bubbles = trace_event_propagation(child, "save", &graph, &FxHashMap::default());

    assert_eq!(bubbles.len(), 1);
    assert_eq!(bubbles[0].propagation_path, vec![child, parent]);
    assert_eq!(bubbles[0].handler, None);
}

fn add_vue_component(
    registry: &mut ModuleRegistry,
    graph: &mut DependencyGraph,
    path: &str,
    analysis: Croquis,
) -> FileId {
    let (id, is_new) = registry.register(path, "", analysis);
    assert!(is_new);
    graph.add_node(ModuleNode::new(id, path));
    id
}

fn script_analysis(script: &str) -> Croquis {
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.finish()
}

fn analysis_with_handler(event_name: &str) -> Croquis {
    let mut analysis = Croquis::default();
    analysis.scopes.enter_event_handler_scope(
        EventHandlerScopeData {
            event_name: CompactString::new(event_name),
            has_implicit_event: false,
            param_names: Default::default(),
            handler_expression: Some(CompactString::new("onSave")),
            target_component: None,
        },
        0,
        0,
    );
    analysis.scopes.exit_scope();
    analysis
}
