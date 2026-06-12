use oxc_ast::ast::{Argument, ChainElement, Expression, ObjectPropertyKind, Statement};

use super::super::super::ScriptParseResult;
use super::ReactivePlainValue;

pub(super) fn record_reactive_plain_values_in_composable_arg(
    result: &mut ScriptParseResult,
    expr: &Expression<'_>,
    callee_name: &vize_carton::CompactString,
    source: &str,
) {
    if let Some(value) = super::sources::reactive_plain_value_from_expr(result, expr, source) {
        result.reactivity.record_function_argument_extract(
            value.source_name,
            value.argument_name,
            callee_name.clone(),
            value.start,
            value.end,
        );
        return;
    }

    match expr {
        Expression::ArrayExpression(arr) => {
            for elem in arr.elements.iter() {
                match elem {
                    oxc_ast::ast::ArrayExpressionElement::SpreadElement(spread) => {
                        record_reactive_plain_values_in_composable_arg(
                            result,
                            &spread.argument,
                            callee_name,
                            source,
                        );
                    }
                    oxc_ast::ast::ArrayExpressionElement::Elision(_) => {}
                    _ => {
                        if let Some(expr) = elem.as_expression() {
                            record_reactive_plain_values_in_composable_arg(
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
        Expression::AwaitExpression(await_expr) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &await_expr.argument,
                callee_name,
                source,
            );
        }
        Expression::BinaryExpression(binary) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &binary.left,
                callee_name,
                source,
            );
            record_reactive_plain_values_in_composable_arg(
                result,
                &binary.right,
                callee_name,
                source,
            );
        }
        Expression::CallExpression(call) => {
            for arg in call.arguments.iter() {
                record_argument_reactive_plain_values(result, arg, callee_name, source);
            }
        }
        Expression::ChainExpression(chain) => {
            record_chain_reactive_plain_values(result, &chain.expression, callee_name, source);
        }
        Expression::ObjectExpression(obj) => {
            for prop in obj.properties.iter() {
                match prop {
                    ObjectPropertyKind::ObjectProperty(prop) => {
                        if let Some(key) = prop.key.as_expression() {
                            record_reactive_plain_values_in_composable_arg(
                                result,
                                key,
                                callee_name,
                                source,
                            );
                        }
                        record_reactive_plain_values_in_composable_arg(
                            result,
                            &prop.value,
                            callee_name,
                            source,
                        );
                    }
                    ObjectPropertyKind::SpreadProperty(spread) => {
                        record_reactive_plain_values_in_composable_arg(
                            result,
                            &spread.argument,
                            callee_name,
                            source,
                        );
                    }
                }
            }
        }
        Expression::ComputedMemberExpression(member) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &member.object,
                callee_name,
                source,
            );
            record_reactive_plain_values_in_composable_arg(
                result,
                &member.expression,
                callee_name,
                source,
            );
        }
        Expression::StaticMemberExpression(member) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &member.object,
                callee_name,
                source,
            );
        }
        Expression::PrivateFieldExpression(field) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &field.object,
                callee_name,
                source,
            );
        }
        Expression::ConditionalExpression(cond) => {
            record_reactive_plain_values_in_composable_arg(result, &cond.test, callee_name, source);
            record_reactive_plain_values_in_composable_arg(
                result,
                &cond.consequent,
                callee_name,
                source,
            );
            record_reactive_plain_values_in_composable_arg(
                result,
                &cond.alternate,
                callee_name,
                source,
            );
        }
        Expression::LogicalExpression(logical) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &logical.left,
                callee_name,
                source,
            );
            record_reactive_plain_values_in_composable_arg(
                result,
                &logical.right,
                callee_name,
                source,
            );
        }
        Expression::NewExpression(new_expr) => {
            for arg in new_expr.arguments.iter() {
                record_argument_reactive_plain_values(result, arg, callee_name, source);
            }
        }
        Expression::TaggedTemplateExpression(tagged) => {
            for expr in tagged.quasi.expressions.iter() {
                record_reactive_plain_values_in_composable_arg(result, expr, callee_name, source);
            }
        }
        Expression::TemplateLiteral(template) => {
            for expr in template.expressions.iter() {
                record_reactive_plain_values_in_composable_arg(result, expr, callee_name, source);
            }
        }
        Expression::UnaryExpression(unary) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &unary.argument,
                callee_name,
                source,
            );
        }
        Expression::YieldExpression(yield_expr) => {
            if let Some(argument) = &yield_expr.argument {
                record_reactive_plain_values_in_composable_arg(
                    result,
                    argument,
                    callee_name,
                    source,
                );
            }
        }
        Expression::PrivateInExpression(private_in) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &private_in.right,
                callee_name,
                source,
            );
        }
        Expression::ImportExpression(import_expr) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &import_expr.source,
                callee_name,
                source,
            );
            if let Some(options) = &import_expr.options {
                record_reactive_plain_values_in_composable_arg(
                    result,
                    options,
                    callee_name,
                    source,
                );
            }
        }
        Expression::SequenceExpression(seq) => {
            for expr in seq.expressions.iter() {
                record_reactive_plain_values_in_composable_arg(result, expr, callee_name, source);
            }
        }
        Expression::ParenthesizedExpression(paren) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &paren.expression,
                callee_name,
                source,
            );
        }
        Expression::TSAsExpression(ts_as) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &ts_as.expression,
                callee_name,
                source,
            );
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &ts_satisfies.expression,
                callee_name,
                source,
            );
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &ts_non_null.expression,
                callee_name,
                source,
            );
        }
        Expression::TSTypeAssertion(ts_assertion) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &ts_assertion.expression,
                callee_name,
                source,
            );
        }
        Expression::TSInstantiationExpression(ts_instantiation) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &ts_instantiation.expression,
                callee_name,
                source,
            );
        }
        Expression::V8IntrinsicExpression(intrinsic) => {
            for arg in intrinsic.arguments.iter() {
                record_argument_reactive_plain_values(result, arg, callee_name, source);
            }
        }
        _ => {}
    }
}

fn record_argument_reactive_plain_values(
    result: &mut ScriptParseResult,
    arg: &Argument<'_>,
    callee_name: &vize_carton::CompactString,
    source: &str,
) {
    match arg {
        Argument::SpreadElement(spread) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &spread.argument,
                callee_name,
                source,
            );
        }
        _ => {
            if let Some(expr) = arg.as_expression() {
                record_reactive_plain_values_in_composable_arg(result, expr, callee_name, source);
            }
        }
    }
}

fn record_chain_reactive_plain_values(
    result: &mut ScriptParseResult,
    chain: &ChainElement<'_>,
    callee_name: &vize_carton::CompactString,
    source: &str,
) {
    match chain {
        ChainElement::CallExpression(call) => {
            for arg in call.arguments.iter() {
                record_argument_reactive_plain_values(result, arg, callee_name, source);
            }
        }
        ChainElement::TSNonNullExpression(ts_non_null) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &ts_non_null.expression,
                callee_name,
                source,
            );
        }
        ChainElement::ComputedMemberExpression(member) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &member.object,
                callee_name,
                source,
            );
            record_reactive_plain_values_in_composable_arg(
                result,
                &member.expression,
                callee_name,
                source,
            );
        }
        ChainElement::StaticMemberExpression(member) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &member.object,
                callee_name,
                source,
            );
        }
        ChainElement::PrivateFieldExpression(field) => {
            record_reactive_plain_values_in_composable_arg(
                result,
                &field.object,
                callee_name,
                source,
            );
        }
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
