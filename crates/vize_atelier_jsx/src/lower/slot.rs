//! Lowering JSX component-slot idioms into synthetic `<template v-slot>`s.
//!
//! `@vue/babel-plugin-jsx` expresses named/scoped slots by passing a single
//! object-expression child (`<Comp>{{ header: () => <h1/> }}</Comp>`) or, for a
//! default scoped slot, a single render-prop child
//! (`<List>{(item) => <li/>}</List>`). Rather than teach the VDOM/Vapor
//! backends a JSX-specific slot shape, we lower these into the same
//! `<template v-slot:name="params">…</template>` element children the SFC
//! template path already produces. The shared slot transform + codegen then
//! build the slots object from those templates.
//!
//! Plain element/text children of a component are left untouched: the backends
//! already treat them as an implicit default slot. Opt-in Babel VDOM mode
//! instead routes a lone expression child through [`super::babel_slot`].

use oxc_ast::ast::{
    ArrowFunctionExpression, Expression, Function, JSXChild, ObjectPropertyKind, PropertyKey,
    Statement,
};
use oxc_span::{GetSpan, Span};
use vize_relief::ElementType;
use vize_relief::{DirectiveNode, ElementNode, PropNode, TemplateChildNode, TextNode};
use vize_s0::{Box, Vec};

