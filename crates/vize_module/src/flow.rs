use vize_carton::source_range::SourceRange;
use vize_flow::{
    BlockId, ControlEdgeKind, FlowGraph, FlowResult, NodeKind, Provenance, SourceId, TerminatorKind,
};

use crate::{
    ModuleBlock, ModuleDocument, ModuleEdgeKind, ModuleInstructionKind, ModuleSpan, ModuleSyntax,
};

pub fn project_module_flow(document: &ModuleDocument) -> FlowResult<FlowGraph> {
    let mut graph = FlowGraph::new();
    append_module_flow(document, &mut graph)?;
    debug_assert!(graph.validate().is_ok());
    Ok(graph)
}

pub fn append_module_flow(
    document: &ModuleDocument,
    graph: &mut FlowGraph,
) -> FlowResult<Vec<BlockId>> {
    let mut entries = Vec::with_capacity(document.modules.len());
    for (index, module) in document.modules.iter().enumerate() {
        entries.push(append_one(
            module,
            graph,
            index == 0 && graph.blocks().len() == 1,
        )?);
    }
    Ok(entries)
}

fn append_one(
    module: &ModuleSyntax,
    graph: &mut FlowGraph,
    reuse_entry: bool,
) -> FlowResult<BlockId> {
    let source = add_source(module, graph)?;
    let mut blocks = Vec::with_capacity(module.cfg.blocks.len());
    for (index, block) in module.cfg.blocks.iter().enumerate() {
        let provenance = provenance(source, block.span);
        let id = if reuse_entry && index == module.cfg.entry {
            graph.set_block_provenance(graph.entry_block(), provenance)?;
            graph.entry_block()
        } else {
            graph.add_block(provenance)?
        };
        append_instructions(block, id, source, graph)?;
        blocks.push(id);
    }
    let entry = blocks
        .get(module.cfg.entry)
        .copied()
        .unwrap_or_else(|| graph.entry_block());
    if !reuse_entry {
        graph.add_control_edge(
            graph.entry_block(),
            entry,
            ControlEdgeKind::Normal,
            Provenance::Synthetic,
        )?;
    }
    for edge in &module.cfg.edges {
        let (Some(from), Some(to)) = (blocks.get(edge.from), blocks.get(edge.to)) else {
            continue;
        };
        graph.add_control_edge(
            *from,
            *to,
            control_kind(edge.kind),
            provenance(source, edge.span),
        )?;
    }
    Ok(entry)
}

fn add_source(module: &ModuleSyntax, graph: &mut FlowGraph) -> FlowResult<SourceId> {
    match module.source_anchor {
        Some(anchor) => graph.add_source_with_anchor(module.name.as_ref(), anchor),
        None => graph.add_source(module.name.as_ref()),
    }
}

fn append_instructions(
    block: &ModuleBlock,
    id: BlockId,
    source: SourceId,
    graph: &mut FlowGraph,
) -> FlowResult<()> {
    for instruction in &block.instructions {
        graph.add_node(
            id,
            node_kind(instruction.kind),
            provenance(source, instruction.span.or(block.span)),
        )?;
    }
    Ok(())
}

const fn node_kind(kind: ModuleInstructionKind) -> NodeKind {
    match kind {
        ModuleInstructionKind::Condition | ModuleInstructionKind::Iteration => {
            NodeKind::Terminator(TerminatorKind::Branch)
        }
        ModuleInstructionKind::Return => NodeKind::Terminator(TerminatorKind::Return),
        ModuleInstructionKind::Throw => NodeKind::Terminator(TerminatorKind::Throw),
        ModuleInstructionKind::Break | ModuleInstructionKind::Continue => {
            NodeKind::Terminator(TerminatorKind::Jump)
        }
        ModuleInstructionKind::Unreachable => NodeKind::Terminator(TerminatorKind::Unreachable),
        ModuleInstructionKind::Operation => NodeKind::Operation,
    }
}

const fn control_kind(kind: ModuleEdgeKind) -> ControlEdgeKind {
    match kind {
        ModuleEdgeKind::TrueBranch => ControlEdgeKind::TrueBranch,
        ModuleEdgeKind::FalseBranch => ControlEdgeKind::FalseBranch,
        ModuleEdgeKind::LoopBack => ControlEdgeKind::LoopBack,
        ModuleEdgeKind::Return => ControlEdgeKind::Return,
        ModuleEdgeKind::Break => ControlEdgeKind::Break,
        ModuleEdgeKind::Continue => ControlEdgeKind::Continue,
        ModuleEdgeKind::Exception => ControlEdgeKind::Exception,
        ModuleEdgeKind::Function => ControlEdgeKind::FunctionEntry,
        ModuleEdgeKind::Unreachable => ControlEdgeKind::Unreachable,
        ModuleEdgeKind::Normal => ControlEdgeKind::Normal,
    }
}

const fn provenance(source: SourceId, span: Option<ModuleSpan>) -> Provenance {
    match span {
        Some(span) => Provenance::source(source, SourceRange::new(span.start, span.end)),
        None => Provenance::Synthetic,
    }
}
