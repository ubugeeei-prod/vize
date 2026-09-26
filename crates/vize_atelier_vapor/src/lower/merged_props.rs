//! Computed DOM keys share an ordered prop source with every authored attribute.
//! Keep key/value nodes separate, preserving retained expression trees and
//! allowing the runtime to restore static values when a computed key changes.

use vize_atelier_core::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, SimpleExpressionNode,
};
use vize_carton::{Box, Vec};

use super::context::TransformContext;
use crate::ir::{BlockIRNode, IRProp, MergedPropsSource, OperationNode, SetMergedPropsIRNode};

pub(super) fn uses_computed_props(el: &ElementNode<'_>) -> bool {
    let mut computed = false;
    for prop in &el.props {
        if let PropNode::Directive(dir) = prop
            && dir.name == "bind"
        {
            if !dir.modifiers.is_empty() || !matches!(dir.exp, Some(ExpressionNode::Simple(_))) {
                return false;
            }
            match &dir.arg {
                Some(ExpressionNode::Simple(key)) => {
                    computed |= !key.is_static;
                }
                None => {}
                _ => return false,
            }
        }
    }
    computed
}

pub(super) fn transform<'a>(
    ctx: &mut TransformContext<'a>,
    current: &DirectiveNode<'a>,
    element: usize,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    let first = el.props.iter().find_map(|prop| match prop {
        PropNode::Directive(dir) if dir.name == "bind" => Some(dir.as_ref()),
        _ => None,
    });
    if !first.is_some_and(|first| std::ptr::eq(first, current)) {
        return;
    }
    let mut sources = Vec::new_in(&ctx.allocator);
    let mut group: Vec<'a, IRProp<'a>> = Vec::new_in(&ctx.allocator);
    for prop in &el.props {
        let (key, value) = match prop {
            PropNode::Attribute(attr)
                if !super::element::template::is_runtime_only_attr(attr.name) =>
            {
                let key = SimpleExpressionNode::new(attr.name, true, attr.name_loc.clone());
                let value = attr.value.as_ref().map_or_else(
                    || SimpleExpressionNode::new("", true, attr.loc.clone()),
                    |value| SimpleExpressionNode::new(value.content, true, value.loc.clone()),
                );
                (key, value)
            }
            PropNode::Directive(dir) if dir.name == "bind" => {
                let Some(ExpressionNode::Simple(value)) = &dir.exp else {
                    continue;
                };
                let Some(ExpressionNode::Simple(key)) = &dir.arg else {
                    if !group.is_empty() {
                        sources.push(MergedPropsSource::Group(std::mem::replace(
                            &mut group,
                            Vec::new_in(&ctx.allocator),
                        )));
                    }
                    sources.push(MergedPropsSource::Object(Box::new_in(
                        SimpleExpressionNode::from_node(value),
                        &ctx.allocator,
                    )));
                    continue;
                };
                if key.is_static && matches!(key.content, "key" | "ref" | "ref_for" | "ref_key") {
                    continue;
                }
                (
                    SimpleExpressionNode::from_node(key),
                    SimpleExpressionNode::from_node(value),
                )
            }
            _ => continue,
        };
        let value = Box::new_in(value, &ctx.allocator);
        if key.is_static
            && matches!(key.content, "class" | "style")
            && let Some(previous) = group
                .iter_mut()
                .find(|prop| prop.key.is_static && prop.key.content == key.content)
        {
            previous.values.push(value);
        } else {
            let mut values = Vec::new_in(&ctx.allocator);
            values.push(value);
            group.push(IRProp::new(Box::new_in(key, &ctx.allocator), values, false));
        }
    }
    if !group.is_empty() {
        sources.push(MergedPropsSource::Group(group));
    }
    ctx.push_dynamic_operation(
        block,
        OperationNode::SetMergedProps(SetMergedPropsIRNode { element, sources }),
    );
}
