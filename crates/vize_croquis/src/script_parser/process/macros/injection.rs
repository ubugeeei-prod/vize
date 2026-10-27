//! Recognize direct, imported-alias and toRefs-wrapped inject calls.

use oxc_ast::ast::CallExpression;

use super::{Argument, Expression, ScriptParseResult, extract_call_expression};

pub(super) fn is_inject_call(
    call: &oxc_ast::ast::CallExpression<'_>,
    result: &ScriptParseResult,
) -> bool {
    let Expression::Identifier(id) = &call.callee else {
        return false;
    };
    let callee_name = id.name.as_str();
    callee_name == "inject" || result.inject_aliases.contains(callee_name)
}

/// Preserve uncertainty from a spread without rescanning or reparsing source.
#[inline]
pub(super) fn record_default_spread(
    result: &mut ScriptParseResult,
    call: &oxc_ast::ast::CallExpression<'_>,
) {
    if matches!(call.arguments.get(1), Some(Argument::SpreadElement(_))) {
        result.provide_inject.mark_last_inject_default_spread();
    }
}

pub(super) fn extract_inject_call_from_torefs<'a>(
    init: &'a Expression<'a>,
    result: &ScriptParseResult,
) -> Option<&'a CallExpression<'a>> {
    let outer = extract_call_expression(init)?;
    if !is_torefs_call(outer, result) {
        return None;
    }

    let first_arg = outer.arguments.first()?;
    let inner = match first_arg {
        Argument::CallExpression(call) => Some(call.as_ref()),
        _ => first_arg.as_expression().and_then(extract_call_expression),
    }?;

    is_inject_call(inner, result).then_some(inner)
}

fn is_torefs_call(call: &CallExpression<'_>, result: &ScriptParseResult) -> bool {
    let Expression::Identifier(id) = &call.callee else {
        return false;
    };
    let callee_name = id.name.as_str();
    callee_name == "toRefs"
        || result
            .reactivity_aliases
            .get(callee_name)
            .is_some_and(|api_name| api_name.as_str() == "toRefs")
}
