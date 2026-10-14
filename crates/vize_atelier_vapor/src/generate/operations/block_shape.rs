//! Block shapes the Vapor runtime reads from `createIf` and `createFor`
//! flags, as Vue's compiler encodes them since 3.6.0-rc.6. Hydration uses
//! them to claim each branch and row as one node or as a fragment.

use crate::ir::{BlockIRNode, ComponentKind, IfIRNode, NegativeBranch, OperationNode};
use vize_carton::FxHashMap;

use super::super::context::GenerateContext;

const SINGLE_ROOT: u32 = 1;
const MULTI_ROOT: u32 = 2;

/// The operation that creates the block's sole returned node, if any.
fn sole_operation<'b, 'a>(block: &'b BlockIRNode<'a>) -> Option<&'b OperationNode<'a>> {
    let [id] = block.returns.as_slice() else {
        return None;
    };
    block.operation.iter().find(|operation| match operation {
        OperationNode::If(node) => node.id == *id,
        OperationNode::For(node) => node.id == *id,
        OperationNode::CreateComponent(node) => node.id == *id,
        OperationNode::SlotOutlet(node) => node.id == *id,
        OperationNode::Key(node) => node.id == *id,
        _ => false,
    })
}

/// Whether the block's sole returned node is a text template.
fn returns_text(
    ctx: &GenerateContext<'_>,
    block: &BlockIRNode<'_>,
    templates: &FxHashMap<usize, usize>,
) -> bool {
    matches!(block.returns.as_slice(), [id] if templates
        .get(id)
        .and_then(|index| ctx.templates.get(*index))
        .is_some_and(|html| !html.starts_with('<')))
}

/// One root node, or a fragment of several (or none, or text).
fn shape(
    ctx: &GenerateContext<'_>,
    block: &BlockIRNode<'_>,
    templates: &FxHashMap<usize, usize>,
) -> u32 {
    if block.returns.len() != 1
        || matches!(sole_operation(block), Some(OperationNode::If(_)))
        || returns_text(ctx, block, templates)
    {
        MULTI_ROOT
    } else {
        SINGLE_ROOT
    }
}

/// The `createIf` flags, or `None` for the runtime default (one root, no
/// false branch). `force_multi_root` marks a branch that is a loop row.
pub(super) fn if_flags(
    ctx: &GenerateContext<'_>,
    if_node: &IfIRNode<'_>,
    templates: &FxHashMap<usize, usize>,
    force_multi_root: bool,
    slot_root: bool,
) -> Option<IfFlags> {
    let mut flags = if force_multi_root {
        MULTI_ROOT | MULTI_ROOT << 2
    } else {
        let negative = match &if_node.negative {
            None => 0,
            Some(NegativeBranch::If(_)) => SINGLE_ROOT,
            Some(NegativeBranch::Block(block)) => shape(ctx, block, templates),
        };
        shape(ctx, &if_node.positive, templates) | negative << 2
    };
    if slot_root {
        flags |= 128;
    }
    if if_node.once {
        flags |= 16;
    }
    (flags != SINGLE_ROOT).then_some(IfFlags(flags))
}

/// `createIf` flags other than the runtime default.
#[derive(Clone, Copy)]
pub(super) struct IfFlags(u32);

impl IfFlags {
    /// Writes the flags with their names: `6 /* TRUE_MULTI_ROOT, FALSE_SINGLE_ROOT */`.
    pub(super) fn write(self, ctx: &mut GenerateContext<'_>) {
        use std::fmt::Write as _;
        let flags = self.0;
        let _ = write!(ctx, "{flags} /* TRUE_{}", shape_name(flags));
        if flags >> 2 & 3 != 0 {
            let _ = write!(ctx, ", FALSE_{}", shape_name(flags >> 2));
        }
        if flags & 16 != 0 {
            ctx.push(", ONCE");
        }
        if flags & 128 != 0 {
            ctx.push(", SLOT_ROOT");
        }
        ctx.push(" */");
    }
}

fn shape_name(flags: u32) -> &'static str {
    match flags & 3 {
        0 => "EMPTY",
        SINGLE_ROOT => "SINGLE_ROOT",
        _ => "MULTI_ROOT",
    }
}

/// Whether each loop row is one node created from a template.
pub(super) fn single_node_rows(
    block: &BlockIRNode<'_>,
    templates: &FxHashMap<usize, usize>,
) -> bool {
    matches!(block.returns.as_slice(), [id] if templates.contains_key(id))
}

/// Whether each loop row is a fragment the runtime moves as one unit.
pub(super) fn fragment_rows(block: &BlockIRNode<'_>) -> bool {
    match sole_operation(block) {
        Some(OperationNode::If(node)) => !node.once,
        Some(OperationNode::For(_) | OperationNode::SlotOutlet(_) | OperationNode::Key(_)) => true,
        Some(OperationNode::CreateComponent(node)) => {
            node.kind == ComponentKind::Dynamic
                && node
                    .is_expr
                    .as_ref()
                    .is_none_or(|expression| !expression.is_static)
        }
        _ => false,
    }
}
