//! `v-slots` — the `@vue/babel-plugin-jsx` built-in that supplies a component's
//! slots from an attribute instead of from its children.
//!
//! `v-slots={{ default: () => <i/>, foo: (p) => <b/> }}` is the same object the
//! object-children idiom (`<B>{{ … }}</B>`) accepts, so it is lowered through the
//! same [`Lowerer::lower_object_slots`] into synthetic `<template v-slot:name>`
//! children.
//!
//! # Forwarding an opaque slots object
//!
//! `v-slots={slots}` — the spelling the plugin README uses for slot forwarding —
//! carries a value the compiler cannot see inside, so there are no entries to
//! synthesize templates from. It lowers instead to a relief `slots` directive on
//! the component, which `vize_atelier_core`'s slot codegen emits as a spread
//! (#3467). Two shapes, both matching babel exactly:
//!
//! ```jsx
//! const A = () => <B v-slots={slots}/>;
//! const C = () => <B v-slots={slots}><div>A</div></B>;
//! ```
//!
//! ```js
//! const A = () => _createVNode(_resolveComponent("B"), null, slots);
//! const C = () => _createVNode(_resolveComponent("B"), null, {
//!   default: () => [_createVNode("div", null, [_createTextVNode("A")])],
//!   ...slots
//! });
//! ```
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
//! - `v-slots` with no value, or with a literal value (`v-slots="str"`,
//!   `v-slots={1}`, `v-slots={[…]}`). Babel forwards these as the component's
//!   children, which is meaningless for a component: a slots object is either an
//!   object literal to expand or an opaque expression to forward, never a
//!   primitive. Opt-in Babel VDOM mode (#3391) forwards the *self-contained*
//!   literals verbatim, including substitution-free template literals, because
//!   that is exactly what babel emits and the source text is already valid JavaScript;
//!   arrays, interpolated template literals, raw JSX, functions, and sequences
//!   stay diagnosed rather than emitted as a malformed module.
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
    Expression, JSXAttribute, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXExpression,
    ObjectExpression,
};
use oxc_span::{GetSpan, Span};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode};

use super::Lowerer;
use super::expr::container_expr_span;

