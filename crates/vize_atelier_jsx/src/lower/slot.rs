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
//! already treat them as an implicit default slot.

use oxc_ast::ast::{
    ArrowFunctionExpression, Expression, Function, JSXChild, JSXExpression, ObjectPropertyKind,
    PropertyKey, Statement,
};
use oxc_span::{GetSpan, Span};
use vize_carton::{Box, Vec};
use vize_relief::ast::core::ElementType;
use vize_relief::ast::{DirectiveNode, ElementNode, PropNode, TemplateChildNode};

use super::Lowerer;
use crate::diagnostics::JsxDiagnostic;

impl<'a, 'm, 's> Lowerer<'a, 'm, 's> {
    /// Lower the children of a component element.
    ///
    /// When the component's sole meaningful child (ignoring whitespace text) is
    /// a single `JSXExpressionContainer` wrapping an object expression or an
    /// arrow/function, synthesize `<template v-slot>` children. Otherwise fall
    /// back to ordinary child lowering (which becomes an implicit default slot
    /// in the backends).
    pub(crate) fn lower_component_children(
        &mut self,
        children: &[JSXChild<'_>],
    ) -> Vec<'a, TemplateChildNode<'a>> {
        if let Some(slots) = self.try_lower_slot_idiom(children) {
            return slots;
        }
        self.lower_children(children)
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
        match &container.expression {
            JSXExpression::ObjectExpression(object) => Some(self.lower_object_slots(object)),
            JSXExpression::ArrowFunctionExpression(arrow) => {
                Some(self.lower_default_scoped_slot(arrow.as_ref().into()))
            }
            JSXExpression::FunctionExpression(func) => {
                Some(self.lower_default_scoped_slot(func.as_ref().into()))
            }
            JSXExpression::ParenthesizedExpression(paren) => {
                match paren.expression.get_inner_expression() {
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
            _ => None,
        }
    }

    /// `{{ name: (params) => body, … }}` — an object whose entries are named
    /// (and possibly scoped) slots.
    pub(crate) fn lower_object_slots(
        &mut self,
        object: &oxc_ast::ast::ObjectExpression<'_>,
    ) -> Vec<'a, TemplateChildNode<'a>> {
        let mut out = Vec::new_in(self.bump());
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

            let template = self.build_slot_template(slot_name, name_span, &slot_fn);
            out.push(TemplateChildNode::Element(Box::new_in(
                template,
                self.bump(),
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
        let mut out = Vec::new_in(self.bump());
        let template = self.build_slot_template("default", slot_fn.span, &slot_fn);
        out.push(TemplateChildNode::Element(Box::new_in(
            template,
            self.bump(),
        )));
        out
    }

    /// Build a synthetic `<template>` element carrying a `slot` directive whose
    /// `arg` is the static slot name and (for scoped slots) whose `exp` is the
    /// raw param-pattern source, with the lowered slot body as its children.
    fn build_slot_template(
        &mut self,
        slot_name: &str,
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
        node.props
            .push(PropNode::Directive(Box::new_in(directive, self.bump())));

        node.children = self.extract_fn_slot_body(slot_fn);
        node
    }

    /// Lower the body of a slot function into template children.
    ///
    /// Expression-body arrows (`() => <p/>`) reach the returned expression;
    /// block bodies (`() => { return <p/>; }`) reach the `return` argument.
    /// Only JSX elements/fragments are lowered as slot content; anything else
    /// is reported and produces an empty body.
    fn extract_fn_slot_body(&mut self, slot_fn: &SlotFn<'_>) -> Vec<'a, TemplateChildNode<'a>> {
        match slot_fn.return_expr {
            Some(Expression::JSXElement(element)) => {
                let mut out = Vec::new_in(self.bump());
                out.push(TemplateChildNode::Element(Box::new_in(
                    self.lower_element_node(element),
                    self.bump(),
                )));
                out
            }
            Some(Expression::JSXFragment(fragment)) => {
                let mut out = Vec::new_in(self.bump());
                out.push(TemplateChildNode::Element(Box::new_in(
                    self.lower_fragment_node(fragment),
                    self.bump(),
                )));
                out
            }
            _ => Vec::new_in(self.bump()),
        }
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
fn is_whitespace_child(child: &JSXChild<'_>) -> bool {
    matches!(child, JSXChild::Text(text) if text.value.as_str().trim().is_empty())
}
