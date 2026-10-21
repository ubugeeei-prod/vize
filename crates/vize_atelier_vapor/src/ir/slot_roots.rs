//! Returned slot roots own forwarding/fallback boundaries in the Vapor runtime.

use super::{
    BlockIRNode, ComponentKind, IRSlot, IRSlotControl, NegativeBranch, OperationNode, RootIRNode,
};
use vize_carton::{FxHashMap, ensure_sufficient_stack};

/// Registered element templates, used to decide whether a returned root is content.
pub(crate) struct Layout<'a> {
    templates: &'a [&'a str],
    elements: &'a FxHashMap<usize, usize>,
}

impl<'a> Layout<'a> {
    pub(crate) fn new(templates: &'a [&'a str], elements: &'a FxHashMap<usize, usize>) -> Self {
        Self {
            templates,
            elements,
        }
    }
}

impl RootIRNode<'_> {
    /// The native lane finishes slot boundaries in one pass over its emitted IR.
    /// The lowering lane finishes each slot and in-slot fallback as it is built.
    pub(crate) fn prepare_slot_roots(&mut self) {
        let layout = Layout::new(&self.templates, &self.element_template_map);
        visit_block(&mut self.block, &layout, false);
    }
}

/// Finish a complete component slot and its conditional alternatives.
///
/// Stack guards sit only on nested recursion: flat forwarding slots stay cheap.
pub(crate) fn finish_slot(mut slot: &mut IRSlot<'_>, layout: &Layout<'_>) {
    loop {
        finish_slot_root(slot, layout);
        match &mut slot.control {
            Some(IRSlotControl::If {
                negative: Some(next),
                ..
            }) => slot = next,
            _ => return,
        }
    }
}

/// Finish an outlet fallback that is rendered inside component slot content.
pub(crate) fn finish_slot_fallback(fallback: &mut BlockIRNode<'_>, layout: &Layout<'_>) {
    mark(fallback, layout, false);
}

fn finish_slot_root(slot: &mut IRSlot<'_>, layout: &Layout<'_>) {
    slot.nonstable = !stable(&slot.block, layout);
    if slot.nonstable {
        mark_returns(&mut slot.block, layout, false);
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
                OperationNode::Key(node) => {
                    ensure_sufficient_stack(|| stable(&node.render, layout))
                }
                _ => false,
            };
        }
        layout
            .elements
            .get(id)
            .and_then(|index| layout.templates.get(*index))
            .is_some_and(|text| !text.is_empty() && !comments_only(text))
    })
}

fn mark(block: &mut BlockIRNode<'_>, layout: &Layout<'_>, shared: bool) {
    if !stable(block, layout) {
        mark_returns(block, layout, shared);
    }
}

fn mark_returns(block: &mut BlockIRNode<'_>, layout: &Layout<'_>, shared: bool) {
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
            OperationNode::If(node) => ensure_sufficient_stack(|| {
                mark(&mut node.positive, layout, shared);
                mark_negative(node.negative.as_mut(), layout, shared);
            }),
            OperationNode::For(node) => {
                ensure_sufficient_stack(|| mark(&mut node.render, layout, true));
            }
            OperationNode::Key(node) => {
                ensure_sufficient_stack(|| mark(&mut node.render, layout, shared));
            }
            _ => {}
        }
    }
}

fn mark_negative(mut negative: Option<&mut NegativeBranch<'_>>, layout: &Layout<'_>, shared: bool) {
    loop {
        match negative {
            Some(NegativeBranch::Block(block)) => return mark(block, layout, shared),
            Some(NegativeBranch::If(node)) => {
                mark(&mut node.positive, layout, shared);
                negative = node.negative.as_mut();
            }
            None => return,
        }
    }
}

fn visit_slot(slot: &mut IRSlot<'_>, layout: &Layout<'_>) {
    ensure_sufficient_stack(|| {
        finish_slot_root(slot, layout);
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
