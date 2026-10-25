use crate::{DependencyEdge, DependencyGraph, FileId, ModuleNode};
use vize_l0::{String, cstr};

fn graph(count: u32) -> DependencyGraph {
    let mut graph = DependencyGraph::new();
    for id in 0..count {
        graph.add_node(ModuleNode::new(FileId::new(id), cstr!("{id}.vue")));
    }
    graph
}

fn raw_nodes(graph: &DependencyGraph) -> Vec<String> {
    graph.nodes().map(|node| cstr!("{node:?}")).collect()
}

#[test]
fn preserves_both_subsequences_and_every_other_raw_node_field() {
    use DependencyEdge::{
        ComponentUsage as C, DynamicImport as D, EventEmit as E, Import as I, ProvideInject as P,
        ReExport as R, SlotContent as S, TypeImport as T,
    };
    let mut graph = graph(9);
    for (source, kind) in [
        (1, C),
        (2, T),
        (3, D),
        (4, C),
        (5, I),
        (6, P),
        (7, E),
        (8, S),
        (2, R),
    ] {
        graph.add_edge(FileId::new(source), FileId::new(0), kind);
    }
    let original_outgoing: Vec<_> = graph
        .nodes()
        .map(|node| (node.file_id, node.imports.clone()))
        .collect();
    let mut expected = graph.get_node(FileId::new(0)).unwrap().clone();
    expected.importers = [
        (2, T),
        (3, D),
        (5, I),
        (6, P),
        (7, E),
        (8, S),
        (2, R),
        (1, C),
        (4, C),
    ]
    .into_iter()
    .map(|(id, kind)| (FileId::new(id), kind))
    .collect();

    graph.finalize_importer_order();

    assert_eq!(
        cstr!("{:?}", graph.get_node(FileId::new(0)).unwrap()),
        cstr!("{expected:?}")
    );
    assert_eq!(
        graph
            .nodes()
            .map(|node| (node.file_id, node.imports.clone()))
            .collect::<Vec<_>>(),
        original_outgoing
    );
    let first = raw_nodes(&graph);
    for _ in 0..3 {
        graph.finalize_importer_order();
        assert_eq!(raw_nodes(&graph), first);
    }
}

#[test]
fn large_interleaved_vectors_preserve_order_across_buffer_reuse() {
    let mut graph = graph(66);
    let mut expected = Vec::new();
    let mut expected_components = Vec::new();
    for source in 2..66 {
        let id = FileId::new(source);
        for target in [FileId::new(0), FileId::new(1)] {
            graph.add_edge(id, target, DependencyEdge::ComponentUsage);
            graph.add_edge(id, target, DependencyEdge::Import);
        }
        expected.push((id, DependencyEdge::Import));
        expected_components.push((id, DependencyEdge::ComponentUsage));
    }
    expected.extend(expected_components);
    graph.finalize_importer_order();
    for target in [FileId::new(0), FileId::new(1)] {
        assert_eq!(
            graph.get_node(target).unwrap().importers.as_slice(),
            expected.as_slice()
        );
    }
    let first = raw_nodes(&graph);
    graph.finalize_importer_order();
    assert_eq!(raw_nodes(&graph), first);
}

#[test]
fn empty_or_already_grouped_graphs_preserve_complete_raw_nodes() {
    let mut empty = graph(0);
    empty.finalize_importer_order();
    assert!(raw_nodes(&empty).is_empty());
    let mut graph = graph(4);
    graph.add_edge(FileId::new(1), FileId::new(0), DependencyEdge::TypeImport);
    graph.add_edge(FileId::new(2), FileId::new(0), DependencyEdge::Import);
    graph.add_edge(
        FileId::new(3),
        FileId::new(0),
        DependencyEdge::ComponentUsage,
    );
    let original = raw_nodes(&graph);
    graph.finalize_importer_order();
    assert_eq!(raw_nodes(&graph), original);
}