use super::Lowerer;
use super::babel_slot::sole_expression_container;
use crate::diagnostics::JsxDiagnostic;

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    /// Lower the children of a component element.
    ///
    /// When the component's sole meaningful child (ignoring whitespace text) is
    /// a single `JSXExpressionContainer` wrapping an object expression or an
    /// arrow/function, synthesize `<template v-slot>` children. Otherwise fall
    /// back to ordinary child lowering (which becomes an implicit default slot
    /// in the backends).
    pub(crate) fn lower_component_children_into(
        &mut self,
        node: &mut ElementNode<'a>,
        children: &[JSXChild<'_>],
        has_v_slots: bool,
    ) {
        if !(self.uses_babel_vdom_compat() && has_v_slots)
            && let Some(slots) = self.try_lower_slot_idiom(children)
        {
            node.children = slots;
            return;
        }

        // Babel passes a component's lone expression child straight into the
        // vnode's children argument rather than making it an implicit default
        // slot; `babel_slot` owns that shape.
        if self.uses_babel_vdom_compat()
            && let Some(container) = sole_expression_container(children)
            && self.lower_babel_sole_expression_child(node, container, has_v_slots)
        {
            return;
        }

        node.children = self.lower_children(children);
    }

    /// Detect and lower the slot idiom, returning `None` when the children are
    /// not a single object/render-prop slot expression.
    fn try_lower_slot_idiom(
        &mut self,
        children: &[JSXChild<'_>],
    ) -> Option<Vec<'a, TemplateChildNode<'a>>> {
        // The sole meaningful child must be a single expression container.
        let mut meaningful = children.iter().filter(|child| !is_whitespace_child(child));
        let only = meaningful.next()?;
        if meaningful.next().is_some() {
            return None;
        }
        let JSXChild::ExpressionContainer(container) = only else {
            return None;
        };
        let expression = container.expression.as_expression()?;
        match expression.get_inner_expression() {
            Expression::ObjectExpression(object) => Some(self.lower_object_slots(object)),
            Expression::ArrowFunctionExpression(arrow) => {
                Some(self.lower_default_scoped_slot(arrow.as_ref().into()))
            }
            Expression::FunctionExpression(func) => {
                Some(self.lower_default_scoped_slot(func.as_ref().into()))
            }
            _ => None,
        }
    }

    /// `{{ name: (params) => body, … }}` — an object whose entries are named
    /// (and possibly scoped) slots.
    pub(crate) fn lower_object_slots(
        &mut self,
        object: &oxc_ast::ast::ObjectExpression<'_>,
    ) -> Vec<'a, TemplateChildNode<'a>> {
        let mut out = self.vec();
        for prop in object.properties.iter() {
            let ObjectPropertyKind::ObjectProperty(property) = prop else {
                self.report(JsxDiagnostic::warning(
                    "spread in a JSX slot object is not supported and was ignored",
                    prop.span().start,
                    prop.span().end,
                ));
                continue;
            };

            if property.computed {
                self.report(JsxDiagnostic::warning(
                    "computed JSX slot names are not supported and were ignored",
                    property.span.start,
                    property.span.end,
                ));
                continue;
            }

            let Some((slot_name, name_span)) = static_key(&property.key) else {
                self.report(JsxDiagnostic::warning(
                    "unsupported JSX slot name; only identifier or string keys are allowed",
                    property.key.span().start,
                    property.key.span().end,
                ));
                continue;
            };

            let Some(slot_fn) = SlotFn::from_value(&property.value) else {
                self.report(JsxDiagnostic::warning(
                    "JSX slot values must be a function returning the slot content; ignored",
                    property.value.span().start,
                    property.value.span().end,
                ));
                continue;
            };

            // Slot names come out of the parsed module, so they are copied
            // into the compile arena before reaching the node.
            let slot_name = self.bump().alloc_str(slot_name);
            let template = self.build_slot_template(slot_name, name_span, &slot_fn);
            out.push(TemplateChildNode::Element(Box::new_in(
                template,
                &self.bump(),
            )));
        }
        out
    }

    /// `{(params) => body}` — a single render-prop child becomes the default
    /// scoped slot.
    pub(crate) fn lower_default_scoped_slot(
        &mut self,
        slot_fn: SlotFn<'_>,
    ) -> Vec<'a, TemplateChildNode<'a>> {
        let mut out = self.vec();
        let template = self.build_slot_template("default", slot_fn.span, &slot_fn);
        out.push(TemplateChildNode::Element(Box::new_in(
            template,
            &self.bump(),
        )));
        out
    }

    /// Build a synthetic `<template>` element carrying a `slot` directive whose
    /// `arg` is the static slot name and (for scoped slots) whose `exp` is the
    /// raw param-pattern source, with the lowered slot body as its children.
    fn build_slot_template(
        &mut self,
        slot_name: &'a str,
        name_span: Span,
        slot_fn: &SlotFn<'_>,
    ) -> ElementNode<'a> {
        let loc = self.mapper().location(slot_fn.span);
        let mut node = ElementNode::new(self.bump(), "template", loc);
        // REQUIRED: the Vapor slot-IR build keys off `tag_type == Template`.
        node.tag_type = ElementType::Template;

        let mut directive =
            DirectiveNode::new(self.bump(), "slot", self.mapper().location(name_span));
        directive.arg = Some(self.static_expr(slot_name, name_span));
        if let Some(param_span) = slot_fn.param_span {
            // The scoped-slot params carry the RAW pattern source (`{ x }`,
            // `item`); `dyn_expr` slices exactly that span.
            directive.exp = Some(self.dyn_expr(param_span));
        }
        node.props.push(PropNode::Directive(self.boxed(directive)));

        node.children = self.extract_fn_slot_body(slot_fn);
        node
    }

    /// Lower the body of a slot function into template children.
    ///
    /// Expression-body arrows (`() => <p/>`) reach the returned expression;
    /// block bodies (`() => { return <p/>; }`) reach the `return` argument. A
    /// JSX element/fragment becomes the slot content directly; a control-flow
    /// expression (`(rows) => rows.map(...)`, a conditional, or `&&`) reuses the
    /// shared control-flow lowering so a list/conditional rendered inside a slot
    /// works the same as one rendered as an ordinary child.
    ///
    /// Anything else is rendered as the slot's content, exactly as the same
    /// expression would be as an ordinary child. This used to produce an empty
    /// body instead, so `<B>{() => 'foo'}</B>` silently rendered nothing.
    fn extract_fn_slot_body(&mut self, slot_fn: &SlotFn<'_>) -> Vec<'a, TemplateChildNode<'a>> {
        let mut out = self.vec();
        let Some(expr) = slot_fn.return_expr else {
            return out;
        };
        match expr.get_inner_expression() {
            Expression::JSXElement(element) => {
                out.push(TemplateChildNode::Element(Box::new_in(
                    self.lower_element_node(element),
                    &self.bump(),
                )));
            }
            // A slot body is already a child *list*, so a fragment body splices
            // into it exactly as it does in element child position (#3421).
            Expression::JSXFragment(fragment) => {
                out = self.lower_children(&fragment.children);
            }
            // `&&` / ternary / `.map(...)` slot bodies lower to the same
            // If/For relief children as control-flow expression children.
            other => {
                if let Some(child) = self.lower_control_flow_expr(other, other.span()) {
                    out.push(child);
                } else {
                    out.push(self.slot_body_value(other));
                }
            }
        }
        out
    }

    /// Render a plain-expression slot body the way the same expression renders
    /// as an ordinary child: a string literal becomes text, anything else an
    /// interpolation.
    fn slot_body_value(&mut self, expr: &Expression<'_>) -> TemplateChildNode<'a> {
        if let Expression::StringLiteral(string) = expr {
            return TemplateChildNode::Text(Box::new_in(
                TextNode::new(
                    self.bump().alloc_str(string.value.as_str()),
                    self.mapper().location(string.span),
                ),
                &self.bump(),
            ));
        }
        let content = self.dyn_expr(expr.span());
        self.interpolation(content, expr.span())
    }
}

/// A normalized view of a slot function (arrow or `function`).
pub(crate) struct SlotFn<'o> {
    span: Span,
    /// Span of the single formal param's binding pattern (scoped slot), if any.
    param_span: Option<Span>,
    /// The JSX expression returned by the function body, if reachable.
    return_expr: Option<&'o Expression<'o>>,
}

