//! `v-model` and `v-models` as `ui.model`, the op a template `v-model`
//! produces: read and write are the authored target, the argument rides the
//! name position, and the element kind and modifiers ride as attributes
//! carrying the binding's span.
//!
//! The JSX spellings name the same fields: `v-model:arg={x}`,
//! `v-model_trim={x}` (`_` modifiers on the name), `v-model={[x, 'arg',
//! ['trim']]}` and `v-models={[[x, 'arg', ['trim']], …]}`.

use oxc_ast::ast::{
    ArrayExpression, Expression, JSXAttributeItem, JSXAttributeName, JSXAttributeValue,
};
use oxc_span::GetSpan;
use vize_l0::{Box, Vec};
use vize_l2::op::{Attribute, BindingContract, BindingOp, DynamicName, ModelOp};
use vize_l2::template::OpFamily;

use super::attributes::{Owner, Props};
use super::directives::ArraySpelling;
use super::{Cx, JsxRefusal, span};

/// The `ui.model` element kind of a component.
const COMPONENT: &str = "component";

/// One model binding before its owner admits it.
struct Model<'a> {
    target: oxc_span::Span,
    argument: Option<&'a str>,
    modifiers: Vec<'a, &'a str>,
}

/// `v-model` (`name` is `model` or `model_<modifiers>`), with the argument
/// of a `v-model:arg` spelling.
pub(super) fn model<'a>(
    cx: &mut Cx<'a, '_>,
    name: &'a str,
    argument: Option<&'a str>,
    value: Option<&Expression<'_>>,
    at: oxc_span::Span,
    owner: Owner<'a, '_, '_>,
    out: &mut Props<'a>,
) -> Result<(), JsxRefusal> {
    let value = value.ok_or(JsxRefusal::Attribute)?;
    let mut modifiers = Vec::new_in(&cx.allocator);
    if let Some(suffix) = name.strip_prefix("model_") {
        modifiers.extend(suffix.split('_').filter(|modifier| !modifier.is_empty()));
        if modifiers.is_empty() {
            return Err(JsxRefusal::Attribute);
        }
    }
    // The `_` split of a `v-model:arg_m` argument waits for #12313.
    if argument.is_some_and(|argument| argument.contains('_')) {
        return Err(JsxRefusal::Attribute);
    }
    let model = match value {
        Expression::ArrayExpression(array) if argument.is_none() && modifiers.is_empty() => {
            array_model(cx, array)?
        }
        // Under another spelling an array is the bound value itself.
        Expression::ArrayExpression(_) => return Err(JsxRefusal::Attribute),
        target if is_assignable(target) => Model {
            target: target.span(),
            argument,
            modifiers,
        },
        _ => return Err(JsxRefusal::Attribute),
    };
    push(cx, model, at, owner, out)
}

/// `v-models={[[x, 'arg', ['m']], …]}`: one `ui.model` per entry, on a
/// component.
pub(super) fn models<'a>(
    cx: &mut Cx<'a, '_>,
    value: Option<&Expression<'_>>,
    at: oxc_span::Span,
    owner: Owner<'a, '_, '_>,
    out: &mut Props<'a>,
) -> Result<(), JsxRefusal> {
    let (Owner::Component, Some(Expression::ArrayExpression(entries))) = (owner, value) else {
        return Err(JsxRefusal::Attribute);
    };
    if entries.elements.is_empty() {
        return Err(JsxRefusal::Attribute);
    }
    for entry in &entries.elements {
        let Some(Expression::ArrayExpression(entry)) = entry.as_expression() else {
            return Err(JsxRefusal::Attribute);
        };
        let model = array_model(cx, entry)?;
        push(cx, model, at, owner, out)?;
    }
    Ok(())
}

