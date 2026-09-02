//! `@vue/babel-plugin-jsx`'s treatment of a component's lone expression child
//! (#3391), used only in opt-in Babel VDOM compatibility mode.
//!
//! Native Vize lowers such a child like any other component child, so it becomes
//! an implicit default slot and is stringified through `toDisplayString`. Babel
//! instead passes the value straight into the vnode's children argument:
//!
//! - with `enableObjectSlots: true` (its default) an identifier or a call result
//!   might *already* be a slots object, so it is discriminated at runtime by the
//!   plugin's `_isSlot` helper — a call is wrapped in an IIFE so it is evaluated
//!   exactly once;
//! - everything else — and every child under `enableObjectSlots: false` —
//!   becomes the raw value of an unescaped `default` slot.
//!
//! Object and render-prop children never reach here: they are the slot idiom
//! [`super::slot`] lowers into `<template v-slot>` children.

use oxc_ast::ast::{Expression, JSXChild, JSXExpressionContainer};
use oxc_span::{GetSpan, Span};
use vize_relief::{ElementNode, ExpressionNode, SimpleExpressionNode};
use vize_s0::{Box, String};

use super::Lowerer;
use super::slot::is_whitespace_child;

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    /// Lower a component's sole expression child the way Babel does, reporting
    /// whether it consumed the child. `false` leaves ordinary child lowering to
    /// the caller.
    pub(crate) fn lower_babel_sole_expression_child(
        &mut self,
        node: &mut ElementNode<'a>,
        container: &JSXExpressionContainer<'_>,
        has_v_slots: bool,
    ) -> bool {
        let Some(expression) = container.expression.as_expression() else {
            return false;
        };

        // `cond && <x/>` and friends are still lowered to v-if / v-for: they
        // render the same DOM as babel's untouched JavaScript.
        if let Some(child) = self.lower_control_flow_child(&container.expression, container.span) {
            node.children.push(child);
            return true;
        }

        // An explicit `v-slots` attribute owns the slots object, so ordinary
        // children stay children instead of becoming the slots argument.
        if has_v_slots {
            node.children
                .push(self.raw_expression_child(expression.span(), container.span));
            return true;
        }

        let inner = expression.get_inner_expression();
        if let Some(helper) = self.object_slots_helper().map(String::from) {
            let slots = match inner {
                Expression::Identifier(_) => {
                    Some(self.identifier_object_slots_expression(helper.as_str(), inner))
                }
                Expression::CallExpression(_) => {
                    Some(self.call_object_slots_expression(helper.as_str(), expression))
                }
                _ => None,
            };
            if let Some(slots) = slots {
                self.forward_slots_expression(node, slots);
                return true;
            }
        }

        // The comma operator has to keep its parentheses: `{ default: () => [a,
        // b] }` would otherwise be two array elements rather than one sequence.
        let slots = self.raw_default_slots_expression(
            expression,
            matches!(inner, Expression::SequenceExpression(_)),
        );
        self.forward_slots_expression(node, slots);
        true
    }

    /// `_isSlot(x) ? x : { default: () => [x] }` — Babel's identifier form,
    /// which is safe to repeat because an identifier has no side effects.
    fn identifier_object_slots_expression(
        &self,
        helper: &str,
        expression: &Expression<'_>,
    ) -> ExpressionNode<'a> {
        let source = self.mapper().slice(expression.span());
        let mut content = String::with_capacity(helper.len() + source.len() * 3 + 40);
        content.push_str(helper);
        content.push('(');
        content.push_str(source);
        content.push_str(") ? ");
        content.push_str(source);
        content.push_str(" : { default: () => [");
        content.push_str(source);
        content.push_str("] }");
        self.generated_slot_expression(content, expression.span())
    }

    /// The same check for a call, bound through an IIFE so the call itself is
    /// evaluated exactly once.
    fn call_object_slots_expression(
        &self,
        helper: &str,
        expression: &Expression<'_>,
    ) -> ExpressionNode<'a> {
        let source = self.mapper().slice(expression.span());
        let mut content = String::with_capacity(helper.len() + source.len() + 80);
        content.push_str("(_slot => ");
        content.push_str(helper);
        content.push_str("(_slot) ? _slot : { default: () => [_slot] })(");
        content.push_str(source);
        content.push(')');
        self.generated_slot_expression(content, expression.span())
    }

    /// `{ default: () => [x] }` — the raw default-slot value.
    fn raw_default_slots_expression(
        &self,
        expression: &Expression<'_>,
        parenthesize: bool,
    ) -> ExpressionNode<'a> {
        let source = self.mapper().slice(expression.span());
        let mut content = String::with_capacity(source.len() + 28);
        content.push_str("{ default: () => [");
        if parenthesize {
            content.push('(');
        }
        content.push_str(source);
        if parenthesize {
            content.push(')');
        }
        content.push_str("] }");
        self.generated_slot_expression(content, expression.span())
    }

    fn generated_slot_expression(&self, content: String, span: Span) -> ExpressionNode<'a> {
        ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new(
                self.bump().alloc_str(&content),
                false,
                self.mapper().location(span),
            ),
            &self.bump(),
        ))
    }
}

/// The component's only meaningful child when it is a single expression
/// container, ignoring whitespace-only text.
pub(super) fn sole_expression_container<'o>(
    children: &'o [JSXChild<'o>],
) -> Option<&'o JSXExpressionContainer<'o>> {
    let mut meaningful = children.iter().filter(|child| !is_whitespace_child(child));
    let only = meaningful.next()?;
    if meaningful.next().is_some() {
        return None;
    }
    match only {
        JSXChild::ExpressionContainer(container) => Some(container),
        _ => None,
    }
}
