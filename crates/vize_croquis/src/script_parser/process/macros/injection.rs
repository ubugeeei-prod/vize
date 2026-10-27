//! Recognize the direct and imported-alias forms of inject.

use super::{Argument, Expression, ScriptParseResult};

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