/// What a `v-slots` attribute value is, once parentheses and TS wrappers are
/// seen through.
enum SlotsValue<'e> {
    /// `v-slots={{ … }}` — entries the compiler expands into slot templates.
    Object(&'e ObjectExpression<'e>),
    /// `v-slots={slots}` — an expression that stays opaque at compile time and
    /// is forwarded into the slots object as a spread. The span is the source to
    /// emit, with any wrapper stripped.
    Forwarded(Span),
    /// A self-contained literal (`v-slots={1}` or a substitution-free template
    /// literal). Native mode diagnoses it like any other non-slots value; Babel
    /// VDOM compatibility forwards it verbatim as the vnode's children
    /// argument, matching the plugin.
    CompatForwarded(Span),
    /// A literal, an array, a lone function, or a quoted/missing value: babel
    /// forwards these as the component's children, which is not a slots object.
    /// The span names the offending source, the `&'static str` says why.
    Rejected(Span, &'static str),
}

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
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
        match classify_v_slots_value(value) {
            SlotsValue::Object(object) => {
                for template in self.lower_object_slots(object) {
                    node.children.push(template);
                }
            }
            SlotsValue::Forwarded(span) => self.forward_slots(node, span),
            SlotsValue::CompatForwarded(span) if self.uses_babel_vdom_compat() => {
                self.forward_slots(node, span);
            }
            SlotsValue::CompatForwarded(span) => {
                self.reject_v_slots_value(span, NOT_A_SLOTS_OBJECT);
            }
            SlotsValue::Rejected(span, reason) => self.reject_v_slots_value(span, reason),
        }
    }

    /// Report a `v-slots` value that is not a slots object, quoting it.
    fn reject_v_slots_value(&mut self, span: Span, reason: &'static str) {
        let source = self.mapper().slice(span);
        self.reject_at(
            span,
            format_args!(
                "v-slots value `{source}` {reason}. Write the slots inline, e.g. \
                 v-slots={{{{ default: () => <div/> }}}}, or forward a slots \
                 object, e.g. v-slots={{slots}}."
            ),
        );
    }

    /// Record a forwarded slots object on `node` as a relief `slots` directive.
    ///
    /// The shared slot codegen turns it into `...expr` inside the slots object,
    /// or into the whole children argument when nothing else contributes slots
    /// (#3467). Keeping it a directive rather than a synthesized child is what
    /// lets the value stay opaque: there are no entries to build templates from.
    fn forward_slots(&mut self, node: &mut ElementNode<'a>, span: Span) {
        let expression = self.dyn_expr(span);
        self.forward_slots_expression(node, expression);
    }

    pub(crate) fn forward_slots_expression(
        &self,
        node: &mut ElementNode<'a>,
        expression: ExpressionNode<'a>,
    ) {
        let loc = expression.loc().clone();
        let mut directive = DirectiveNode::new(self.bump(), "slots", loc);
        directive.exp = Some(expression);
        node.props.push(PropNode::Directive(self.boxed(directive)));
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

/// Classify a `v-slots` attribute value.
///
/// Spans always name the expression itself rather than its `{…}` container, so
/// a diagnostic quotes `1` and not `{1}`, and a forwarded value is emitted
/// without its wrapper (`slots as Slots` forwards as `slots`).
fn classify_v_slots_value<'e>(value: &'e JSXAttributeValue<'e>) -> SlotsValue<'e> {
    let JSXAttributeValue::ExpressionContainer(container) = value else {
        // `v-slots="str"` and the JSX element/fragment value forms.
        return SlotsValue::Rejected(value.span(), NOT_A_SLOTS_OBJECT);
    };
    if let JSXExpression::ObjectExpression(object) = &container.expression {
        return SlotsValue::Object(object);
    }
    let Some(span) = container_expr_span(container) else {
        // `v-slots={}` / `v-slots={/* … */}` carry no expression at all.
        return SlotsValue::Rejected(container.span, NOT_A_SLOTS_OBJECT);
    };
    let Some(expression) = container.expression.as_expression() else {
        return SlotsValue::Rejected(span, NOT_A_SLOTS_OBJECT);
    };
    // Parentheses and TS wrappers (`as`, `satisfies`, `!`) are transparent: what
    // matters is the expression underneath, and forwarding its own span keeps
    // TS-only syntax out of the emitted JavaScript.
    let inner = expression.get_inner_expression();
    match inner {
        Expression::ObjectExpression(object) => SlotsValue::Object(object),
        // A lone function is the *default slot*, not a slots object: babel
        // forwards it as children and Vue's `normalizeChildren` wraps it as
        // `{default: fn}`. Spreading it would yield `{}`, so name it instead.
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => {
            SlotsValue::Rejected(inner.span(), IS_A_FUNCTION)
        }
        // The comma operator does not survive being emitted verbatim as a
        // spread (`{...a, b}` is two properties, not one spread).
        Expression::SequenceExpression(_) => SlotsValue::Rejected(inner.span(), NOT_A_SLOTS_OBJECT),
        _ if is_compat_forwardable_literal(inner) => SlotsValue::CompatForwarded(inner.span()),
        _ if is_literal_value(inner) => SlotsValue::Rejected(inner.span(), NOT_A_SLOTS_OBJECT),
        _ => SlotsValue::Forwarded(inner.span()),
    }
}

const NOT_A_SLOTS_OBJECT: &str = "is not a slots object: babel forwards it as the component's \
                                  children, which leaves the component with no slots";
const IS_A_FUNCTION: &str = "is a function, not a slots object: a lone function is the default \
                             slot, so a spread of it contributes nothing";

/// Literals whose source text is already valid JavaScript in children position,
/// so babel's pass-through can be reproduced by forwarding the span verbatim.
///
/// Container literals are deliberately excluded: an array may hold nested JSX
/// that still has to be lowered, and raw JSX is a vnode rather than a value. A
/// template literal only qualifies when it has no substitutions, for the same
/// reason: its interpolations can contain JSX.
fn is_compat_forwardable_literal(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::BigIntLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        _ => false,
    }
}

/// Whether an expression is a literal value that can never be a slots object.
///
/// Everything else — an identifier, a member/call expression, a conditional —
/// stays opaque at compile time and is forwarded.
fn is_literal_value(expression: &Expression<'_>) -> bool {
    matches!(
        expression,
        Expression::ArrayExpression(_)
            | Expression::BigIntLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::JSXElement(_)
            | Expression::JSXFragment(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::TemplateLiteral(_)
    )
}
