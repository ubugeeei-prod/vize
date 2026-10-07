//! Exact Vue callback identities, without a new parse or semantic stage.

use oxc_ast::ast::{Argument, CallExpression, Expression, ObjectPropertyKind, PropertyKind};
use oxc_span::GetSpan;

use super::ScriptParseResult;

/// Record callback candidates; complete final occurrences establish Vue identity.
pub(super) fn note_call(result: &mut ScriptParseResult, call: &CallExpression<'_>) {
    if result.skip_diagnostics || result.occurrence_capture.is_none() {
        return;
    }
    let Expression::Identifier(identifier) = call.callee.get_inner_expression() else {
        return;
    };
    result
        .setup_context
        .note_ssr_call(identifier.span.start, identifier.span.end);
    if call.optional
        || call
            .arguments
            .iter()
            .any(|arg| matches!(arg, Argument::SpreadElement(_)))
    {
        return;
    }
    let callee = identifier.span;
    for (index, is_watch) in [(0, false), (1, true)] {
        if is_watch && !deferred_watch(call) {
            continue;
        }
        let Some(callback) = call
            .arguments
            .get(index)
            .and_then(Argument::as_expression)
            .map(Expression::get_inner_expression)
        else {
            continue;
        };
        if matches!(
            callback,
            Expression::Identifier(_)
                | Expression::ArrowFunctionExpression(_)
                | Expression::FunctionExpression(_)
        ) {
            let span = callback.span();
            result.setup_context.note_vue_callback(
                callee.start,
                callee.end,
                span.start,
                span.end,
                is_watch,
            );
        }
    }
}

fn deferred_watch(call: &CallExpression<'_>) -> bool {
    if call.arguments.len() == 2 {
        return true;
    }
    if call.arguments.len() != 3 {
        return false;
    }
    let Some(Expression::ObjectExpression(options)) = call
        .arguments
        .get(2)
        .and_then(Argument::as_expression)
        .map(Expression::get_inner_expression)
    else {
        return false;
    };
    let mut immediate_seen = false;
    let mut flush_seen = false;
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        if property.computed
            || property.method
            || property.shorthand
            || property.kind != PropertyKind::Init
        {
            return false;
        }
        let Some(name) = property.key.static_name() else {
            return false;
        };
        match name.as_ref() {
            "immediate" => {
                if immediate_seen
                    || !matches!(property.value.get_inner_expression(), Expression::BooleanLiteral(value) if !value.value)
                {
                    return false;
                }
                immediate_seen = true;
            }
            "flush" => {
                // Synchronous watchers can run after a source mutation during
                // SSR; unknown flush behavior cannot justify an exemption.
                if flush_seen
                    || !matches!(property.value.get_inner_expression(), Expression::StringLiteral(value) if matches!(value.value.as_str(), "pre" | "post"))
                {
                    return false;
                }
                flush_seen = true;
            }
            "__proto__" => return false,
            _ => {}
        }
    }
    true
}
