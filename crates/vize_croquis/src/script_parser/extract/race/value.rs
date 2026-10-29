use oxc_ast::ast::{Expression, ObjectPropertyKind};

use super::super::super::ScriptParseResult;
use super::scan::{RaceScan, is_live_source};

#[derive(Default)]
struct ExpressionValueFacts {
    current_live: bool,
    stale: bool,
}

pub(super) fn expression_is_current_live(
    result: &ScriptParseResult,
    expression: &Expression<'_>,
    scan: &RaceScan,
) -> bool {
    let mut facts = ExpressionValueFacts::default();
    collect_expression_value_facts(result, expression, scan, &mut facts);
    facts.current_live && !facts.stale
}

fn collect_expression_value_facts(
    result: &ScriptParseResult,
    expression: &Expression<'_>,
    scan: &RaceScan,
    facts: &mut ExpressionValueFacts,
) {
    match expression {
        Expression::Identifier(identifier) => {
            if let Some(local) = scan.locals.get(identifier.name.as_str()) {
                if local.async_derived || local.generation < scan.async_generation {
                    facts.stale = true;
                } else if local.current_live {
                    facts.current_live = true;
                }
            } else if is_live_source(result, identifier.name.as_str()) {
                facts.current_live = true;
            }
        }
        Expression::AwaitExpression(await_expression) => {
            facts.stale = true;
            collect_expression_value_facts(result, &await_expression.argument, scan, facts);
        }
        Expression::CallExpression(call) => {
            collect_expression_value_facts(result, &call.callee, scan, facts);
            for argument in &call.arguments {
                if let Some(expression) = argument.as_expression() {
                    collect_expression_value_facts(result, expression, scan, facts);
                }
            }
        }
        Expression::StaticMemberExpression(member) => {
            collect_expression_value_facts(result, &member.object, scan, facts);
        }
        Expression::ComputedMemberExpression(member) => {
            collect_expression_value_facts(result, &member.object, scan, facts);
            collect_expression_value_facts(result, &member.expression, scan, facts);
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::CallExpression(call) => {
                collect_expression_value_facts(result, &call.callee, scan, facts);
                for argument in &call.arguments {
                    if let Some(expression) = argument.as_expression() {
                        collect_expression_value_facts(result, expression, scan, facts);
                    }
                }
            }
            oxc_ast::ast::ChainElement::TSNonNullExpression(non_null) => {
                collect_expression_value_facts(result, &non_null.expression, scan, facts);
            }
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                collect_expression_value_facts(result, &member.object, scan, facts);
            }
            oxc_ast::ast::ChainElement::ComputedMemberExpression(member) => {
                collect_expression_value_facts(result, &member.object, scan, facts);
                collect_expression_value_facts(result, &member.expression, scan, facts);
            }
            oxc_ast::ast::ChainElement::PrivateFieldExpression(field) => {
                collect_expression_value_facts(result, &field.object, scan, facts);
            }
        },
        Expression::ConditionalExpression(conditional) => {
            collect_expression_value_facts(result, &conditional.test, scan, facts);
            collect_expression_value_facts(result, &conditional.consequent, scan, facts);
            collect_expression_value_facts(result, &conditional.alternate, scan, facts);
        }
        Expression::LogicalExpression(logical) => {
            collect_expression_value_facts(result, &logical.left, scan, facts);
            collect_expression_value_facts(result, &logical.right, scan, facts);
        }
        Expression::BinaryExpression(binary) => {
            collect_expression_value_facts(result, &binary.left, scan, facts);
            collect_expression_value_facts(result, &binary.right, scan, facts);
        }
        Expression::ArrayExpression(array) => {
            for element in &array.elements {
                if let Some(expression) = element.as_expression() {
                    collect_expression_value_facts(result, expression, scan, facts);
                }
            }
        }
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                match property {
                    ObjectPropertyKind::ObjectProperty(property) => {
                        collect_expression_value_facts(result, &property.value, scan, facts);
                    }
                    ObjectPropertyKind::SpreadProperty(spread) => {
                        collect_expression_value_facts(result, &spread.argument, scan, facts);
                    }
                }
            }
        }
        Expression::UnaryExpression(unary) => {
            collect_expression_value_facts(result, &unary.argument, scan, facts);
        }
        Expression::SequenceExpression(sequence) => {
            for expression in &sequence.expressions {
                collect_expression_value_facts(result, expression, scan, facts);
            }
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            collect_expression_value_facts(result, &parenthesized.expression, scan, facts);
        }
        Expression::TSAsExpression(as_expression) => {
            collect_expression_value_facts(result, &as_expression.expression, scan, facts);
        }
        Expression::TSSatisfiesExpression(satisfies) => {
            collect_expression_value_facts(result, &satisfies.expression, scan, facts);
        }
        Expression::TSNonNullExpression(non_null) => {
            collect_expression_value_facts(result, &non_null.expression, scan, facts);
        }
        _ => {}
    }
}
