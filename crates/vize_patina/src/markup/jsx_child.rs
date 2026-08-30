use super::{MarkupElement, MarkupNode, MarkupText, span_to_range};
use oxc_ast::ast::{
    ArrowFunctionExpression, CallExpression, ConditionalExpression, Expression, Function, JSXChild,
    JSXExpression, LogicalOperator, Statement,
};

impl<'a> MarkupNode<'a> {
    pub(super) fn from_jsx_child(child: &'a JSXChild<'a>, offset: u32) -> Self {
        match child {
            JSXChild::Text(text) => Self::Text(MarkupText::from_jsx(&**text as *const _, offset)),
            JSXChild::Element(element) => Self::Element(MarkupElement::from_jsx_element(
                &**element as *const _,
                offset,
            )),
            JSXChild::Fragment(fragment) => Self::Element(MarkupElement::from_jsx_fragment(
                &**fragment as *const _,
                offset,
            )),
            JSXChild::ExpressionContainer(container) => match &container.expression {
                JSXExpression::EmptyExpression(_) => {
                    Self::Comment(span_to_range(container.span, offset))
                }
                JSXExpression::StringLiteral(string) => Self::Text(MarkupText::from_static(
                    string.value.as_str(),
                    span_to_range(string.span, offset),
                )),
                expression if jsx_expression_is_conditional_markup(expression) => {
                    Self::If(span_to_range(container.span, offset))
                }
                expression if jsx_expression_is_list_markup(expression) => {
                    Self::For(span_to_range(container.span, offset))
                }
                _ => Self::Interpolation(span_to_range(container.span, offset)),
            },
            JSXChild::Spread(spread) => Self::Interpolation(span_to_range(spread.span, offset)),
        }
    }
}

fn jsx_expression_is_conditional_markup(expr: &JSXExpression<'_>) -> bool {
    let Some(expr) = jsx_expression_as_expression(expr) else {
        return false;
    };
    match unwrap_jsx_parens(expr) {
        Expression::LogicalExpression(logical) => {
            logical.operator == LogicalOperator::And && expression_is_direct_jsx(&logical.right)
        }
        Expression::ConditionalExpression(conditional) => conditional_has_jsx_arm(conditional),
        _ => false,
    }
}

fn jsx_expression_is_list_markup(expr: &JSXExpression<'_>) -> bool {
    let Some(Expression::CallExpression(call)) =
        jsx_expression_as_expression(expr).map(unwrap_jsx_parens)
    else {
        return false;
    };
    map_call_returns_render_child(call)
}

fn map_call_returns_render_child(call: &CallExpression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) = unwrap_jsx_parens(&call.callee) else {
        return false;
    };
    if member.property.name.as_str() != "map" || member.optional || call.arguments.len() != 1 {
        return false;
    }

    let Some(argument) = call
        .arguments
        .first()
        .and_then(|argument| argument.as_expression())
    else {
        return false;
    };

    match unwrap_jsx_parens(argument) {
        Expression::ArrowFunctionExpression(arrow) => arrow_returns_render_child(arrow),
        Expression::FunctionExpression(function) => function_returns_render_child(function),
        _ => false,
    }
}

fn arrow_returns_render_child(arrow: &ArrowFunctionExpression<'_>) -> bool {
    if arrow.expression {
        let Some(Statement::ExpressionStatement(statement)) = arrow.body.statements.first() else {
            return false;
        };
        expression_returns_render_child(&statement.expression)
    } else {
        statements_return_render_child(&arrow.body.statements)
    }
}

fn function_returns_render_child(function: &Function<'_>) -> bool {
    let Some(body) = function.body.as_ref() else {
        return false;
    };
    statements_return_render_child(&body.statements)
}

fn statements_return_render_child(statements: &[Statement<'_>]) -> bool {
    statements.iter().any(|statement| {
        let Statement::ReturnStatement(statement) = statement else {
            return false;
        };
        statement
            .argument
            .as_ref()
            .is_some_and(|argument| expression_returns_render_child(argument))
    })
}

fn expression_returns_render_child(expr: &Expression<'_>) -> bool {
    match unwrap_jsx_parens(expr) {
        Expression::JSXElement(_) | Expression::JSXFragment(_) => true,
        Expression::LogicalExpression(_) | Expression::ConditionalExpression(_) => {
            expression_is_conditional_markup(expr)
        }
        Expression::CallExpression(call) => map_call_returns_render_child(call),
        _ => false,
    }
}

fn expression_is_conditional_markup(expr: &Expression<'_>) -> bool {
    match unwrap_jsx_parens(expr) {
        Expression::LogicalExpression(logical) => {
            logical.operator == LogicalOperator::And && expression_is_direct_jsx(&logical.right)
        }
        Expression::ConditionalExpression(conditional) => conditional_has_jsx_arm(conditional),
        _ => false,
    }
}

fn conditional_has_jsx_arm(conditional: &ConditionalExpression<'_>) -> bool {
    if expression_is_direct_jsx(&conditional.consequent) {
        return true;
    }
    match unwrap_jsx_parens(&conditional.alternate) {
        Expression::ConditionalExpression(nested) => conditional_has_jsx_arm(nested),
        other => expression_is_direct_jsx(other),
    }
}

fn expression_is_direct_jsx(expr: &Expression<'_>) -> bool {
    matches!(
        unwrap_jsx_parens(expr),
        Expression::JSXElement(_) | Expression::JSXFragment(_)
    )
}

fn unwrap_jsx_parens<'e, 'a>(mut expr: &'e Expression<'a>) -> &'e Expression<'a> {
    while let Expression::ParenthesizedExpression(inner) = expr {
        expr = &inner.expression;
    }
    expr
}

fn jsx_expression_as_expression<'e, 'a>(expr: &'e JSXExpression<'a>) -> Option<&'e Expression<'a>> {
    match expr {
        JSXExpression::EmptyExpression(_) => None,
        other => other.as_expression(),
    }
}
