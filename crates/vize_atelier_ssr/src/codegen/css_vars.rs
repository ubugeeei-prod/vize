//! Generated CSS-variable binds for roots that cannot consume `_attrs`.
//!
//! The single fallthrough root already receives the premerged CSS variables.
//! Other physical roots need their own bind, as Vue's SSR root transform does.
//! Only root lists and conditional branches are visited, never ordinary
//! descendants or `v-for` bodies. Generated binds have no authored span.

use vize_atelier_core::{
    DirectiveNode, ElementType, ExpressionNode, PropNode, RootNode, SimpleExpressionNode,
    SourceLocation, TemplateChildNode,
};
use vize_l0::{Allocator, Box};

use super::helpers::{branch_fragment, single_fallthrough_child_index};

pub(crate) fn root_is_fragment(children: &[TemplateChildNode<'_>]) -> bool {
    let mut rendered = 0;
    let mut non_text = false;
    for child in children {
        if matches!(child, TemplateChildNode::Comment(comment) if comment.directive.is_some()) {
            continue;
        }
        rendered += 1;
        non_text |= !matches!(child, TemplateChildNode::Text(_));
    }
    rendered > 1 && non_text
}

pub(crate) fn inject<'a>(allocator: &'a Allocator, root: &mut RootNode<'a>) {
    let fragment = root_is_fragment(&root.children);
    inject_children(allocator, &mut root.children, fragment, true);
}

fn inject_children<'a>(
    allocator: &'a Allocator,
    children: &mut [TemplateChildNode<'a>],
    fragment: bool,
    inherit_attrs: bool,
) {
    vize_l0::ensure_sufficient_stack(|| {
        inject_children_inner(allocator, children, fragment, inherit_attrs);
    });
}

fn inject_children_inner<'a>(
    allocator: &'a Allocator,
    children: &mut [TemplateChildNode<'a>],
    fragment: bool,
    inherit_attrs: bool,
) {
    let fallthrough = if inherit_attrs && !fragment {
        single_fallthrough_child_index(children)
    } else {
        None
    };
    for (index, child) in children.iter_mut().enumerate() {
        let inherit = fallthrough == Some(index);
        match child {
            TemplateChildNode::If(node) => {
                for branch in &mut node.branches {
                    let fragment = branch_fragment::needs_fragment(&branch.children);
                    if branch.is_template_if
                        && let [TemplateChildNode::Element(wrapper)] =
                            branch.children.as_mut_slice()
                        && wrapper.tag_type == ElementType::Template
                    {
                        inject_children(
                            allocator,
                            &mut wrapper.children,
                            false,
                            inherit && !fragment,
                        );
                        continue;
                    }
                    inject_children(allocator, &mut branch.children, fragment, inherit);
                }
            }
            TemplateChildNode::Element(element) => match element.tag_type {
                ElementType::Component if matches!(element.tag, "Suspense" | "suspense") => {
                    for child in &mut element.children {
                        if let TemplateChildNode::Element(slot) = child
                            && slot.tag_type == ElementType::Template
                            && slot.props.iter().any(|prop| {
                                matches!(prop, PropNode::Directive(dir) if dir.name == "slot")
                            })
                        {
                            inject_children(allocator, &mut slot.children, false, false);
                        } else {
                            inject_children(allocator, core::slice::from_mut(child), false, false);
                        }
                    }
                }
                ElementType::Element | ElementType::Component if !inherit => {
                    let mut bind = DirectiveNode::new(allocator, "bind", SourceLocation::STUB);
                    bind.exp = Some(ExpressionNode::Simple(Box::new_in(
                        SimpleExpressionNode::new("_cssVars", false, SourceLocation::STUB),
                        allocator,
                    )));
                    element
                        .props
                        .push(PropNode::Directive(Box::new_in(bind, allocator)));
                }
                _ => {}
            },
            _ => {}
        }
    }
}
