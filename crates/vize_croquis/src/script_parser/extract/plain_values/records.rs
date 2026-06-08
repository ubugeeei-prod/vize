use oxc_ast::ast::{Expression, ObjectPropertyKind, Statement};

use vize_carton::CompactString;

use super::super::super::{ReactiveValueOrigin, ScriptParseResult};
use super::ReactivePlainValue;

pub(super) fn record_reactive_plain_values_in_call_arg(
    result: &mut ScriptParseResult,
    expr: &Expression<'_>,
    callee_name: &CompactString,
    source: &str,
) {
    if let Some(value) = super::sources::reactive_plain_value_from_expr(result, expr, source) {
        result.reactivity.record_function_argument_extract(
            value.source_name.clone(),
            value.argument_name.clone(),
            callee_name.clone(),
            value.start,
            value.end,
        );
        result.reactive_value_origins.insert(
            value.argument_name,
            ReactiveValueOrigin::FunctionArgument {
                source_name: value.source_name,
                callee_name: callee_name.clone(),
            },
        );
        return;
    }

    match expr {
        Expression::ArrayExpression(arr) => {
            for elem in arr.elements.iter() {
                match elem {
                    oxc_ast::ast::ArrayExpressionElement::SpreadElement(spread) => {
                        record_reactive_plain_values_in_call_arg(
                            result,
                            &spread.argument,
                            callee_name,
                            source,
                        );
                    }
                    oxc_ast::ast::ArrayExpressionElement::Elision(_) => {}
                    _ => {
                        if let Some(expr) = elem.as_expression() {
                            record_reactive_plain_values_in_call_arg(
                                result,
                                expr,
                                callee_name,
                                source,
                            );
                        }
                    }
                }
            }
        }
        Expression::ObjectExpression(obj) => {
            for prop in obj.properties.iter() {
                match prop {
                    ObjectPropertyKind::ObjectProperty(prop) => {
                        record_reactive_plain_values_in_call_arg(
                            result,
                            &prop.value,
                            callee_name,
                            source,
                        );
                    }
                    ObjectPropertyKind::SpreadProperty(spread) => {
                        record_reactive_plain_values_in_call_arg(
                            result,
                            &spread.argument,
                            callee_name,
                            source,
                        );
                    }
                }
            }
        }
        Expression::ConditionalExpression(cond) => {
            record_reactive_plain_values_in_call_arg(result, &cond.test, callee_name, source);
            record_reactive_plain_values_in_call_arg(result, &cond.consequent, callee_name, source);
            record_reactive_plain_values_in_call_arg(result, &cond.alternate, callee_name, source);
        }
        Expression::LogicalExpression(logical) => {
            record_reactive_plain_values_in_call_arg(result, &logical.left, callee_name, source);
            record_reactive_plain_values_in_call_arg(result, &logical.right, callee_name, source);
        }
        Expression::SequenceExpression(seq) => {
            for expr in seq.expressions.iter() {
                record_reactive_plain_values_in_call_arg(result, expr, callee_name, source);
            }
        }
        Expression::ParenthesizedExpression(paren) => {
            record_reactive_plain_values_in_call_arg(
                result,
                &paren.expression,
                callee_name,
                source,
            );
        }
        Expression::TSAsExpression(ts_as) => {
            record_reactive_plain_values_in_call_arg(
                result,
                &ts_as.expression,
                callee_name,
                source,
            );
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            record_reactive_plain_values_in_call_arg(
                result,
                &ts_satisfies.expression,
                callee_name,
                source,
            );
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            record_reactive_plain_values_in_call_arg(
                result,
                &ts_non_null.expression,
                callee_name,
                source,
            );
        }
        _ => {}
    }
}

pub(super) fn getter_source_from_function(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
    source: &str,
) -> Option<ReactivePlainValue> {
    let returned = match expr {
        Expression::ArrowFunctionExpression(arrow) => {
            if !arrow.params.items.is_empty() {
                return None;
            }
            arrow_return_expression(arrow)?
        }
        Expression::FunctionExpression(func) => {
            if !func.params.items.is_empty() {
                return None;
            }
            function_return_expression(func)?
        }
        Expression::ParenthesizedExpression(paren) => {
            return getter_source_from_function(result, &paren.expression, source);
        }
        Expression::TSAsExpression(ts_as) => {
            return getter_source_from_function(result, &ts_as.expression, source);
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            return getter_source_from_function(result, &ts_satisfies.expression, source);
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            return getter_source_from_function(result, &ts_non_null.expression, source);
        }
        _ => return None,
    };

    super::sources::reactive_plain_value_from_expr(result, returned, source)
}

fn arrow_return_expression<'a>(
    arrow: &'a oxc_ast::ast::ArrowFunctionExpression<'a>,
) -> Option<&'a Expression<'a>> {
    if !arrow.expression {
        return function_body_return_expression(&arrow.body.statements);
    }

    let Statement::ExpressionStatement(expr_stmt) = arrow.body.statements.first()? else {
        return None;
    };
    Some(&expr_stmt.expression)
}

fn function_return_expression<'a>(
    func: &'a oxc_ast::ast::Function<'a>,
) -> Option<&'a Expression<'a>> {
    function_body_return_expression(&func.body.as_ref()?.statements)
}

fn function_body_return_expression<'a>(
    statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
) -> Option<&'a Expression<'a>> {
    for stmt in statements.iter() {
        if let Statement::ReturnStatement(ret) = stmt
            && let Some(argument) = &ret.argument
        {
            return Some(argument);
        }
    }
    None
}