impl<'o> SlotFn<'o> {
    fn from_value(value: &'o Expression<'o>) -> Option<Self> {
        match value.get_inner_expression() {
            Expression::ArrowFunctionExpression(arrow) => Some(arrow.as_ref().into()),
            Expression::FunctionExpression(func) => Some(func.as_ref().into()),
            _ => None,
        }
    }
}

impl<'o> From<&'o ArrowFunctionExpression<'o>> for SlotFn<'o> {
    fn from(arrow: &'o ArrowFunctionExpression<'o>) -> Self {
        SlotFn {
            span: arrow.span,
            param_span: single_param_span(arrow.params.items.as_slice()),
            return_expr: arrow_return_expr(arrow),
        }
    }
}

impl<'o> From<&'o Function<'o>> for SlotFn<'o> {
    fn from(func: &'o Function<'o>) -> Self {
        SlotFn {
            span: func.span,
            param_span: single_param_span(func.params.items.as_slice()),
            return_expr: func.body.as_ref().and_then(|body| block_return_expr(body)),
        }
    }
}

/// The binding-pattern span when a function has exactly one formal parameter.
fn single_param_span(items: &[oxc_ast::ast::FormalParameter<'_>]) -> Option<Span> {
    match items {
        [only] => Some(only.pattern.span()),
        _ => None,
    }
}

/// The expression returned by an arrow (expression body or `return`).
fn arrow_return_expr<'o>(arrow: &'o ArrowFunctionExpression<'o>) -> Option<&'o Expression<'o>> {
    if arrow.expression {
        // Expression body: the synthetic body holds a single ExpressionStatement.
        match arrow.body.statements.first()? {
            Statement::ExpressionStatement(stmt) => Some(&stmt.expression),
            _ => None,
        }
    } else {
        block_return_expr(&arrow.body)
    }
}

/// The argument of the first `return` statement in a block body.
fn block_return_expr<'o>(body: &'o oxc_ast::ast::FunctionBody<'o>) -> Option<&'o Expression<'o>> {
    body.statements.iter().find_map(|stmt| match stmt {
        Statement::ReturnStatement(ret) => ret.argument.as_ref(),
        _ => None,
    })
}

/// A static object-property key as `(name, span)`; `None` for computed/dynamic.
fn static_key<'o>(key: &'o PropertyKey<'o>) -> Option<(&'o str, Span)> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some((id.name.as_str(), id.span)),
        PropertyKey::StringLiteral(lit) => Some((lit.value.as_str(), lit.span)),
        _ => None,
    }
}

/// Whether a child is whitespace-only text (dropped before slot detection).
pub(super) fn is_whitespace_child(child: &JSXChild<'_>) -> bool {
    matches!(child, JSXChild::Text(text) if text.value.as_str().trim().is_empty())
}
