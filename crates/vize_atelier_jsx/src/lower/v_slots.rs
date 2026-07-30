//! `v-slots` — the `@vue/babel-plugin-jsx` built-in that supplies a component's
//! slots from an attribute instead of from its children.
//!
//! `v-slots={{ default: () => <i/>, foo: (p) => <b/> }}` is the same object the
//! object-children idiom (`<B>{{ … }}</B>`) accepts, so it is lowered through the
//! same [`Lowerer::lower_object_slots`] into synthetic `<template v-slot:name>`
//! children.
//!
//! # Why this is a built-in and not a custom directive
//!
//! `v-slots` is a plugin built-in, not a user directive. Before #3418 any
//! unrecognized `v-*` attribute fell through to the generic custom-directive
//! path, so `v-slots` compiled to `resolveDirective("slots")` — a lookup for a
//! directive that does not exist, leaving the component with no slots, no error
//! and no warning. For the object-literal form it was worse than that: the
//! directive value was emitted as the **raw attribute source**, so
//! `v-slots={{ default: () => <i/> }}` put unparsed JSX into the generated
//! JavaScript module.
//!
//! # Combining with the element's own children
//!
//! The templates are appended **after** the element's own children, so the plain
//! children still become the `default` slot whenever the slots object does not
//! name one — babel's `{ default: () => [children], ...slots }`, with the same
//! keys in a different literal order.
//!
//! When the slots object *does* name `default` and the element also has
//! children, the shared transform reports "Extraneous children found when
//! component already has an explicit default slot."
//! (`vize_relief::errors`). Babel instead emits the `default` key twice and lets
//! JavaScript keep the later one, silently discarding the children — precisely
//! the failure shape this fix exists to remove — so Vize names the ambiguity
//! rather than picking a winner.
//!
//! # Shapes that are diagnosed rather than lowered
//!
//! - `v-slots={slots}` and any other non-object-literal value. Vize builds the
//!   slots object at compile time and has no IR for spreading an opaque value
//!   into it; babel emits `{…, ...slots}` or passes the value straight through as
//!   children. Tracked by #3467, which also records why implementing it needs a
//!   runtime harness rather than just a codegen change.
//! - `v-slots` with no value, or with a string value. Babel forwards these as
//!   `null` / `"str"` children, which is meaningless for a component.
//! - `v-slots:arg={…}` — `v-slots` takes no argument; the slot names are the
//!   object's keys.
//! - `v-slots` on a plain element, which has no slots.
//! - more than one `v-slots` on the same element. Babel keeps the last and drops
//!   the rest silently, which is the failure shape #3418 exists to remove.
//!
//! Inside the object literal, spread properties and computed keys keep the
//! existing object-children behavior: a warning naming what was ignored (see
//! `slot.rs`), not silence.

use oxc_ast::ast::{
    Expression, JSXAttribute, JSXAttributeItem, JSXAttributeName, JSXAttributeValue,
};
use oxc_span::{GetSpan, Span};
use vize_relief::ElementNode;

use super::Lowerer;
use super::expr::container_expr_span;

impl<'a, 'm, 's> Lowerer<'a, 'm, 's> {
    /// Whether `attr` is a `v-slots` attribute in any spelling.
    ///
    /// `lower_attribute` uses this to drop the attribute instead of turning it
    /// into a prop: `v-slots` supplies children, and it is applied by
    /// [`Self::apply_v_slots`] once the element's own children are lowered.
    pub(crate) fn is_v_slots_attribute(&self, attr: &JSXAttribute<'_>) -> bool {
        self.v_slots_name_span(attr).is_some()
    }

    /// Append the slot templates a `v-slots` attribute contributes to `node`, or
    /// report why the attribute could not be lowered.
    ///
    /// Called after `node.children` is built so the `v-slots` entries land last,
    /// leaving the element's plain children to become the `default` slot when the
    /// slots object does not name one. `on_component` is whether the owning tag
    /// renders as a component; `false` rejects the attribute.
    pub(crate) fn apply_v_slots(
        &mut self,
        node: &mut ElementNode<'a>,
        items: &[JSXAttributeItem<'_>],
        on_component: bool,
    ) {
        let attrs: std::vec::Vec<&JSXAttribute<'_>> = items
            .iter()
            .filter_map(|item| match item {
                JSXAttributeItem::Attribute(attr) if self.is_v_slots_attribute(attr) => {
                    Some(attr.as_ref())
                }
                _ => None,
            })
            .collect();
        let Some((attr, rest)) = attrs.split_first() else {
            return;
        };
        if let Some(extra) = rest.first() {
            self.reject(
                extra.span,
                "v-slots is given more than once on this element; babel keeps only the \
                 last one, so merge them into a single slots object.",
            );
            return;
        }

        if !matches!(&attr.name, JSXAttributeName::Identifier(_)) {
            self.reject(
                attr.span,
                "v-slots does not take an argument; the slot names are the keys of its \
                 object, e.g. v-slots={{ header: () => <h1/> }}.",
            );
            return;
        }
        if !on_component {
            self.reject(
                attr.span,
                "v-slots can only be used on a component; a plain element has no slots.",
            );
            return;
        }

        let Some(value) = attr.value.as_ref() else {
            self.reject(
                attr.span,
                "v-slots is missing its slots object, e.g. \
                 v-slots={{ default: () => <div/> }}.",
            );
            return;
        };
        let Some(object) = self.v_slots_object(value) else {
            // Name the expression itself rather than the `{…}` container, so the
            // message quotes `slots` and not `{slots}`.
            let span = match value {
                JSXAttributeValue::ExpressionContainer(container) => {
                    container_expr_span(container).unwrap_or_else(|| value.span())
                }
                other => other.span(),
            };
            let source = self.mapper().slice(span);
            self.reject_at(
                span,
                format_args!(
                    "v-slots value `{source}` is not an object literal: Vize builds the \
                     slots object at compile time and cannot forward an opaque slots \
                     value. Write the slots inline, e.g. \
                     v-slots={{{{ default: () => <div/> }}}}. Forwarding a slots object \
                     is tracked by #3467."
                ),
            );
            return;
        };

        for template in self.lower_object_slots(object) {
            node.children.push(template);
        }
    }

    /// The object literal a `v-slots` attribute value carries, or `None` when it
    /// is anything else.
    fn v_slots_object<'e>(
        &self,
        value: &'e JSXAttributeValue<'e>,
    ) -> Option<&'e oxc_ast::ast::ObjectExpression<'e>> {
        let JSXAttributeValue::ExpressionContainer(container) = value else {
            return None;
        };
        match &container.expression {
            oxc_ast::ast::JSXExpression::ObjectExpression(object) => Some(object),
            // `v-slots={({ … })}` — parentheses are transparent.
            oxc_ast::ast::JSXExpression::ParenthesizedExpression(paren) => {
                match paren.expression.get_inner_expression() {
                    Expression::ObjectExpression(object) => Some(object),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// The span of the `v-slots` name itself, for any spelling, or `None` when
    /// the attribute is not `v-slots`.
    fn v_slots_name_span(&self, attr: &JSXAttribute<'_>) -> Option<Span> {
        let (raw, span) = match &attr.name {
            JSXAttributeName::NamespacedName(named) => (
                self.mapper().slice(named.namespace.span()),
                named.namespace.span(),
            ),
            JSXAttributeName::Identifier(id) => (self.mapper().slice(id.span()), id.span()),
        };
        (raw == "v-slots").then_some(span)
    }
}
