use oxc_ast::ast::{CallExpression, Expression};
use oxc_span::{GetSpan, Span};
use vize_carton::CompactString;

use super::super::super::ScriptParseResult;

pub(in crate::script_parser::extract) fn resolved_call_name(
    result: &ScriptParseResult,
    call: &CallExpression<'_>,
) -> Option<CompactString> {
    let raw_name = match &call.callee {
        Expression::Identifier(id) => Some(id.name.as_str()),
        Expression::StaticMemberExpression(member) => Some(member.property.name.as_str()),
        Expression::ComputedMemberExpression(_) => None,
        _ => None,
    }?;

    Some(
        result
            .reactivity_aliases
            .get(raw_name)
            .cloned()
            .unwrap_or_else(|| CompactString::new(raw_name)),
    )
}

pub(in crate::script_parser::extract) fn call_label(
    result: &ScriptParseResult,
    call: &CallExpression<'_>,
    source: &str,
) -> CompactString {
    resolved_call_name(result, call).unwrap_or_else(|| expression_label(source, call.callee.span()))
}

pub(in crate::script_parser::extract) fn expression_label(
    source: &str,
    span: Span,
) -> CompactString {
    source
        .get(span.start as usize..span.end as usize)
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(CompactString::new)
        .unwrap_or_else(|| CompactString::new("<expression>"))
}
