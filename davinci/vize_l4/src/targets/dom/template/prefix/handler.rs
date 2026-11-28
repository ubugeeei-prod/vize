//! Event-handler prefixing: the transform's `process_inline_handler`
//! followed by the codegen's `generate_event_handler` tail, ported so the
//! shipped bytes survive intact — including the second `$event => (…)`
//! wrap the codegen adds when the processed text no longer parses as a
//! function (a trailing line comment swallowing the arrow body's `)`).

use vize_l0::String;

use super::Erased;
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
    js: &Erased<'_, '_, '_>,
    scope: &mut PrefixScope<'_>,
) -> RewriteResult {
    process_classified_handler(js, scope).0
}

/// [`process_inline_handler`] plus whether the authored handler is a method
/// reference: rewriting `fn` to `_unref(fn)` must not turn it into a statement.
pub(super) fn process_classified_handler(
    js: &Erased<'_, '_, '_>,
    scope: &mut PrefixScope<'_>,
) -> (RewriteResult, bool) {
    let content = js.authored;
    let retained = js.retained;
    // Both shapes are read off the erased text. The retained AST still
    // applies only while that text is the node's own bytes: `onPick!`
    // erases to the identifier `onPick`, which Vue passes through as a
    // reference rather than `$event => (onPick)`.
    let shape = js.js();
    let is_function = if js.changed() {
        is_function_expression(shape)
    } else {
        is_function_expression_node(content, retained)
    };
    if is_function {
        return (process(js, scope), false);
    }
    let reference = if js.changed() {
        is_reference(shape, None)
    } else {
        is_reference(content, retained)
    };
    if reference {
        // Inline render functions read setup bindings directly, so an
        // identifier still needs its `_unref` / `.value` access.
        if is_simple_identifier(shape) && !scope.inline() {
            let code = match scope.identifier_prefix(shape) {
                Some(prefix) => {
                    let mut code = String::with_capacity(prefix.len() + shape.len());
                    code.push_str(prefix);
                    code.push_str(shape);
                    code
                }
                None => String::from(shape),
            };
            let result = RewriteResult {
                code,
                used_unref: false,
                used_is_ref: false,
                has_identifier_references: true,
                parse_error: false,
            };
            return (result, true);
        }
        return (process(js, scope), true);
    }
    let mark = scope.mark();
    scope.push_event();
    let rewritten = process(js, scope);
    scope.pop(mark);
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
    let result = RewriteResult {
        code,
        // The wrap is the shipped lane's `$event => (…)`; the helper the
        // rewritten *body* needed is still the transform's registration.
        used_unref: rewritten.used_unref,
        used_is_ref: rewritten.used_is_ref,
        has_identifier_references: rewritten.has_identifier_references,
        parse_error: rewritten.parse_error,
    };
    (result, false)
}

/// A method reference (Vue's `isMemberExpression`) over `content`.
///
/// Callers pass the erased spelling once TypeScript changed the bytes, and
/// `None` for the retained AST: that AST still describes the authored text.
/// `TSNonNullExpression` / `TSAsExpression` therefore unwrap to the same
/// identifier or member reference Vue forwards, instead of a parenthesized
/// value the click never calls.
pub(super) fn is_reference(content: &str, retained: Option<Retained<'_, '_>>) -> bool {
    is_simple_identifier(content) || is_event_handler_reference_node(content, retained)
}

/// The handler body itself: rewritten when identifiers are prefixed,
/// type-erased alone when only `is_ts` is on.
fn process(js: &Erased<'_, '_, '_>, scope: &PrefixScope<'_>) -> RewriteResult {
    if scope.prefixes_identifiers() {
        return rewrite_expression(js, scope, false);
    }
    RewriteResult {
        code: String::from(js.js()),
        used_unref: false,
        used_is_ref: false,
        has_identifier_references: false,
        parse_error: false,
    }
}

/// `generate_event_handler` over an `is_ref_transformed` node: strip the
/// slot-param prefixes, then re-derive the shape from the processed text
/// (the retained AST no longer applies to rewritten bytes). A method
/// reference keeps the transform's classification.
pub(super) fn finish_event_handler(
    processed: String,
    scope: &PrefixScope<'_>,
    for_caching: bool,
    is_reference: bool,
) -> String {
    let processed = if scope.has_slot_params() {
        strip_scope_prefixes_for_slot_params(scope, processed.as_str())
    } else {
        processed
    };
    if !is_reference && is_function_expression(processed.as_str()) {
        return processed;
    }
    if is_reference
        || is_simple_identifier(processed.as_str())
        || is_event_handler_reference_expression(processed.as_str())
    {
        // A cached reference is guarded and forwarded, so the slot holds
        // a stable closure rather than whatever the name held on the
        // first render.
        if !for_caching {
            return processed;
        }
        let mut code = String::with_capacity(processed.len() * 2 + 24);
        code.push_str("(...args) => (");
        code.push_str(processed.as_str());
        code.push_str(" && ");
        code.push_str(processed.as_str());
        code.push_str("(...args))");
        return code;
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