/// The array spelling with an assignable target.
fn array_model<'a>(cx: &Cx<'a, '_>, array: &ArrayExpression<'_>) -> Result<Model<'a>, JsxRefusal> {
    let spelling = ArraySpelling::of(array).ok_or(JsxRefusal::Attribute)?;
    if !is_assignable(spelling.value) {
        return Err(JsxRefusal::Attribute);
    }
    let mut modifiers = Vec::new_in(&cx.allocator);
    spelling.modifiers(cx, &mut modifiers);
    Ok(Model {
        target: spelling.value.span(),
        argument: spelling.argument(cx),
        modifiers,
    })
}

/// The owner's `ui.model`. A component takes any argument and modifiers; a
/// native `input`, `select` or `textarea` takes no argument and only `lazy`,
/// `number` and `trim`.
fn push<'a>(
    cx: &mut Cx<'a, '_>,
    model: Model<'a>,
    at: oxc_span::Span,
    owner: Owner<'a, '_, '_>,
    out: &mut Props<'a>,
) -> Result<(), JsxRefusal> {
    let element_kind = match owner {
        Owner::Component => COMPONENT,
        Owner::Element { tag, items } => {
            let native = match tag {
                "input" => input_type_admits_model(cx, items)?,
                "select" | "textarea" => true,
                _ => false,
            };
            let modifiers_are_native = model
                .modifiers
                .iter()
                .all(|modifier| matches!(*modifier, "lazy" | "number" | "trim"));
            // A target spelled as an array (`[a][0]`) reads as a model tuple.
            let tuple = cx.slice(model.target)?.trim_start().starts_with('[');
            if !native || model.argument.is_some() || !modifiers_are_native || tuple {
                return Err(JsxRefusal::Attribute);
            }
            tag
        }
    };
    cx.mint();
    let value = cx.expr(model.target)?;
    let mut attributes = Vec::new_in(&cx.allocator);
    attributes.push(Attribute {
        name: "element-kind",
        value: Some(element_kind),
        span: span(at),
    });
    for modifier in &model.modifiers {
        attributes.push(Attribute {
            name: modifier,
            value: None,
            span: span(at),
        });
    }
    cx.observe(OpFamily::Model);
    out.bindings.push(BindingOp::Model(Box::new_in(
        ModelOp {
            contract: BindingContract {
                read: value,
                write: value,
            },
            argument: model.argument.map(DynamicName::Static),
            attributes,
            span: span(at),
        },
        &cx.allocator,
    )));
    Ok(())
}

/// Whether an `input`'s `type` keeps one native model: every `type` is the
/// same static `checkbox`, `radio` or `text`, and no spread or bound `type`
/// can change it.
fn input_type_admits_model(
    cx: &Cx<'_, '_>,
    items: &[JSXAttributeItem<'_>],
) -> Result<bool, JsxRefusal> {
    let mut static_type = None;
    for item in items {
        let attribute = match item {
            JSXAttributeItem::SpreadAttribute(_) => return Ok(false),
            JSXAttributeItem::Attribute(attribute) => attribute,
        };
        let JSXAttributeName::Identifier(name) = &attribute.name else {
            continue;
        };
        if cx.slice(name.span)? != "type" {
            continue;
        }
        let Some(JSXAttributeValue::StringLiteral(value)) = &attribute.value else {
            return Ok(false);
        };
        let value = value.value.as_str();
        if !matches!(value, "checkbox" | "radio" | "text")
            || static_type.is_some_and(|seen| seen != value)
        {
            return Ok(false);
        }
        static_type = Some(value);
    }
    Ok(true)
}

/// Whether `expression` can be written back to: an identifier, a member
/// access or a private field, through parentheses and type-only wrappers.
fn is_assignable(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::Identifier(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::PrivateFieldExpression(_)
        | Expression::ChainExpression(_) => true,
        Expression::ParenthesizedExpression(inner) => is_assignable(&inner.expression),
        Expression::TSAsExpression(inner) => is_assignable(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => is_assignable(&inner.expression),
        Expression::TSNonNullExpression(inner) => is_assignable(&inner.expression),
        Expression::TSTypeAssertion(inner) => is_assignable(&inner.expression),
        _ => false,
    }
}
