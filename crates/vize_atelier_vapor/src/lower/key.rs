//! Authored keys outside list carriers introduce an existing block scope.

use oxc_ast::ast::Expression;
use vize_atelier_core::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode, SimpleExpressionNode,
};
use vize_carton::Box;

use super::{
    context::TransformContext,
    element::{transform_classified_element, transform_element_unkeyed},
};
use crate::ir::{BlockIRNode, InsertionAnchor, KeyIRNode, OperationNode};

mod branch;
mod classify;
pub(super) use branch::transform_branch;
pub(super) use classify::{DirectiveAnalysis, ScopeDirectives, classify};

pub(super) fn is_non_reactive(el: &ElementNode<'_>, inherited: bool) -> bool {
    inherited || el.props.iter().any(|prop| matches!(prop, PropNode::Directive(dir)
        if dir.name == "once" || dir.name == "memo"
            && matches!(dir.exp.as_ref(), Some(ExpressionNode::Simple(exp)) if exp.content.trim() == "[]")))
}

/// The exact original binding, shared by lowering and template classification.
pub(super) fn value<'a, 'b>(
    el: &'b ElementNode<'a>,
    non_reactive: bool,
) -> Option<&'b SimpleExpressionNode<'a>> {
    if non_reactive || !matches!(el.tag_type, ElementType::Element | ElementType::Component) {
        return None;
    }
    let value = el.props.iter().find_map(|prop| match prop {
        PropNode::Directive(dir) => binding(dir),
        _ => None,
    })?;
    (eligible_target(el) && eligible(el, value, non_reactive)).then_some(value)
}

fn binding<'a, 'b>(dir: &'b DirectiveNode<'a>) -> Option<&'b SimpleExpressionNode<'a>> {
    if dir.name != "bind" {
        return None;
    }
    binding_value(dir)
}

/// The caller has already matched this original directive's `bind` name.
fn binding_value<'a, 'b>(dir: &'b DirectiveNode<'a>) -> Option<&'b SimpleExpressionNode<'a>> {
    let Some(ExpressionNode::Simple(arg)) = &dir.arg else {
        return None;
    };
    if !arg.is_static || arg.content != "key" {
        return None;
    }
    match &dir.exp {
        Some(ExpressionNode::Simple(value)) => Some(value.as_ref()),
        _ => None,
    }
}

fn eligible(el: &ElementNode<'_>, value: &SimpleExpressionNode<'_>, non_reactive: bool) -> bool {
    !is_non_reactive(el, non_reactive)
        && !el
            .props
            .iter()
            .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "for"))
        && eligible_value(value)
}

fn eligible_target(el: &ElementNode<'_>) -> bool {
    matches!(el.tag_type, ElementType::Element | ElementType::Component)
        && !matches!(
            el.tag,
            "component"
                | "Component"
                | "KeepAlive"
                | "keep-alive"
                | "Teleport"
                | "teleport"
                | "Suspense"
                | "suspense"
                | "Transition"
                | "transition"
                | "TransitionGroup"
                | "transition-group"
        )
}

fn eligible_value(value: &SimpleExpressionNode<'_>) -> bool {
    !(value.is_static
        || value.const_type != vize_atelier_core::ConstantType::NotConstant
        || matches!(value.content.trim(), "true" | "false" | "null")
        || value
            .js_ast
            .and_then(|js| js.as_expression())
            .is_some_and(|js| {
                js.raw == value.content
                    && matches!(
                        js.ast,
                        Expression::NumericLiteral(_)
                            | Expression::StringLiteral(_)
                            | Expression::BooleanLiteral(_)
                            | Expression::NullLiteral(_)
                            | Expression::BigIntLiteral(_)
                    )
            }))
}

#[expect(
    clippy::too_many_arguments,
    reason = "the original value joins its insertion owner"
)]
pub(super) fn transform<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
    id: Option<usize>,
    parent: Option<usize>,
    anchor: Option<InsertionAnchor>,
    add_return: bool,
    value: &SimpleExpressionNode<'a>,
    directives: Option<DirectiveAnalysis<'a, '_>>,
) {
    let value = Box::new_in(SimpleExpressionNode::from_node(value), &ctx.allocator);
    let id = id.unwrap_or_else(|| ctx.next_id());
    let mut render = BlockIRNode::new(ctx.allocator);
    // Reuse the same original element; only this consumed key is suppressed.
    if let Some(directives) = directives {
        transform_classified_element(ctx, el, &mut render, directives);
    } else {
        transform_element_unkeyed(ctx, el, &mut render);
    }
    block.operation.push(OperationNode::Key(Box::new_in(
        KeyIRNode {
            id,
            value,
            render,
            parent,
            anchor,
        },
        &ctx.allocator,
    )));
    if add_return {
        block.returns.push(id);
    }
}

pub(crate) fn transform_element<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    element(ctx, el, block, true);
}

pub(super) fn element<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
    own_key: bool,
) {
    if el.tag_type == ElementType::Slot {
        super::element::transform_slot(ctx, el, block);
        return;
    }
    let directives = classify(el, ctx.is_key_non_reactive(), own_key);
    if let Some(value) = directives.key {
        transform(
            ctx,
            el,
            block,
            None,
            None,
            None,
            true,
            value,
            Some(directives),
        );
    } else {
        transform_classified_element(ctx, el, block, directives);
    }
}

#[cfg(test)]
mod target_tests;

#[cfg(test)]
mod property_tests;
