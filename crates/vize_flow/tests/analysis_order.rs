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
