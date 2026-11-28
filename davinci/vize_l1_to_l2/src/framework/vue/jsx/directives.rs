//! `v-*` attributes as the ops their template spellings produce: `ui.model`
//! ([`super::model`]), `vue.show`, `vue.html`, `vue.text` and
//! `vue.directive`.
//!
//! A custom directive takes its argument from `v-name:arg` or from the
//! array spelling `v-name={[x, 'arg', ['m']]}`. A `_` in a directive name
//! or argument waits for the `_`-modifier split (#12313); `v-slots`,
//! `v-slot`, `v-if`, `v-for`, `v-once`, `v-memo`, `v-pre`, `v-cloak`,
//! `v-bind` and `v-on` wait for their slices.

use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, Expression, JSXAttribute, JSXAttributeName,
    JSXAttributeValue, JSXExpression, StringLiteral,
};
use oxc_span::GetSpan;
use vize_l0::{Box, Vec};
use vize_l2::op::{BindingOp, DynamicName, VueDirectiveOp, VueHtmlOp, VueShowOp, VueTextOp};

use super::attributes::{Owner, Props};
use super::children::contains_jsx;
use super::model::{model, models};
use super::{Cx, JsxRefusal, span};

/// Lower the `v-*` attribute `attribute` of `owner` into `out`.
pub(super) fn directive<'a>(
    cx: &mut Cx<'a, '_, impl super::syntax::SyntaxSink>,
    attribute: &JSXAttribute<'_>,
    owner: Owner<'a, '_, '_>,
    out: &mut Props<'a>,
) -> Result<(), JsxRefusal> {
    let (name, argument) = match &attribute.name {
        JSXAttributeName::Identifier(name) => (cx.slice(name.span)?, None),
        JSXAttributeName::NamespacedName(name) => (
            cx.slice(name.namespace.span)?,
            Some(cx.slice(name.name.span)?),
        ),
    };
    let name = name.strip_prefix("v-").ok_or(JsxRefusal::Attribute)?;
    let value = match &attribute.value {
        None => None,
        Some(JSXAttributeValue::ExpressionContainer(container)) => match &container.expression {
            JSXExpression::EmptyExpression(_) => None,
            expression => {
                let expression = expression.as_expression().ok_or(JsxRefusal::Attribute)?;
                if contains_jsx(expression) {
                    return Err(JsxRefusal::Attribute);
                }
                Some(expression)
            }
        },
        // A string value would be read as an expression; markup is a slot.
        Some(
            JSXAttributeValue::StringLiteral(_)
            | JSXAttributeValue::Element(_)
            | JSXAttributeValue::Fragment(_),
        ) => return Err(JsxRefusal::Attribute),
    };
    let at = attribute.span;
    if name == "model" || name.starts_with("model_") {
        return model(cx, name, argument, value, at, owner, out);
    }
    if name == "models" && argument.is_none() {
        return models(cx, value, at, owner, out);
    }
    if argument.is_some() && is_built_in(name) {
        return Err(JsxRefusal::Attribute);
    }
    let op = match (name, owner) {
        ("show", _) => BindingOp::VueShow(Box::new_in(
            VueShowOp {
                value: required(cx, value)?,
                span: span(at),
            },
            &cx.allocator,
        )),
        ("html", Owner::Element { .. }) => BindingOp::VueHtml(Box::new_in(
            VueHtmlOp {
                value: Some(required(cx, value)?),
                span: span(at),
            },
            &cx.allocator,
        )),
        ("text", Owner::Element { .. }) => BindingOp::VueText(Box::new_in(
            VueTextOp {
                value: Some(required(cx, value)?),
                span: span(at),
            },
            &cx.allocator,
        )),
        (custom, Owner::Element { .. })
            if !is_built_in(custom)
                && !custom.contains('_')
                && !argument.is_some_and(|argument| argument.contains('_')) =>
        {
            custom_directive(cx, custom, argument, value, at)?
        }
        _ => return Err(JsxRefusal::Attribute),
    };
    cx.mint();
    out.bindings.push(op);
    Ok(())
}

