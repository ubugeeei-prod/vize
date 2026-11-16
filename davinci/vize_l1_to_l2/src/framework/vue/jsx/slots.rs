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
//! `v-slots={slots}` on a component forwards a runtime slots object as
//! `ui.slots-object`, bound after the other props. An object literal there
//! waits with the slot objects above.

use oxc_ast::ast::{
    Expression, FormalParameters, FunctionBody, JSXAttribute, JSXAttributeName, JSXAttributeValue,
    JSXChild, JSXExpression, ObjectExpression, ObjectPropertyKind, PropertyKey, Statement,
};
use oxc_span::GetSpan;
use vize_l0::{Box, Span, String, Vec};
use vize_l2::artifact::OpFamily;
use vize_l2::expr::ExprRef;
use vize_l2::op::{
    BindingOp, DynamicName, ElementOp, Namespace, Op, Region, SlotContentOp, SlotsObjectOp,
};
use vize_l2::scope::{ScopeBinding, ScopeFacts, ScopeOrigin};

use super::attributes::Owner;
use super::children::{contains_jsx, element, push_children};
use super::control_flow::control_flow;
use super::{Cx, JsxRefusal, span};

/// The implicit slot of a render-function child.
const DEFAULT_SLOT: &str = "default";

/// Lower `children` as slot templates when they are a slot idiom. `false`
/// leaves them to ordinary child lowering, the implicit default slot.
pub(super) fn slot_children<'a>(
    cx: &mut Cx<'a, '_>,
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
    cx: &mut Cx<'a, '_>,
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
    cx: &mut Cx<'a, '_>,
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
        cx.attach_scope(slot, ScopeFacts { tag, bindings });
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
    Ok(Op::Element(Box::new_in(
        ElementOp {
            tag: "template",
            namespace: Namespace::Html,
            attributes: Vec::new_in(&cx.allocator),
            bindings,
            children: Region { ops },
            span: Span::new(at.start.min(name_at.start), at.end.max(name_at.end)),
        },
        &cx.allocator,
    )))
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

/// The value `v-slots` forwards: any expression but an object, a function,
/// a sequence or a literal, which are not a runtime slots object.
pub(super) fn forwarded_slots(
    attribute: &JSXAttribute<'_>,
    owner: Owner<'_, '_, '_>,
) -> Result<oxc_span::Span, JsxRefusal> {
    let (
        Owner::Component,
        JSXAttributeName::Identifier(_),
        Some(JSXAttributeValue::ExpressionContainer(container)),
    ) = (owner, &attribute.name, &attribute.value)
    else {
        return Err(JsxRefusal::Attribute);
    };
    let expression = match &container.expression {
        JSXExpression::ObjectExpression(_) | JSXExpression::EmptyExpression(_) => None,
        expression => expression.as_expression(),
    };
    let inner = expression
        .map(Expression::get_inner_expression)
        .ok_or(JsxRefusal::Attribute)?;
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
    Ok(inner.span())
}

/// `ui.slots-object` for the forwarded value at `at`.
pub(super) fn slots_object<'a>(
    cx: &Cx<'a, '_>,
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
