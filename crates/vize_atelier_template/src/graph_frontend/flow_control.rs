use vize_flow::{
    BlockId, ControlEdgeKind, FlowGraph, FlowResult, NodeKind, Provenance, TerminatorKind,
};

pub(super) fn add_jump(
    graph: &mut FlowGraph,
    from: BlockId,
    to: BlockId,
    kind: ControlEdgeKind,
    provenance: Provenance,
) -> FlowResult<()> {
    graph.add_node(from, NodeKind::Terminator(TerminatorKind::Jump), provenance)?;
    graph.add_control_edge(from, to, kind, provenance)?;
    Ok(())
}
