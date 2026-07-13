use vize_flow::{ControlEdgeKind, FlowGraph, Provenance};

#[test]
fn reverse_postorder_is_stable_and_omits_unreachable_blocks() {
    let mut graph = FlowGraph::new();
    let left = graph.add_block(Provenance::Synthetic).unwrap();
    let right = graph.add_block(Provenance::Synthetic).unwrap();
    let merge = graph.add_block(Provenance::Synthetic).unwrap();
    let _unreachable = graph.add_block(Provenance::Synthetic).unwrap();
    graph
        .add_control_edge(
            graph.entry_block(),
            left,
            ControlEdgeKind::TrueBranch,
            Provenance::Synthetic,
        )
        .unwrap();
    graph
        .add_control_edge(
            graph.entry_block(),
            right,
            ControlEdgeKind::FalseBranch,
            Provenance::Synthetic,
        )
        .unwrap();
    graph
        .add_control_edge(left, merge, ControlEdgeKind::Normal, Provenance::Synthetic)
        .unwrap();
    graph
        .add_control_edge(right, merge, ControlEdgeKind::Normal, Provenance::Synthetic)
        .unwrap();

    assert_eq!(
        graph.reverse_postorder(),
        [graph.entry_block(), right, left, merge]
    );
}

#[test]
fn declaration_and_impossible_edges_are_not_entry_execution_paths() {
    let mut graph = FlowGraph::new();
    let live = graph.add_block(Provenance::Synthetic).unwrap();
    let function = graph.add_block(Provenance::Synthetic).unwrap();
    let dead = graph.add_block(Provenance::Synthetic).unwrap();
    graph
        .add_control_edge(
            graph.entry_block(),
            live,
            ControlEdgeKind::Normal,
            Provenance::Synthetic,
        )
        .unwrap();
    graph
        .add_control_edge(
            graph.entry_block(),
            function,
            ControlEdgeKind::FunctionEntry,
            Provenance::Synthetic,
        )
        .unwrap();
    graph
        .add_control_edge(
            graph.entry_block(),
            dead,
            ControlEdgeKind::Unreachable,
            Provenance::Synthetic,
        )
        .unwrap();

    let reachable = graph.reachability();
    assert!(reachable.contains(live));
    assert!(!reachable.contains(function));
    assert!(!reachable.contains(dead));
    assert_eq!(graph.reverse_postorder(), [graph.entry_block(), live]);
    let dominators = graph.dominators();
    assert!(!dominators.is_reachable(function));
    assert!(!dominators.is_reachable(dead));
}
