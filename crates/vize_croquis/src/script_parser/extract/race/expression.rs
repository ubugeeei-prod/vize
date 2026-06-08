use oxc_ast::ast::{
    AssignmentTarget, CallExpression, Expression, ObjectPropertyKind, SimpleAssignmentTarget,
};

use vize_carton::CompactString;

use super::super::super::ScriptParseResult;
use super::is_scheduler_api;
use super::scan::RaceScan;

pub(super) fn scan_expression_for_race(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
    scan: &mut RaceScan,
) {
    match expr {
        Expression::AwaitExpression(await_expr) => {
            scan.add_async_operation("await");
            scan_expression_for_race(result, &await_expr.argument, scan);
        }
        Expression::CallExpression(call) => {
            scan_call_expression_for_race(result, call, scan);
        }
        Expression::AssignmentExpression(assign) => {
            if let Some(target) = assignment_target_root(result, &assign.left) {
                scan.mutated_targets.insert(target);
            }
            scan_expression_for_race(result, &assign.right, scan);
        }
        Expression::UpdateExpression(update) => {
            if let Some(target) = simple_assignment_target_root(result, &update.argument) {
                scan.mutated_targets.insert(target);
            }
        }
        Expression::StaticMemberExpression(member) => {
            scan_expression_for_race(result, &member.object, scan);
        }
        Expression::ComputedMemberExpression(member) => {
            scan_expression_for_race(result, &member.object, scan);
            scan_expression_for_race(result, &member.expression, scan);
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::CallExpression(call) => {
                scan_call_expression_for_race(result, call, scan);
            }
            oxc_ast::ast::ChainElement::TSNonNullExpression(expr) => {
                scan_expression_for_race(result, &expr.expression, scan);
            }
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                scan_expression_for_race(result, &member.object, scan);
            }
            oxc_ast::ast::ChainElement::ComputedMemberExpression(member) => {
                scan_expression_for_race(result, &member.object, scan);
                scan_expression_for_race(result, &member.expression, scan);
            }
            oxc_ast::ast::ChainElement::PrivateFieldExpression(field) => {
                scan_expression_for_race(result, &field.object, scan);
            }
        },
        Expression::ConditionalExpression(cond) => {
            scan_expression_for_race(result, &cond.test, scan);
            scan_expression_for_race(result, &cond.consequent, scan);
            scan_expression_for_race(result, &cond.alternate, scan);
        }
        Expression::LogicalExpression(logical) => {
            scan_expression_for_race(result, &logical.left, scan);
            scan_expression_for_race(result, &logical.right, scan);
        }
        Expression::BinaryExpression(binary) => {
            scan_expression_for_race(result, &binary.left, scan);
            scan_expression_for_race(result, &binary.right, scan);
        }
        Expression::ArrayExpression(arr) => {
            for elem in arr.elements.iter() {
                if let Some(expr) = elem.as_expression() {
                    scan_expression_for_race(result, expr, scan);
                }
            }
        }
        Expression::ObjectExpression(obj) => {
            for prop in obj.properties.iter() {
                match prop {
                    ObjectPropertyKind::ObjectProperty(prop) => {
                        scan_expression_for_race(result, &prop.value, scan);
                    }
                    ObjectPropertyKind::SpreadProperty(spread) => {
                        scan_expression_for_race(result, &spread.argument, scan);
                    }
                }
            }
        }
        Expression::UnaryExpression(unary) => {
            scan_expression_for_race(result, &unary.argument, scan);
        }
        Expression::SequenceExpression(seq) => {
            for expr in seq.expressions.iter() {
                scan_expression_for_race(result, expr, scan);
            }
        }
        Expression::ParenthesizedExpression(paren) => {
            scan_expression_for_race(result, &paren.expression, scan);
        }
        Expression::TSAsExpression(ts_as) => {
            scan_expression_for_race(result, &ts_as.expression, scan);
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            scan_expression_for_race(result, &ts_satisfies.expression, scan);
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            scan_expression_for_race(result, &ts_non_null.expression, scan);
        }
        _ => {}
    }
}

