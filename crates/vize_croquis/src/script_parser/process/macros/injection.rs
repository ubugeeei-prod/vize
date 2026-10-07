//! Recognize the direct and imported-alias forms of inject.

use super::{Expression, ScriptParseResult};

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
