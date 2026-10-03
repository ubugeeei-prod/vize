//! Keep ordinary component children beside named slot carriers.

use super::{TransformContext, structural_slots};
use crate::ir::{BlockIRNode, IRSlot};
use crate::lower::transform_children_into;
use vize_atelier_core::{SimpleExpressionNode, SourceLocation, TemplateChildNode};
use vize_carton::Box;

pub(super) fn lower<'a>(
    ctx: &mut TransformContext<'a>,
    children: &[TemplateChildNode<'a>],
) -> Option<IRSlot<'a>> {
    let meaningful = children.iter().any(|child| match child {
        TemplateChildNode::Comment(_) => false,
        TemplateChildNode::Text(text) => !text.content.trim().is_empty(),
        _ => !structural_slots::is_slot(child),
    });
    if !meaningful {
        return None;
    }

    let mut block = BlockIRNode::new(ctx.allocator);
    // Borrow contiguous runs: neither clone AST nodes nor lower named carriers
    // into the default slot. Preserve ordinary children in authored order.
    for run in children.split(structural_slots::is_slot) {
        if !run.is_empty() {
            transform_children_into(ctx, run, &mut block);
        }
    }
    Some(IRSlot {
        name: Box::new_in(
            SimpleExpressionNode::new("default", true, SourceLocation::STUB),
            &ctx.allocator,
        ),
        fn_exp: None,
        control: None,
        block,
    })
}
