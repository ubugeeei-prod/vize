//! Returned slot roots own forwarding/fallback boundaries in the Vapor runtime.

use super::{
    BlockIRNode, ComponentKind, IRSlot, IRSlotControl, NegativeBranch, OperationNode, RootIRNode,
};
use vize_carton::{FxHashMap, ensure_sufficient_stack};

struct Layout<'a> {
    templates: &'a [&'a str],
    elements: &'a FxHashMap<usize, usize>,
}

impl RootIRNode<'_> {
    /// Both lowering lanes finish the same explicit slot-boundary contract.
    pub(crate) fn prepare_slot_roots(&mut self) {
        let layout = Layout {
            templates: &self.templates,
            elements: &self.element_template_map,
        };
        visit_block(&mut self.block, &layout, false);
    }
}

fn operation_id(operation: &OperationNode<'_>) -> Option<usize> {
    match operation {
        OperationNode::CreateComponent(node) => Some(node.id),
        OperationNode::SlotOutlet(node) => Some(node.id),
        OperationNode::If(node) => Some(node.id),
        OperationNode::For(node) => Some(node.id),
        OperationNode::Key(node) => Some(node.id),
        _ => None,
    }
}

// Matches compiler-vapor's anchored comment-only template pattern. Its
// wildcard spans intermediate comments too; preserve that conservative ABI.
fn comments_only(text: &str) -> bool {
    let text = text.trim();
    text.len() >= 7 && text.starts_with("<!--") && text.ends_with("-->")
}

/// Any returned guaranteed content stabilizes the complete slot, even beside
/// a conditional or forwarded outlet. Whitespace text is content; comments are not.
fn stable(block: &BlockIRNode<'_>, layout: &Layout<'_>) -> bool {
    ensure_sufficient_stack(|| {
        block.returns.iter().any(|id| {
            if let Some(operation) = block
                .operation
                .iter()
                .find(|operation| operation_id(operation) == Some(*id))
            {
                return match operation {
                    OperationNode::CreateComponent(node) => {
                        node.kind != ComponentKind::Dynamic
                            || node.is_expr.as_ref().is_some_and(|is| is.is_static)
                    }
                    OperationNode::Key(node) => stable(&node.render, layout),
                    _ => false,
                };
            }
            layout
                .elements
                .get(id)
                .and_then(|index| layout.templates.get(*index))
                .is_some_and(|text| !text.is_empty() && !comments_only(text))
        })
    })
}

fn mark(block: &mut BlockIRNode<'_>, layout: &Layout<'_>, shared: bool) {
    ensure_sufficient_stack(|| {
        if stable(block, layout) {
            return;
        }
        let shared = shared
            || block
                .operation
                .iter()
                .filter(|operation| {
                    operation_id(operation).is_some_and(|id| block.returns.contains(&id))
                })
                .take(2)
                .count()
                > 1;
        for operation in &mut block.operation {
            if !operation_id(operation).is_some_and(|id| block.returns.contains(&id)) {
                continue;
            }
            match operation {
                OperationNode::SlotOutlet(node) => node.flags |= 4 | if shared { 8 } else { 0 },
                OperationNode::If(node) => {
                    mark(&mut node.positive, layout, shared);
                    mark_negative(node.negative.as_mut(), layout, shared);
                }
                OperationNode::For(node) => mark(&mut node.render, layout, true),
                OperationNode::Key(node) => mark(&mut node.render, layout, shared),
                _ => {}
            }
        }
    });
}

fn mark_negative(negative: Option<&mut NegativeBranch<'_>>, layout: &Layout<'_>, shared: bool) {
    ensure_sufficient_stack(|| match negative {
        Some(NegativeBranch::Block(block)) => mark(block, layout, shared),
        Some(NegativeBranch::If(node)) => {
            mark(&mut node.positive, layout, shared);
            mark_negative(node.negative.as_mut(), layout, shared);
        }
        None => {}
    });
}

fn visit_slot(slot: &mut IRSlot<'_>, layout: &Layout<'_>) {
    ensure_sufficient_stack(|| {
        slot.nonstable = !stable(&slot.block, layout);
        if slot.nonstable {
            mark(&mut slot.block, layout, false);
        }
        visit_block(&mut slot.block, layout, true);
        if let Some(IRSlotControl::If {
            negative: Some(next),
            ..
        }) = &mut slot.control
        {
            visit_slot(next, layout);
        }
    });
}

fn visit_negative(negative: Option<&mut NegativeBranch<'_>>, layout: &Layout<'_>, in_slot: bool) {
    ensure_sufficient_stack(|| match negative {
        Some(NegativeBranch::Block(block)) => visit_block(block, layout, in_slot),
        Some(NegativeBranch::If(node)) => {
            visit_block(&mut node.positive, layout, in_slot);
            visit_negative(node.negative.as_mut(), layout, in_slot);
        }
        None => {}
    });
}

fn visit_block(block: &mut BlockIRNode<'_>, layout: &Layout<'_>, in_slot: bool) {
    ensure_sufficient_stack(|| {
        for operation in &mut block.operation {
            match operation {
                OperationNode::CreateComponent(node) => {
                    for slot in &mut node.slots {
                        visit_slot(slot, layout);
                    }
                }
                OperationNode::SlotOutlet(node) => {
                    if let Some(fallback) = &mut node.fallback {
                        if in_slot {
                            mark(fallback, layout, false);
                        }
                        visit_block(fallback, layout, in_slot);
                    }
                }
                OperationNode::If(node) => {
                    visit_block(&mut node.positive, layout, in_slot);
                    visit_negative(node.negative.as_mut(), layout, in_slot);
                }
                OperationNode::For(node) => visit_block(&mut node.render, layout, in_slot),
                OperationNode::Key(node) => visit_block(&mut node.render, layout, in_slot),
                _ => {}
            }
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn comment_template_pattern_matches_the_primary_compiler_boundary() {
        for (text, expected) in [
            ("", false),
            ("  ", false),
            ("<!-->", false),
            ("<!--->", false),
            ("<!---->", true),
            ("<!--x-->", true),
            ("  <!--x-->\n", true),
            ("<!--x--> <!--y-->", true),
            ("<!--x--><b></b><!--y-->", true),
            ("<!--x--><b>", false),
            ("<b><!--y-->", false),
        ] {
            assert_eq!(super::comments_only(text), expected, "{text}");
        }
    }
}
