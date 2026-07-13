use oxc_ast::ast::{Argument, CallExpression, Expression, ObjectPropertyKind};
use oxc_span::GetSpan;

use crate::{ModuleExpression, ModuleExpressionKind, ModuleLiteralKind, ModuleObjectProperty};

use super::{absolute, pattern::formal_parameters, slice};

pub(super) fn expression_snapshot(
    expression: &Expression<'_>,
    source: &str,
    base: u32,
) -> ModuleExpression {
    let span = absolute(expression.span(), base);
    let kind = match expression {
        Expression::Identifier(identifier) => {
            ModuleExpressionKind::Identifier(identifier.name.as_str().into())
        }
        Expression::StringLiteral(literal) => literal_kind(
            ModuleLiteralKind::String,
            expression.span(),
            source,
            Some(literal.value.as_str()),
        ),
        Expression::BooleanLiteral(literal) => literal_kind(
            ModuleLiteralKind::Boolean,
            expression.span(),
            source,
            Some(if literal.value { "true" } else { "false" }),
        ),
        Expression::NumericLiteral(_) => {
            literal_kind(ModuleLiteralKind::Number, expression.span(), source, None)
        }
        Expression::BigIntLiteral(_) => {
            literal_kind(ModuleLiteralKind::BigInt, expression.span(), source, None)
        }
        Expression::NullLiteral(_) => literal_kind(
            ModuleLiteralKind::Null,
            expression.span(),
            source,
            Some("null"),
        ),
        Expression::TemplateLiteral(_) => {
            literal_kind(ModuleLiteralKind::Template, expression.span(), source, None)
        }
        Expression::CallExpression(call) => return call_snapshot(call, source, base),
        Expression::ObjectExpression(object) => ModuleExpressionKind::Object {
            properties: object
                .properties
                .iter()
                .map(|property| object_property(property, source, base))
                .collect(),
        },
        Expression::ArrayExpression(array) => ModuleExpressionKind::Array(
            array
                .elements
                .iter()
                .map(|item| {
                    item.as_expression()
                        .map(|expression| expression_snapshot(expression, source, base))
                        .or_else(|| match item {
                            oxc_ast::ast::ArrayExpressionElement::SpreadElement(spread) => {
                                Some(spread_expression(&spread.argument, source, base))
                            }
                            _ => None,
                        })
                })
                .collect(),
        ),
        Expression::ArrowFunctionExpression(function) => ModuleExpressionKind::Function {
            async_: function.r#async,
            parameters: formal_parameters(&function.params, source, base),
        },
        Expression::FunctionExpression(function) => ModuleExpressionKind::Function {
            async_: function.r#async,
            parameters: formal_parameters(&function.params, source, base),
        },
        Expression::AwaitExpression(awaited) => ModuleExpressionKind::Await(Box::new(
            expression_snapshot(&awaited.argument, source, base),
        )),
        Expression::ParenthesizedExpression(inner) => {
            return expression_snapshot(&inner.expression, source, base);
        }
        Expression::TSAsExpression(inner) => {
            return expression_snapshot(&inner.expression, source, base);
        }
        Expression::TSSatisfiesExpression(inner) => {
            return expression_snapshot(&inner.expression, source, base);
        }
        Expression::TSNonNullExpression(inner) => {
            return expression_snapshot(&inner.expression, source, base);
        }
        _ => static_path(expression)
            .map(ModuleExpressionKind::Path)
            .unwrap_or_else(|| {
                ModuleExpressionKind::Unknown(slice(source, expression.span()).into())
            }),
    };
    ModuleExpression { kind, span }
}

pub(super) fn call_snapshot(
    call: &CallExpression<'_>,
    source: &str,
    base: u32,
) -> ModuleExpression {
    let arguments = call
        .arguments
        .iter()
        .map(|argument| argument_snapshot(argument, source, base))
        .collect();
    ModuleExpression {
        kind: ModuleExpressionKind::Call {
            callee: Box::new(expression_snapshot(&call.callee, source, base)),
            arguments,
            type_arguments: call
                .type_arguments
                .as_ref()
                .map(|arguments| slice(source, arguments.span).into()),
        },
        span: absolute(call.span, base),
    }
}

fn argument_snapshot(argument: &Argument<'_>, source: &str, base: u32) -> ModuleExpression {
    argument.as_expression().map_or_else(
        || match argument {
            Argument::SpreadElement(spread) => spread_expression(&spread.argument, source, base),
            _ => unknown(source, argument.span(), base),
        },
        |expression| expression_snapshot(expression, source, base),
    )
}

fn spread_expression(expression: &Expression<'_>, source: &str, base: u32) -> ModuleExpression {
    ModuleExpression {
        kind: ModuleExpressionKind::Spread(Box::new(expression_snapshot(expression, source, base))),
        span: absolute(expression.span(), base),
    }
}

fn object_property(
    property: &ObjectPropertyKind<'_>,
    source: &str,
    base: u32,
) -> ModuleObjectProperty {
    match property {
        ObjectPropertyKind::ObjectProperty(property) => ModuleObjectProperty {
            key: Some(slice(source, property.key.span()).into()),
            value: expression_snapshot(&property.value, source, base),
            spread: false,
        },
        ObjectPropertyKind::SpreadProperty(spread) => ModuleObjectProperty {
            key: None,
            value: expression_snapshot(&spread.argument, source, base),
            spread: true,
        },
    }
}

pub(super) fn static_path(expression: &Expression<'_>) -> Option<Vec<Box<str>>> {
    match expression {
        Expression::Identifier(identifier) => Some(vec![identifier.name.as_str().into()]),
        Expression::StaticMemberExpression(member) => {
            let mut path = static_path(&member.object)?;
            path.push(member.property.name.as_str().into());
            Some(path)
        }
        Expression::ParenthesizedExpression(inner) => static_path(&inner.expression),
        Expression::TSAsExpression(inner) => static_path(&inner.expression),
        Expression::TSNonNullExpression(inner) => static_path(&inner.expression),
        _ => None,
    }
}

fn literal_kind(
    kind: ModuleLiteralKind,
    span: oxc_span::Span,
    source: &str,
    value: Option<&str>,
) -> ModuleExpressionKind {
    ModuleExpressionKind::Literal {
        kind,
        text: slice(source, span).into(),
        value: value.map(Into::into),
    }
}

fn unknown(source: &str, span: oxc_span::Span, base: u32) -> ModuleExpression {
    ModuleExpression {
        kind: ModuleExpressionKind::Unknown(slice(source, span).into()),
        span: absolute(span, base),
    }
}