/// A Vue directive, or the JSX `v-models` built on `v-model`.
fn is_built_in(name: &str) -> bool {
    vize_l0::is_builtin_directive(name) || name == "models"
}

fn required<'a>(
    cx: &Cx<'a, '_, impl super::syntax::SyntaxSink>,
    value: Option<&Expression<'_>>,
) -> Result<vize_l2::expr::ExprRef<'a>, JsxRefusal> {
    cx.expr(value.ok_or(JsxRefusal::Attribute)?.span())
}

/// `vue.directive` as authored. Without a `v-name:arg` argument, an array
/// `[x]`, `[x, 'arg']`, `[x, ['m']]` or `[x, 'arg', ['m']]` of string
/// literals spells the value, argument and modifiers; any other array is
/// the value itself.
fn custom_directive<'a>(
    cx: &Cx<'a, '_, impl super::syntax::SyntaxSink>,
    name: &'a str,
    argument: Option<&'a str>,
    value: Option<&Expression<'_>>,
    at: oxc_span::Span,
) -> Result<BindingOp<'a>, JsxRefusal> {
    let mut directive = VueDirectiveOp {
        name,
        argument: argument.map(DynamicName::Static),
        modifiers: Vec::new_in(&cx.allocator),
        value: value.map(|value| cx.expr(value.span())).transpose()?,
        span: span(at),
    };
    if argument.is_none()
        && let Some(Expression::ArrayExpression(array)) = value
        && let Some(spelling) = ArraySpelling::of(array)
    {
        directive.value = Some(cx.expr(spelling.value.span())?);
        directive.argument = spelling.argument(cx).map(DynamicName::Static);
        spelling.modifiers(cx, &mut directive.modifiers);
    }
    Ok(BindingOp::VueDirective(Box::new_in(
        directive,
        &cx.allocator,
    )))
}

/// Babel's array spelling of a directive: `[x]`, `[x, 'arg']`, `[x, ['m']]`
/// or `[x, 'arg', ['m']]`, with string literal modifiers.
pub(super) struct ArraySpelling<'e, 'x> {
    pub(super) value: &'e Expression<'x>,
    argument: Option<&'e StringLiteral<'x>>,
    modifiers: Option<&'e ArrayExpression<'x>>,
}

impl<'e, 'x> ArraySpelling<'e, 'x> {
    pub(super) fn of(array: &'e ArrayExpression<'x>) -> Option<Self> {
        use ArrayExpressionElement as Element;
        let (value, argument, modifiers) = match array.elements.as_slice() {
            [value] => (value, None, None),
            [value, Element::StringLiteral(argument)] => (value, Some(&**argument), None),
            [value, Element::ArrayExpression(listed)] => (value, None, Some(&**listed)),
            [
                value,
                Element::StringLiteral(argument),
                Element::ArrayExpression(listed),
            ] => (value, Some(&**argument), Some(&**listed)),
            _ => return None,
        };
        let strings = |listed: &ArrayExpression<'_>| {
            listed
                .elements
                .iter()
                .all(|modifier| matches!(modifier, Element::StringLiteral(_)))
        };
        if !modifiers.is_none_or(strings) {
            return None;
        }
        Some(Self {
            // Not a spread or a hole.
            value: value.as_expression()?,
            argument,
            modifiers,
        })
    }

    pub(super) fn argument<'a>(
        &self,
        cx: &Cx<'a, '_, impl super::syntax::SyntaxSink>,
    ) -> Option<&'a str> {
        self.argument
            .map(|argument| cx.allocator.alloc_str(argument.value.as_str()))
    }

    pub(super) fn modifiers<'a>(
        &self,
        cx: &Cx<'a, '_, impl super::syntax::SyntaxSink>,
        out: &mut Vec<'a, &'a str>,
    ) {
        for modifier in self.modifiers.iter().flat_map(|listed| &listed.elements) {
            if let ArrayExpressionElement::StringLiteral(modifier) = modifier {
                out.push(cx.allocator.alloc_str(modifier.value.as_str()));
            }
        }
    }
}
