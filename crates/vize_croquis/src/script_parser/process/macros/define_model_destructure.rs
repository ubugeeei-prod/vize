use oxc_ast::ast::{ArrayPattern, Expression};
use oxc_span::GetSpan;

use crate::{
    macros::MacroKind,
    reactivity::ReactiveKind,
    script_parser::{
        ScriptParseResult,
        extract::{extract_call_expression, process_call_expression},
        process::bindings::get_binding_pattern_name,
    },
};
use vize_carton::CompactString;

pub(super) fn process(
    result: &mut ScriptParseResult,
    init: Option<&Expression<'_>>,
    array: &ArrayPattern<'_>,
    source: &str,
) -> bool {
    let macro_kind = init
        .and_then(extract_call_expression)
        .and_then(|call| process_call_expression(result, call, source));
    if macro_kind != Some(MacroKind::DefineModel) {
        return false;
    }
    if let Some(first) = array.elements.first().and_then(|elem| elem.as_ref())
        && let Some(name) = get_binding_pattern_name(first)
    {
        result
            .macros
            .set_latest_model_local_name(CompactString::new(name.as_str()));
        result.reactivity.register(
            CompactString::new(name.as_str()),
            ReactiveKind::Ref,
            first.span().start,
        );
    }
    true
}
