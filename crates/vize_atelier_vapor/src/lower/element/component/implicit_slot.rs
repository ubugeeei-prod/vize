//! Keep ordinary component children beside named slot carriers.

use super::{TransformContext, structural_slots};
use crate::ir::{BlockIRNode, IRSlot};
use crate::lower::transform_children;
use vize_atelier_core::{SimpleExpressionNode, SourceLocation, TemplateChildNode};
use vize_carton::Box;

pub(super) fn is_meaningful(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Comment(_) => false,
        TemplateChildNode::Text(text) => !text.content.trim().is_empty(),
        _ => !structural_slots::is_slot(child),
    }
}

pub(super) fn lower<'a>(
    ctx: &mut TransformContext<'a>,
    children: &[TemplateChildNode<'a>],
) -> IRSlot<'a> {
    let mut block = BlockIRNode::new(ctx.allocator);
    // Borrow contiguous runs: neither clone AST nodes nor lower named carriers
    // into the default slot. Preserve ordinary children in authored order.
    for run in children.split(|child| {
        structural_slots::is_slot(child) || matches!(child, TemplateChildNode::Comment(_))
    }) {
        if !run.is_empty() {
            let lowered = transform_children(ctx, run);
            block.operation.extend(lowered.operation);
            block.effect.extend(lowered.effect);
            block.returns.extend(lowered.returns);
        }
    }
    IRSlot {
        name: Box::new_in(
            SimpleExpressionNode::new("default", true, SourceLocation::STUB),
            &ctx.allocator,
        ),
        fn_exp: None,
        control: None,
        block,
    }
}
