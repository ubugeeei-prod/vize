use oxc_ast::ast::{BindingPattern, Expression, FormalParameters, Statement};

use super::{CsfRender, unwrap_expression};

/// Reach the single JSX-returning expression of a `render` arrow/function.
pub(super) fn render_body<'a>(value: &'a Expression<'a>) -> Option<CsfRender<'a>> {
    match unwrap_expression(value) {
        Expression::ArrowFunctionExpression(arrow) => {
            let args_name = first_param_name(&arrow.params);
            let body = if arrow.expression {
                first_expression_statement(&arrow.body.statements)
            } else {
                first_return_argument(&arrow.body.statements)
            }?;
            render_expression(body, args_name)
        }
        Expression::FunctionExpression(func) => {
            let args_name = first_param_name(&func.params);
            render_expression(
                func.body
                    .as_ref()
                    .and_then(|body| first_return_argument(&body.statements))?,
                args_name,
            )
        }
        _ => None,
    }
}

fn render_expression<'a>(
    expr: &'a Expression<'a>,
    args_name: Option<&'a str>,
) -> Option<CsfRender<'a>> {
    match unwrap_expression(expr) {
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => {
            let mut render = render_body(expr)?;
            if render.args_name.is_none() {
                render.args_name = args_name;
            }
            Some(render)
        }
        other => Some(CsfRender {
            expression: other,
            args_name,
        }),
    }
}

fn first_param_name<'a>(params: &'a FormalParameters<'a>) -> Option<&'a str> {
    let param = params.items.first()?;
    binding_identifier_name(&param.pattern)
}

fn binding_identifier_name<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn first_expression_statement<'a>(statements: &'a [Statement<'a>]) -> Option<&'a Expression<'a>> {
    match statements.first()? {
        Statement::ExpressionStatement(stmt) => Some(&stmt.expression),
        _ => None,
    }
}

fn first_return_argument<'a>(statements: &'a [Statement<'a>]) -> Option<&'a Expression<'a>> {
    match statements.first()? {
        Statement::ReturnStatement(stmt) => stmt.argument.as_ref(),
        _ => None,
    }
}
