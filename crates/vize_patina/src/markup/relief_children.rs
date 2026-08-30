use super::{MarkupElement, MarkupElementInner};
use vize_relief::TemplateChildNode;

pub(super) fn walk_relief_children<'a>(
    children: &'a [TemplateChildNode<'a>],
    enter: &mut impl FnMut(MarkupElement<'a>),
    exit: &mut impl FnMut(MarkupElement<'a>),
) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                let element = MarkupElement::new(element);
                enter(element);
                walk_relief_children(element_children(element), enter, exit);
                exit(element);
            }
            TemplateChildNode::If(if_node) => {
                for branch in if_node.branches.iter() {
                    walk_relief_children(&branch.children, enter, exit);
                }
            }
            TemplateChildNode::For(for_node) => {
                walk_relief_children(&for_node.children, enter, exit);
            }
            _ => {}
        }
    }
}

fn element_children<'a>(element: MarkupElement<'a>) -> &'a [TemplateChildNode<'a>] {
    match element.inner {
        MarkupElementInner::Relief(node) => &node.children,
        _ => &[],
    }
}
