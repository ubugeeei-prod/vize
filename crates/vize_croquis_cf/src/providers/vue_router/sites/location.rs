//! A route location object literal, reduced to what the typing rule checks.

use oxc_ast::ast::{
    ArrayExpressionElement, Expression, IdentifierReference, ObjectExpression, ObjectPropertyKind,
    UnaryOperator,
};
use oxc_span::GetSpan;
use vize_carton::{CompactString, Span};

use super::{NavSite, Target};
use crate::providers::ModuleId;
use crate::providers::vue_router::records::router_options::static_string;

/// The `params` of a location.
#[derive(Debug)]
pub(in crate::providers::vue_router) enum Params {
    /// No `params` property.
    Absent,
    /// `params` is not an object literal of plain keys: no param claim.
    Unknown,
    /// `params: { … }`, every key static.
    Known { span: Span, entries: Vec<ParamArg> },
}

/// One `key: value` of a `params` object literal.
#[derive(Debug)]
pub(in crate::providers::vue_router) struct ParamArg {
    pub key: CompactString,
    pub key_span: Span,
    pub value_span: Span,
    pub value: ValueKind,
}

/// What a param value is, where the syntax alone decides it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::providers::vue_router) enum ValueKind {
    /// A string (`empty` when statically `''`).
    Text { empty: bool },
    /// A number literal.
    Number,
    /// An array literal (`empty` when `[]`).
    Array { empty: bool },
    /// `null` or `undefined`.
    Nullish(&'static str),
    /// A value of a type no param accepts, named for the message.
    Invalid(&'static str),
    /// Anything the syntax does not decide.
    Unknown,
}

/// The named navigation `location` describes, or `None` when it is not a
/// statically named location.
pub(super) fn named<'a>(
    module: ModuleId,
    target: Target,
    location: &ObjectExpression<'a>,
    span: &dyn Fn(oxc_span::Span) -> Span,
    global: &dyn Fn(&IdentifierReference<'a>) -> bool,
) -> Option<NavSite> {
    let mut name = None;
    let mut params = Params::Absent;
    for property in &location.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed {
            return None;
        }
        match property.key.static_name()?.as_ref() {
            "name" => name = Some((static_string(&property.value)?, span(property.value.span()))),
            "path" => return None,
            "params" => params = params_of(&property.value, span, global),
            _ => {}
        }
    }
    let (name, name_span) = name?;
    Some(NavSite {
        module,
        target,
        name,
        name_span,
        params,
    })
}

fn params_of<'a>(
    value: &Expression<'a>,
    span: &dyn Fn(oxc_span::Span) -> Span,
    global: &dyn Fn(&IdentifierReference<'a>) -> bool,
) -> Params {
    let Expression::ObjectExpression(object) = value.get_inner_expression() else {
        return Params::Unknown;
    };
    let mut entries = Vec::with_capacity(object.properties.len());
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return Params::Unknown;
        };
        if property.computed {
            return Params::Unknown;
        }
        let Some(key) = property.key.static_name() else {
            return Params::Unknown;
        };
        entries.push(ParamArg {
            key: CompactString::new(key.as_ref()),
            key_span: span(property.key.span()),
            value_span: span(property.value.span()),
            value: kind(&property.value, global),
        });
    }
    Params::Known {
        span: span(object.span),
        entries,
    }
}

fn kind<'a>(
    value: &Expression<'a>,
    global: &dyn Fn(&IdentifierReference<'a>) -> bool,
) -> ValueKind {
    match value.get_inner_expression() {
        Expression::StringLiteral(literal) => ValueKind::Text {
            empty: literal.value.is_empty(),
        },
        Expression::TemplateLiteral(literal) => {
            if literal
                .quasis
                .iter()
                .any(|quasi| !quasi.value.raw.is_empty())
            {
                ValueKind::Text { empty: false }
            } else if literal.expressions.is_empty() {
                ValueKind::Text { empty: true }
            } else {
                ValueKind::Unknown
            }
        }
        Expression::NumericLiteral(_) => ValueKind::Number,
        Expression::UnaryExpression(unary)
            if matches!(
                unary.operator,
                UnaryOperator::UnaryNegation | UnaryOperator::UnaryPlus
            ) && matches!(
                unary.argument.get_inner_expression(),
                Expression::NumericLiteral(_)
            ) =>
        {
            ValueKind::Number
        }
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::Void => {
            ValueKind::Nullish("undefined")
        }
        Expression::ArrayExpression(array) => ValueKind::Array {
            empty: array.elements.is_empty()
                || array
                    .elements
                    .iter()
                    .all(|element| matches!(element, ArrayExpressionElement::Elision(_))),
        },
        Expression::NullLiteral(_) => ValueKind::Nullish("null"),
        Expression::Identifier(ident) if ident.name == "undefined" && global(ident) => {
            ValueKind::Nullish("undefined")
        }
        Expression::BooleanLiteral(_) => ValueKind::Invalid("a boolean"),
        Expression::ObjectExpression(_) => ValueKind::Invalid("an object"),
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => {
            ValueKind::Invalid("a function")
        }
        Expression::BigIntLiteral(_) => ValueKind::Invalid("a bigint"),
        Expression::RegExpLiteral(_) => ValueKind::Invalid("a regular expression"),
        _ => ValueKind::Unknown,
    }
}
