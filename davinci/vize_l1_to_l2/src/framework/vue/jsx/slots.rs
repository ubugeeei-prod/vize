//! Component slot children: a sole render function (`{(p) => <A/>}`) or an
//! object of them (`{{ name: (p) => <A/> }}`) becomes `<template>` elements
//! carrying `ui.slot-content`, the ops a template's
//! `<template v-slot:name="p">` lowers to.
//!
//! A slot function takes at most one parameter and its body is an expression
//! or one `return` of markup or structural control flow. A slot returning a
//! value waits for its JSX child rendering (#12322); spreads, computed or
//! non-name keys and non-function entries wait for their slice.
//!
//! `v-slots` on a component takes the same object of render functions,
//! appended to the children as templates, or forwards a runtime slots object
//! (`v-slots={slots}`) as `ui.slots-object`, bound after the other props.

use oxc_ast::ast::{
    Expression, FormalParameters, FunctionBody, JSXAttribute, JSXAttributeItem, JSXAttributeName,
    JSXAttributeValue, JSXChild, JSXExpression, ObjectExpression, ObjectPropertyKind, PropertyKey,
    Statement,
};
use oxc_span::GetSpan;
use vize_l0::{Box, Span, String, Vec};
use vize_l2::artifact::OpFamily;
use vize_l2::expr::ExprRef;
use vize_l2::op::{
    BindingOp, DynamicName, ElementOp, Namespace, Op, Region, SlotContentOp, SlotsObjectOp,
};
use vize_l2::scope::{ScopeAnalysis, ScopeBinding, ScopeOrigin};

use super::attributes::Owner;
use super::children::{contains_jsx, element, push_children};
use super::control_flow::control_flow;
use super::{Cx, JsxRefusal, span};

/// The implicit slot of a render-function child.
const DEFAULT_SLOT: &str = "default";

/// Lower `children` as slot templates when they are a slot idiom. `false`
/// leaves them to ordinary child lowering, the implicit default slot.
pub(super) fn slot_children<'a>(
    cx: &mut Cx<'a, '_, impl super::syntax::SyntaxSink>,
    children: &[JSXChild<'_>],
    ops: &mut Vec<'a, Op<'a>>,
) -> Result<bool, JsxRefusal> {
    let mut meaningful = children
        .iter()
        .filter(|child| !matches!(child, JSXChild::Text(text) if text.value.trim().is_empty()));
    let (Some(JSXChild::ExpressionContainer(container)), None) =
        (meaningful.next(), meaningful.next())
    else {
        return Ok(false);
    };
    let Some(expression) = container.expression.as_expression() else {
        return Ok(false);
    };
    match expression.get_inner_expression() {
        Expression::ObjectExpression(object) => object_slots(cx, object, ops)?,
        function @ (Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)) => {
            let at = function.span();
            ops.push(template(cx, DEFAULT_SLOT, at, function)?);
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// `{{ header: () => <h1/>, 'footer': (p) => <p/> }}`: one template per
/// property, named by its identifier or string key.
fn object_slots<'a>(
    cx: &mut Cx<'a, '_, impl super::syntax::SyntaxSink>,
    object: &ObjectExpression<'_>,
    ops: &mut Vec<'a, Op<'a>>,
) -> Result<(), JsxRefusal> {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return Err(JsxRefusal::Child);
        };
        let name = match &property.key {
            _ if property.computed => return Err(JsxRefusal::Child),
            PropertyKey::StaticIdentifier(key) => key.name.as_str(),
            PropertyKey::StringLiteral(key) => key.value.as_str(),
            _ => return Err(JsxRefusal::Child),
        };
        let name = cx.allocator.alloc_str(name);
        ops.push(template(cx, name, property.key.span(), &property.value)?);
    }
    Ok(())
}

/// The `<template>` of one slot function, its `ui.slot-content` named at
/// `name_at`.
fn template<'a>(
    cx: &mut Cx<'a, '_, impl super::syntax::SyntaxSink>,
    name: &'a str,
    name_at: oxc_span::Span,
    function: &Expression<'_>,
) -> Result<Op<'a>, JsxRefusal> {
    let (params, body, expression_body, at) = match function.get_inner_expression() {
        Expression::ArrowFunctionExpression(arrow) => {
            (&*arrow.params, &*arrow.body, arrow.expression, arrow.span)
        }
        Expression::FunctionExpression(function) => (
            &*function.params,
            function.body.as_deref().ok_or(JsxRefusal::Child)?,
            false,
            function.span,
        ),
        _ => return Err(JsxRefusal::Child),
    };
    let rendered = returned(params, body, expression_body).ok_or(JsxRefusal::Child)?;
    cx.mint();
    let slot = cx.mint();
    let params = params
        .items
        .first()
        .map(|param| cx.expr(param.pattern.span()))
        .transpose()?;
    if let Some(params) = &params {
        let tag = cx.mint_scope();
        let mut bindings = alloc::vec::Vec::new();
        if let ExprRef::Js(js) = params
            && matches!(js.ast, Expression::Identifier(_))
        {
            bindings.push(ScopeBinding {
                name: String::from(js.source),
                origin: ScopeOrigin::Authored {
                    span: params.span(),
                },
            });
        }
        cx.attach_scope(slot, ScopeAnalysis { tag, bindings });
    }
    cx.observe(OpFamily::SlotCarrier);
    let mut bindings = Vec::new_in(&cx.allocator);
    bindings.push(BindingOp::SlotContent(Box::new_in(
        SlotContentOp {
            name: Some(DynamicName::Static(name)),
            modifiers: Vec::new_in(&cx.allocator),
            params,
            span: span(name_at),
        },
        &cx.allocator,
    )));
    let mut ops = Vec::new_in(&cx.allocator);
    match rendered.get_inner_expression() {
        Expression::JSXElement(nested) => ops.push(element(cx, nested)?),
        // A slot body is a child list, so a fragment splices into it.
        Expression::JSXFragment(fragment) => push_children(cx, &fragment.children, &mut ops)?,
        inner => ops.push(control_flow(cx, inner, inner.span())?.ok_or(JsxRefusal::Child)?),
    }
    let template = Box::new_in(
        ElementOp {
            tag: "template",
            namespace: Namespace::Html,
            attributes: Vec::new_in(&cx.allocator),
            bindings,
            children: Region { ops },
            span: Span::new(at.start.min(name_at.start), at.end.max(name_at.end)),
        },
        &cx.allocator,
    );
    cx.syntax.slot_range(&template, span(at));
    Ok(Op::Element(template))
}

