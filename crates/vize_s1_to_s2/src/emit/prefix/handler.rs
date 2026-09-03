//! Event-handler prefixing: the transform's `process_inline_handler`
//! followed by the codegen's `generate_event_handler` tail, ported so the
//! shipped bytes survive intact — including the second `$event => (…)`
//! wrap the codegen adds when the processed text no longer parses as a
//! function (a trailing line comment swallowing the arrow body's `)`).

use vize_s0::String;

use super::globals::is_simple_identifier;
use super::rewrite::{Retained, RewriteResult, rewrite_expression};
use super::scope::PrefixScope;
use super::shape::{
    is_event_handler_reference_expression, is_event_handler_reference_node, is_function_expression,
    is_function_expression_node,
};
use super::strip::strip_scope_prefixes_for_slot_params;

/// `process_inline_handler` over the node's content (the padded attribute
/// value the shipped transform held).
pub(super) fn process_inline_handler(
    content: &str,
    retained: Option<Retained<'_, '_>>,
    scope: &PrefixScope<'_>,
) -> RewriteResult {
    if is_function_expression_node(content, retained) {
        return rewrite_expression(content, retained, scope, false);
    }
    if is_simple_identifier(content) || is_event_handler_reference_node(content, retained) {
        if is_simple_identifier(content) {
            let code = match scope.identifier_prefix(content) {
                Some(prefix) => {
                    let mut code = String::with_capacity(prefix.len() + content.len());
                    code.push_str(prefix);
                    code.push_str(content);
                    code
                }
                None => String::from(content),
            };
            return RewriteResult {
                code,
                parse_error: false,
            };
        }
        return rewrite_expression(content, retained, scope, false);
    }
    let rewritten = rewrite_expression(content, retained, scope, false);
    let mut code = String::with_capacity(rewritten.code.len() + 13);
    if rewritten.code.contains(';') {
        code.push_str("$event => {");
        code.push_str(rewritten.code.as_str());
        code.push('}');
    } else {
        code.push_str("$event => (");
        code.push_str(rewritten.code.as_str());
        code.push(')');
    }
    RewriteResult {
        code,
        parse_error: rewritten.parse_error,
    }
}

/// `generate_event_handler` over an `is_ref_transformed` node: strip the
/// slot-param prefixes, then re-derive the shape from the processed text
/// (the retained AST no longer applies to rewritten bytes).
pub(super) fn finish_event_handler(processed: String, scope: &PrefixScope<'_>) -> String {
    let processed = if scope.has_slot_params() {
        strip_scope_prefixes_for_slot_params(scope, processed.as_str())
    } else {
        processed
    };
    if is_function_expression(processed.as_str()) {
        return processed;
    }
    if is_simple_identifier(processed.as_str())
        || is_event_handler_reference_expression(processed.as_str())
    {
        return processed;
    }
    let mut code = String::with_capacity(processed.len() + 13);
    if processed.contains(';') {
        code.push_str("$event => {");
        code.push_str(processed.as_str());
        code.push('}');
    } else {
        code.push_str("$event => (");
        code.push_str(processed.as_str());
        code.push(')');
    }
    code
}
