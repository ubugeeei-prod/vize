use vize_relief::{ElementNode, TemplateChildNode};

/// End offset of an element's full subtree (including children) in the source.
/// `ElementNode::loc` only covers the opening tag, so v-for / v-slot scopes
/// that should extend over the element's interior fall back to this helper.
pub(super) fn element_subtree_end(el: &ElementNode<'_>) -> u32 {
    el.children
        .last()
        .map(template_child_end)
        .unwrap_or(el.loc.end.offset)
}

fn template_child_end(child: &TemplateChildNode<'_>) -> u32 {
    match child {
        TemplateChildNode::Element(e) => element_subtree_end(e),
        TemplateChildNode::Text(n) => n.loc.end.offset,
        TemplateChildNode::Comment(n) => n.loc.end.offset,
        TemplateChildNode::Interpolation(n) => n.loc.end.offset,
        TemplateChildNode::If(n) => n.loc.end.offset,
        TemplateChildNode::IfBranch(n) => n.loc.end.offset,
        TemplateChildNode::For(n) => n.loc.end.offset,
        TemplateChildNode::TextCall(n) => n.loc.end.offset,
        TemplateChildNode::CompoundExpression(n) => n.loc.end.offset,
        TemplateChildNode::Hoisted(_) => 0,
    }
}
