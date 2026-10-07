//! Writes through snapshots, including live deep-ref member exclusions.

use oxc_ast::ast::{AssignmentTarget, CallExpression, Expression, SimpleAssignmentTarget};

use super::{ScriptParseResult, names::is_mutating_method, sources};

/// Check member writes through a plain snapshot of reactive state.
#[inline]
pub fn check_reactive_plain_assignment_mutation(
    result: &mut ScriptParseResult,
    target: &AssignmentTarget<'_>,
    source: &str,
) {
    if result.reactive_value_origins.is_empty() {
        return;
    }

    let Some(value) = sources::reactive_plain_value_from_assignment_target(result, target, source)
    else {
        return;
    };

    result.reactivity.record_plain_value_mutation(
        value.source_name,
        value.argument_name,
        value.start,
        value.end,
    );
}

/// Check updates of plain-snapshot bindings or their members.
#[inline]
pub fn check_reactive_plain_update_mutation(
    result: &mut ScriptParseResult,
    target: &SimpleAssignmentTarget<'_>,
    source: &str,
) {
    if result.reactive_value_origins.is_empty() {
        return;
    }

    let Some(value) =
        sources::reactive_plain_value_from_simple_assignment_target(result, target, source)
    else {
        return;
    };

    result.reactivity.record_plain_value_mutation(
        value.source_name,
        value.argument_name,
        value.start,
        value.end,
    );
}

/// Check mutating method calls like `alias.push(...)`, `alias.set(...)`, or
/// `alias.items.splice(...)` on a plain snapshot.
#[inline]
pub fn check_reactive_plain_call_mutation(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    source: &str,
) {
    if result.reactive_value_origins.is_empty() {
        return;
    }

    let Expression::StaticMemberExpression(member) = &call.callee else {
        return;
    };
    if !is_mutating_method(member.property.name.as_str()) {
        return;
    }

    let Some(value) =
        sources::reactive_plain_value_from_mutated_expression(result, &member.object, source)
    else {
        return;
    };

    if sources::keeps_live_ref_value(result, value.source_name.as_str()) {
        return;
    }

    result.reactivity.record_plain_value_mutation(
        value.source_name,
        value.argument_name,
        call.span.start,
        call.span.end,
    );
}
