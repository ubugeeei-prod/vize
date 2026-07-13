use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_cfg::graph::visit::EdgeRef;
use oxc_cfg::{EdgeType, InstructionKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType};
use vize_carton::source_anchor::SourceAnchor;

use crate::{
    ModuleBlock, ModuleCfg, ModuleDiagnostic, ModuleDocument, ModuleEdge, ModuleEdgeKind,
    ModuleInstruction, ModuleInstructionKind, ModuleLanguage, ModuleSpan, ModuleSyntax, facts,
};

pub fn snapshot_module(
    name: &str,
    source: &str,
    language: ModuleLanguage,
    base_offset: u32,
    source_anchor: Option<SourceAnchor>,
) -> ModuleSyntax {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type(language)).parse();
    snapshot_program(
        name,
        source,
        language,
        base_offset,
        source_anchor,
        &parsed.program,
        &parsed.errors,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn snapshot_program(
    name: &str,
    source: &str,
    language: ModuleLanguage,
    base_offset: u32,
    source_anchor: Option<SourceAnchor>,
    program: &Program<'_>,
    parse_errors: &[OxcDiagnostic],
) -> ModuleSyntax {
    let built = SemanticBuilder::new().with_cfg(true).build(program);
    let mut owned_diagnostics = diagnostics(parse_errors, source.len(), base_offset);
    owned_diagnostics.extend(diagnostics(&built.errors, source.len(), base_offset));
    let (imports, exports, declarations, references) =
        facts::collect(program, &built.semantic, base_offset);
    let operations = crate::operations::collect(program, source, base_offset);
    let cfg = built
        .semantic
        .cfg()
        .map(|cfg| cfg_snapshot(cfg, &built.semantic, base_offset))
        .unwrap_or_default();
    ModuleSyntax {
        name: name.into(),
        source: source.into(),
        language,
        base_offset,
        source_anchor,
        diagnostics: owned_diagnostics,
        imports,
        exports,
        declarations,
        references,
        operations,
        cfg,
    }
}

fn cfg_snapshot(
    cfg: &oxc_cfg::ControlFlowGraph,
    semantic: &oxc_semantic::Semantic<'_>,
    base: u32,
) -> ModuleCfg {
    let mut blocks = Vec::new();
    for node in cfg.graph.node_indices() {
        while blocks.len() < node.index() {
            blocks.push(ModuleBlock {
                instructions: Vec::new(),
                span: None,
            });
        }
        let instructions = cfg
            .basic_block(node)
            .instructions()
            .iter()
            .map(|instruction| instruction_snapshot(instruction, semantic, base))
            .collect::<Vec<_>>();
        let span = instructions
            .iter()
            .filter_map(|instruction| instruction.span)
            .reduce(join);
        blocks.push(ModuleBlock { instructions, span });
    }
    let edges = cfg
        .graph
        .edge_references()
        .map(|edge| ModuleEdge {
            from: edge.source().index(),
            to: edge.target().index(),
            kind: edge_kind(edge.weight(), blocks.get(edge.source().index())),
            span: blocks.get(edge.source().index()).and_then(edge_provenance),
        })
        .collect();
    ModuleCfg {
        entry: 0,
        blocks,
        edges,
    }
}

fn instruction_snapshot(
    instruction: &oxc_cfg::Instruction,
    semantic: &oxc_semantic::Semantic<'_>,
    base: u32,
) -> ModuleInstruction {
    let kind = match instruction.kind {
        InstructionKind::Condition => ModuleInstructionKind::Condition,
        InstructionKind::Iteration(_) => ModuleInstructionKind::Iteration,
        InstructionKind::Return(_) | InstructionKind::ImplicitReturn => {
            ModuleInstructionKind::Return
        }
        InstructionKind::Throw => ModuleInstructionKind::Throw,
        InstructionKind::Break(_) => ModuleInstructionKind::Break,
        InstructionKind::Continue(_) => ModuleInstructionKind::Continue,
        InstructionKind::Unreachable => ModuleInstructionKind::Unreachable,
        InstructionKind::Statement => ModuleInstructionKind::Operation,
    };
    let span = instruction
        .node_id
        .map(|node| facts::absolute(semantic.nodes().get_node(node).span(), base));
    ModuleInstruction { kind, span }
}

fn edge_kind(edge: &EdgeType, block: Option<&ModuleBlock>) -> ModuleEdgeKind {
    match terminal(block) {
        Some(ModuleInstructionKind::Return) => return ModuleEdgeKind::Return,
        Some(ModuleInstructionKind::Throw) => return ModuleEdgeKind::Exception,
        Some(ModuleInstructionKind::Break) => return ModuleEdgeKind::Break,
        Some(ModuleInstructionKind::Continue) => return ModuleEdgeKind::Continue,
        _ => {}
    }
    match edge {
        EdgeType::Jump => ModuleEdgeKind::TrueBranch,
        EdgeType::Backedge => ModuleEdgeKind::LoopBack,
        EdgeType::NewFunction => ModuleEdgeKind::Function,
        EdgeType::Finalize | EdgeType::Error(_) => ModuleEdgeKind::Exception,
        EdgeType::Unreachable => ModuleEdgeKind::Unreachable,
        EdgeType::Join => ModuleEdgeKind::Normal,
        EdgeType::Normal => match terminal(block) {
            Some(ModuleInstructionKind::Condition | ModuleInstructionKind::Iteration) => {
                ModuleEdgeKind::FalseBranch
            }
            _ => ModuleEdgeKind::Normal,
        },
    }
}

fn terminal(block: Option<&ModuleBlock>) -> Option<ModuleInstructionKind> {
    block?
        .instructions
        .last()
        .map(|instruction| instruction.kind)
}

fn edge_provenance(block: &ModuleBlock) -> Option<ModuleSpan> {
    block
        .instructions
        .iter()
        .rev()
        .find_map(|instruction| instruction.span)
        .or(block.span)
}

fn join(left: ModuleSpan, right: ModuleSpan) -> ModuleSpan {
    ModuleSpan::new(left.start.min(right.start), left.end.max(right.end))
}

fn diagnostics(errors: &[OxcDiagnostic], len: usize, base: u32) -> Vec<ModuleDiagnostic> {
    errors
        .iter()
        .map(|error| {
            let (start, end) = error
                .labels
                .as_ref()
                .and_then(|labels| labels.iter().find(|label| label.primary()))
                .or_else(|| error.labels.as_ref().and_then(|labels| labels.first()))
                .map(|label| {
                    let start = label.offset().min(len) as u32;
                    let end = start.saturating_add(label.len().max(1) as u32);
                    (start, end)
                })
                .unwrap_or((0, len.min(u32::MAX as usize) as u32));
            ModuleDiagnostic {
                message: error.message.clone().into_owned().into_boxed_str(),
                span: ModuleSpan::new(base.saturating_add(start), base.saturating_add(end)),
            }
        })
        .collect()
}

pub(crate) const fn source_type(language: ModuleLanguage) -> SourceType {
    match language {
        ModuleLanguage::JavaScript => SourceType::mjs(),
        ModuleLanguage::TypeScript => SourceType::ts().with_module(true),
        ModuleLanguage::Jsx => SourceType::jsx().with_module(true),
        ModuleLanguage::Tsx => SourceType::tsx().with_module(true),
    }
}

pub(crate) fn one(module: ModuleSyntax) -> ModuleDocument {
    ModuleDocument::from_module(module)
}
