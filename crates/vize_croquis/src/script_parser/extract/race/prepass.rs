use oxc_ast::ast::{AssignmentTarget, CallExpression, Expression, Program, Statement};
use vize_carton::{CompactString, FxHashSet, cstr};

use super::super::super::ScriptParseResult;

/// Prove the timer ownership shape that needs information from two separate
/// callbacks before the ordinary statement walk emits race facts.
pub(super) fn prepare_race_analysis(result: &mut ScriptParseResult, program: &Program<'_>) {
    let cleanup_timers = unmount_cleanup_timers(&program.body);
    if cleanup_timers.is_empty() {
        return;
    }

    mark_safe_intervals(
        &program.body,
        &cleanup_timers,
        &mut result.race_safe_call_spans,
    );
}

fn unmount_cleanup_timers(statements: &[Statement<'_>]) -> FxHashSet<CompactString> {
    let mut timers = FxHashSet::default();
    for statement in statements {
        let Statement::ExpressionStatement(expression) = statement else {
            continue;
        };
        let Some(call) = call_expression(&expression.expression) else {
            continue;
        };
        if !matches!(
            direct_call_name(call),
            Some("onBeforeUnmount" | "onUnmounted" | "onScopeDispose")
        ) {
            continue;
        }
        let Some(callback) = call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
        else {
            continue;
        };
        collect_clear_interval_calls(callback, &mut timers);
    }
    timers
}

fn collect_clear_interval_calls(
    expression: &Expression<'_>,
    timers: &mut FxHashSet<CompactString>,
) {
    match peel_expression(expression) {
        Expression::ArrowFunctionExpression(arrow) => {
            collect_clears_from_statements(&arrow.body.statements, timers);
        }
        Expression::FunctionExpression(function) => {
            if let Some(body) = &function.body {
                collect_clears_from_statements(&body.statements, timers);
            }
        }
        _ => {}
    }
}

fn collect_clears_from_statements(
    statements: &[Statement<'_>],
    timers: &mut FxHashSet<CompactString>,
) {
    for statement in statements {
        if let Some(timer) = guarded_clear_interval_timer(statement) {
            timers.insert(cstr!("{timer}"));
        }
        match statement {
            Statement::ExpressionStatement(expression) => {
                if let Some(timer) = clear_interval_timer(&expression.expression) {
                    timers.insert(cstr!("{timer}"));
                }
            }
            Statement::BlockStatement(block) => {
                collect_clears_from_statements(&block.body, timers);
            }
            _ => {}
        }
    }
}

fn mark_safe_intervals(
    statements: &[Statement<'_>],
    cleanup_timers: &FxHashSet<CompactString>,
    safe_calls: &mut FxHashSet<(u32, u32)>,
) {
    let mut cleared = FxHashSet::default();

    for statement in statements {
        if let Some(timer) = guarded_clear_interval_timer(statement) {
            cleared.insert(cstr!("{timer}"));
        }
        if let Statement::ExpressionStatement(expression) = statement {
            if let Some(timer) = clear_interval_timer(&expression.expression) {
                cleared.insert(cstr!("{timer}"));
            }

            if let Some((timer, call)) = assigned_interval(&expression.expression) {
                if cleared.contains(timer) && cleanup_timers.contains(timer) {
                    safe_calls.insert((call.span.start, call.span.end));
                }
                cleared.remove(timer);
            }

            mark_safe_intervals_in_expression(&expression.expression, cleanup_timers, safe_calls);
        }

        match statement {
            Statement::FunctionDeclaration(function) => {
                if let Some(body) = &function.body {
                    mark_safe_intervals(&body.statements, cleanup_timers, safe_calls);
                }
            }
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    if let Some(initializer) = &declarator.init {
                        mark_safe_intervals_in_expression(initializer, cleanup_timers, safe_calls);
                    }
                }
            }
            Statement::BlockStatement(block) => {
                mark_safe_intervals(&block.body, cleanup_timers, safe_calls);
            }
            Statement::IfStatement(if_statement) => {
                mark_safe_intervals_in_statement(
                    &if_statement.consequent,
                    cleanup_timers,
                    safe_calls,
                );
                if let Some(alternate) = &if_statement.alternate {
                    mark_safe_intervals_in_statement(alternate, cleanup_timers, safe_calls);
                }
            }
            Statement::TryStatement(try_statement) => {
                mark_safe_intervals(&try_statement.block.body, cleanup_timers, safe_calls);
                if let Some(handler) = &try_statement.handler {
                    mark_safe_intervals(&handler.body.body, cleanup_timers, safe_calls);
                }
                if let Some(finalizer) = &try_statement.finalizer {
                    mark_safe_intervals(&finalizer.body, cleanup_timers, safe_calls);
                }
            }
            _ => {}
        }
    }
}

fn mark_safe_intervals_in_statement(
    statement: &Statement<'_>,
    cleanup_timers: &FxHashSet<CompactString>,
    safe_calls: &mut FxHashSet<(u32, u32)>,
) {
    match statement {
        Statement::BlockStatement(block) => {
            mark_safe_intervals(&block.body, cleanup_timers, safe_calls);
        }
        _ => mark_safe_intervals(std::slice::from_ref(statement), cleanup_timers, safe_calls),
    }
}

fn mark_safe_intervals_in_expression(
    expression: &Expression<'_>,
    cleanup_timers: &FxHashSet<CompactString>,
    safe_calls: &mut FxHashSet<(u32, u32)>,
) {
    match peel_expression(expression) {
        Expression::ArrowFunctionExpression(arrow) => {
            mark_safe_intervals(&arrow.body.statements, cleanup_timers, safe_calls);
        }
        Expression::FunctionExpression(function) => {
            if let Some(body) = &function.body {
                mark_safe_intervals(&body.statements, cleanup_timers, safe_calls);
            }
        }
        Expression::CallExpression(call) => {
            for argument in &call.arguments {
                if let Some(expression) = argument.as_expression() {
                    mark_safe_intervals_in_expression(expression, cleanup_timers, safe_calls);
                }
            }
        }
        _ => {}
    }
}

fn assigned_interval<'b, 'a>(
    expression: &'b Expression<'a>,
) -> Option<(&'b str, &'b CallExpression<'a>)> {
    let Expression::AssignmentExpression(assignment) = peel_expression(expression) else {
        return None;
    };
    let AssignmentTarget::AssignmentTargetIdentifier(identifier) = &assignment.left else {
        return None;
    };
    let call = call_expression(&assignment.right)?;
    if direct_call_name(call) != Some("setInterval") || !interval_callback_is_synchronous(call) {
        return None;
    }
    Some((identifier.name.as_str(), call))
}

fn interval_callback_is_synchronous(call: &CallExpression<'_>) -> bool {
    call.arguments
        .first()
        .and_then(|argument| argument.as_expression())
        .is_some_and(|callback| match peel_expression(callback) {
            Expression::ArrowFunctionExpression(arrow) => !arrow.r#async,
            Expression::FunctionExpression(function) => !function.r#async,
            _ => false,
        })
}

fn guarded_clear_interval_timer<'b, 'a>(statement: &'b Statement<'a>) -> Option<&'b str> {
    let Statement::IfStatement(if_statement) = statement else {
        return None;
    };
    if if_statement.alternate.is_some() {
        return None;
    }
    let Expression::Identifier(test) = peel_expression(&if_statement.test) else {
        return None;
    };
    let cleared = clear_interval_statement(&if_statement.consequent)?;
    (test.name.as_str() == cleared).then_some(cleared)
}