fn scan_call_expression_for_race(
    result: &ScriptParseResult,
    call: &CallExpression<'_>,
    scan: &mut RaceScan,
) {
    if let Some(name) = super::super::common::resolved_call_name(result, call) {
        if name == "fetch" {
            scan.add_async_operation("fetch");
        } else if matches!(name.as_str(), "then" | "catch" | "finally") {
            scan.add_async_operation("promise callback");
        } else if is_scheduler_api(name.as_str()) {
            scan.add_async_operation(name.as_str());
        }

        if name == "onWatcherCleanup" || scan.cleanup_names.contains(name.as_str()) {
            scan.has_cleanup_call = true;
        }
    }

    if let Some(target) = mutation_call_target(result, call) {
        scan.mutated_targets.insert(target);
    }

    scan_expression_for_race(result, &call.callee, scan);
    for arg in call.arguments.iter() {
        if let Some(expr) = arg.as_expression() {
            scan_expression_for_race(result, expr, scan);
        }
    }
}

fn mutation_call_target(
    result: &ScriptParseResult,
    call: &CallExpression<'_>,
) -> Option<CompactString> {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };
    if !is_mutating_method(member.property.name.as_str()) {
        return None;
    }
    expression_reactive_root(result, &member.object)
}

fn assignment_target_root(
    result: &ScriptParseResult,
    target: &AssignmentTarget<'_>,
) -> Option<CompactString> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(id) => {
            tracked_mutation_root(result, id.name.as_str())
        }
        AssignmentTarget::StaticMemberExpression(member) => {
            expression_reactive_root(result, &member.object)
        }
        AssignmentTarget::ComputedMemberExpression(member) => {
            expression_reactive_root(result, &member.object)
        }
        _ => None,
    }
}

fn simple_assignment_target_root(
    result: &ScriptParseResult,
    target: &SimpleAssignmentTarget<'_>,
) -> Option<CompactString> {
    match target {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
            tracked_mutation_root(result, id.name.as_str())
        }
        SimpleAssignmentTarget::StaticMemberExpression(member) => {
            expression_reactive_root(result, &member.object)
        }
        SimpleAssignmentTarget::ComputedMemberExpression(member) => {
            expression_reactive_root(result, &member.object)
        }
        _ => None,
    }
}

fn expression_reactive_root(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
) -> Option<CompactString> {
    match expr {
        Expression::Identifier(id) => tracked_mutation_root(result, id.name.as_str()),
        Expression::StaticMemberExpression(member) => {
            expression_reactive_root(result, &member.object)
        }
        Expression::ComputedMemberExpression(member) => {
            expression_reactive_root(result, &member.object)
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                expression_reactive_root(result, &member.object)
            }
            oxc_ast::ast::ChainElement::ComputedMemberExpression(member) => {
                expression_reactive_root(result, &member.object)
            }
            oxc_ast::ast::ChainElement::PrivateFieldExpression(field) => {
                expression_reactive_root(result, &field.object)
            }
            oxc_ast::ast::ChainElement::TSNonNullExpression(expr) => {
                expression_reactive_root(result, &expr.expression)
            }
            _ => None,
        },
        _ => None,
    }
}

fn tracked_mutation_root(result: &ScriptParseResult, name: &str) -> Option<CompactString> {
    (result.reactivity.is_reactive(name) || result.inject_var_names.contains(name))
        .then(|| CompactString::new(name))
}

fn is_mutating_method(name: &str) -> bool {
    matches!(
        name,
        "push"
            | "pop"
            | "shift"
            | "unshift"
            | "splice"
            | "sort"
            | "reverse"
            | "fill"
            | "copyWithin"
            | "set"
            | "add"
            | "delete"
            | "clear"
    )
}
