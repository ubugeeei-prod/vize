use std::error::Error;

use vize_flow::{BlockId, ControlEdgeKind, FlowGraph, NodeKind, Provenance, TerminatorKind};

#[test]
fn branching_has_stable_reachability_and_dominance() -> Result<(), Box<dyn Error>> {
    let mut graph = FlowGraph::new();
    let entry = graph.entry_block();
    let then_block = graph.add_block(Provenance::Synthetic)?;
    let else_block = graph.add_block(Provenance::Synthetic)?;
    let merge = graph.add_block(Provenance::Synthetic)?;
    let exit = graph.add_block(Provenance::Synthetic)?;

    graph.add_node(
        entry,
        NodeKind::Terminator(TerminatorKind::Branch),
        Provenance::Synthetic,
    )?;
    graph.add_node(then_block, NodeKind::Operation, Provenance::Synthetic)?;
    graph.add_node(else_block, NodeKind::Operation, Provenance::Synthetic)?;
    graph.add_node(merge, NodeKind::Merge, Provenance::Synthetic)?;
    graph.add_node(
        merge,
        NodeKind::Terminator(TerminatorKind::Return),
        Provenance::Synthetic,
    )?;

    graph.add_control_edge(
        entry,
        then_block,
        ControlEdgeKind::TrueBranch,
        Provenance::Synthetic,
    )?;
    graph.add_control_edge(
        entry,
        else_block,
        ControlEdgeKind::FalseBranch,
        Provenance::Synthetic,
    )?;
    graph.add_control_edge(
        then_block,
        merge,
        ControlEdgeKind::Normal,
        Provenance::Synthetic,
    )?;
    graph.add_control_edge(
        else_block,
        merge,
        ControlEdgeKind::Normal,
        Provenance::Synthetic,
    )?;
    graph.add_control_edge(merge, exit, ControlEdgeKind::Return, Provenance::Synthetic)?;

    let reachable = graph.reachability();
    assert_eq!(
        reachable.blocks().collect::<Vec<_>>(),
        vec![entry, then_block, else_block, merge, exit]
    );

    let dominators = graph.dominators();
    assert!(dominators.dominates(entry, merge));
    assert!(!dominators.dominates(then_block, merge));
    assert!(!dominators.dominates(else_block, merge));
    assert_eq!(dominators.immediate_dominator(merge), Some(entry));
    assert_eq!(dominators.immediate_dominator(exit), Some(merge));
    graph.validate()?;
    Ok(())
}

#[test]
fn loop_back_edges_preserve_loop_dominance() -> Result<(), Box<dyn Error>> {
    let mut graph = FlowGraph::new();
    let entry = graph.entry_block();
    let header = graph.add_block(Provenance::Synthetic)?;
    let body = graph.add_block(Provenance::Synthetic)?;
    let exit = graph.add_block(Provenance::Synthetic)?;

    graph.add_control_edge(
        entry,
        header,
        ControlEdgeKind::Normal,
        Provenance::Synthetic,
    )?;
    graph.add_control_edge(
        header,
        body,
        ControlEdgeKind::TrueBranch,
        Provenance::Synthetic,
    )?;
    graph.add_control_edge(
        header,
        exit,
        ControlEdgeKind::FalseBranch,
        Provenance::Synthetic,
    )?;
    graph.add_control_edge(
        body,
        header,
        ControlEdgeKind::LoopBack,
        Provenance::Synthetic,
    )?;

    let header_predecessors: Vec<_> = graph
        .predecessor_edges(header)?
        .map(|edge| (edge.from(), edge.kind()))
        .collect();
    assert_eq!(
        header_predecessors,
        vec![
            (entry, ControlEdgeKind::Normal),
            (body, ControlEdgeKind::LoopBack),
        ]
    );

    let dominators = graph.dominators();
    assert_eq!(dominators.immediate_dominator(header), Some(entry));
    assert_eq!(dominators.immediate_dominator(body), Some(header));
    assert_eq!(dominators.immediate_dominator(exit), Some(header));
    assert!(dominators.dominates(header, body));
    assert!(dominators.dominates(header, exit));
    graph.validate()?;
    Ok(())
}

#[test]
fn unreachable_blocks_are_excluded_from_analysis() -> Result<(), Box<dyn Error>> {
    let mut graph = FlowGraph::new();
    let entry = graph.entry_block();
    let live = graph.add_block(Provenance::Synthetic)?;
    let dead = graph.add_block(Provenance::Synthetic)?;
    let dead_successor = graph.add_block(Provenance::Synthetic)?;

    graph.add_control_edge(entry, live, ControlEdgeKind::Normal, Provenance::Synthetic)?;
    graph.add_control_edge(
        dead,
        dead_successor,
        ControlEdgeKind::Exception,
        Provenance::Synthetic,
    )?;

    let reachable = graph.reachability();
    assert!(reachable.contains(entry));
    assert!(reachable.contains(live));
    assert!(!reachable.contains(dead));
    assert!(!reachable.contains(dead_successor));
    assert!(!reachable.contains(BlockId::from_raw(u32::MAX)));

    let dominators = graph.dominators();
    assert!(!dominators.dominates(dead, dead));
    assert_eq!(dominators.immediate_dominator(dead_successor), None);
    assert_eq!(dominators.dominators_of(dead).count(), 0);
    graph.validate()?;
    Ok(())
}

#[test]
fn predecessor_and_successor_indexes_agree() -> Result<(), Box<dyn Error>> {
    let mut graph = FlowGraph::new();
    let entry = graph.entry_block();
    let normal_exit = graph.add_block(Provenance::Synthetic)?;
    let handler = graph.add_block(Provenance::Synthetic)?;
    graph.add_control_edge(
        entry,
        normal_exit,
        ControlEdgeKind::Return,
        Provenance::Synthetic,
    )?;
    graph.add_control_edge(
        entry,
        handler,
        ControlEdgeKind::Exception,
        Provenance::Synthetic,
    )?;

    for edge in graph.control_edges() {
        assert_eq!(
            graph
                .successor_edges(edge.from())?
                .filter(|candidate| candidate.id() == edge.id())
                .count(),
            1
        );
        assert_eq!(
            graph
                .predecessor_edges(edge.to())?
                .filter(|candidate| candidate.id() == edge.id())
                .count(),
            1
        );
    }
    graph.validate()?;
    Ok(())
}