/// What a slot function renders: its expression body or sole `return`, with
/// at most one plain parameter.
fn returned<'e, 'x>(
    params: &FormalParameters<'x>,
    body: &'e FunctionBody<'x>,
    expression_body: bool,
) -> Option<&'e Expression<'x>> {
    if params.items.len() > 1 || params.rest.is_some() {
        return None;
    }
    match body.statements.as_slice() {
        [Statement::ExpressionStatement(statement)] if expression_body => {
            Some(&statement.expression)
        }
        [Statement::ReturnStatement(statement)] if !expression_body => statement.argument.as_ref(),
        _ => None,
    }
}

/// `v-slots` and the camelCase spelling the JSX dialects accept too.
pub(super) const V_SLOTS: [&str; 2] = ["v-slots", "vSlots"];

/// What a component's `v-slots` lowers to.
#[derive(Clone, Copy)]
pub(super) enum SlotsAttribute {
    /// A runtime slots object at this span, bound after the other props.
    Forwarded(oxc_span::Span),
    /// An object of render functions, appended to the children as templates.
    Templates,
}

/// Classify a `v-slots` value. A forwarded value is any expression but an
/// object, a function, a sequence or a literal; an object must hold only
/// render functions.
pub(super) fn slots_attribute(
    attribute: &JSXAttribute<'_>,
    owner: Owner<'_, '_, '_>,
) -> Result<SlotsAttribute, JsxRefusal> {
    let (
        Owner::Component,
        JSXAttributeName::Identifier(_),
        Some(JSXAttributeValue::ExpressionContainer(container)),
    ) = (owner, &attribute.name, &attribute.value)
    else {
        return Err(JsxRefusal::Attribute);
    };
    let inner = match &container.expression {
        JSXExpression::EmptyExpression(_) => None,
        expression => expression.as_expression(),
    }
    .map(Expression::get_inner_expression)
    .ok_or(JsxRefusal::Attribute)?;
    if let Expression::ObjectExpression(object) = inner {
        let renders = object.properties.iter().all(|property| {
            matches!(property, ObjectPropertyKind::ObjectProperty(property)
            if matches!(
                property.value.get_inner_expression(),
                Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
            ))
        });
        // Any other object is forwarded verbatim or partly dropped: not yet.
        return if renders {
            Ok(SlotsAttribute::Templates)
        } else {
            Err(JsxRefusal::Attribute)
        };
    }
    let forwarded = !matches!(
        inner,
        Expression::ObjectExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::SequenceExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::BigIntLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::JSXElement(_)
            | Expression::JSXFragment(_)
    );
    if !forwarded || contains_jsx(inner) {
        return Err(JsxRefusal::Attribute);
    }
    Ok(SlotsAttribute::Forwarded(inner.span()))
}

/// The templates of the `v-slots` object among `items`, after the children.
pub(super) fn v_slots_templates<'a>(
    cx: &mut Cx<'a, '_, impl super::syntax::SyntaxSink>,
    items: &[JSXAttributeItem<'_>],
    ops: &mut Vec<'a, Op<'a>>,
) -> Result<(), JsxRefusal> {
    for item in items {
        if let JSXAttributeItem::Attribute(attribute) = item
            && let JSXAttributeName::Identifier(name) = &attribute.name
            && V_SLOTS.contains(&cx.slice(name.span)?)
            && let Some(JSXAttributeValue::ExpressionContainer(container)) = &attribute.value
            && let Some(Expression::ObjectExpression(object)) = container
                .expression
                .as_expression()
                .map(Expression::get_inner_expression)
        {
            return object_slots(cx, object, ops);
        }
    }
    Err(JsxRefusal::Attribute)
}

/// `ui.slots-object` for the forwarded value at `at`.
pub(super) fn slots_object<'a>(
    cx: &Cx<'a, '_, impl super::syntax::SyntaxSink>,
    at: oxc_span::Span,
) -> Result<BindingOp<'a>, JsxRefusal> {
    Ok(BindingOp::SlotsObject(Box::new_in(
        SlotsObjectOp {
            value: cx.expr(at)?,
            span: span(at),
        },
        &cx.allocator,
    )))
}