fn clear_interval_statement<'b, 'a>(statement: &'b Statement<'a>) -> Option<&'b str> {
    match statement {
        Statement::ExpressionStatement(expression) => clear_interval_timer(&expression.expression),
        Statement::BlockStatement(block) if block.body.len() == 1 => {
            block.body.first().and_then(clear_interval_statement)
        }
        _ => None,
    }
}

fn clear_interval_timer<'b, 'a>(expression: &'b Expression<'a>) -> Option<&'b str> {
    let call = call_expression(expression)?;
    if direct_call_name(call) != Some("clearInterval") {
        return None;
    }
    let argument = call.arguments.first()?.as_expression()?;
    let Expression::Identifier(identifier) = peel_expression(argument) else {
        return None;
    };
    Some(identifier.name.as_str())
}

fn call_expression<'b, 'a>(expression: &'b Expression<'a>) -> Option<&'b CallExpression<'a>> {
    match peel_expression(expression) {
        Expression::CallExpression(call) => Some(call),
        _ => None,
    }
}

fn direct_call_name<'b, 'a>(call: &'b CallExpression<'a>) -> Option<&'b str> {
    match peel_expression(&call.callee) {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn peel_expression<'b, 'a>(expression: &'b Expression<'a>) -> &'b Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(parenthesized) => {
            peel_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(as_expression) => peel_expression(&as_expression.expression),
        Expression::TSSatisfiesExpression(satisfies) => peel_expression(&satisfies.expression),
        Expression::TSNonNullExpression(non_null) => peel_expression(&non_null.expression),
        _ => expression,
    }
}
