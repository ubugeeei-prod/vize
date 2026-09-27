//! Event-handler prefixing: the transform's `process_inline_handler`
//! followed by the codegen's `generate_event_handler` tail, ported so the
//! shipped bytes survive intact — including the second `$event => (…)`
//! wrap the codegen adds when the processed text no longer parses as a
//! function (a trailing line comment swallowing the arrow body's `)`).

use vize_l0::String;

use super::globals::is_simple_identifier;
use super::rewrite::{Retained, RewriteResult, rewrite_expression};
use super::scope::PrefixScope;
use super::shape::{
    function_and_reference, is_event_handler_reference_node, is_function_expression,
    is_function_expression_node,
};
use super::strip::strip_scope_prefixes_for_slot_params;
use super::strip_typescript_from_expression;

/// `process_inline_handler` over the node's content (the padded attribute
/// value the shipped transform held).
pub(super) fn process_inline_handler(
    content: &str,
    retained: Option<Retained<'_, '_>>,
    scope: &mut PrefixScope<'_>,
) -> RewriteResult {
    // The function/reference shape is read off the TS-stripped text; the
    // retained AST still applies only while that text is the node's own
    // bytes. The *rewrite* below re-strips from the original.
    let stripped = scope
        .is_ts()
        .then(|| strip_typescript_from_expression(content));
    let shape_source = stripped.as_ref().map_or(content, String::as_str);
    let is_function = if shape_source == content {
        is_function_expression_node(content, retained)
    } else {
        is_function_expression(shape_source)
    };
    if is_function {
        return process(content, retained, scope);
    }
    // Only the function check reads the stripped text; the reference check
    // is the shipped lane's, over the node's own bytes.
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
                used_unref: false,
                parse_error: false,
            };
        }
        return process(content, retained, scope);
    }
    let mark = scope.mark();
    scope.push_event();
    let rewritten = process(content, retained, scope);
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
    RewriteResult {
        code,
        // The wrap is the shipped lane's `$event => (…)`; the helper the
        // rewritten *body* needed is still the transform's registration.
        used_unref: rewritten.used_unref,
        parse_error: rewritten.parse_error,
    }
}

/// The handler body itself: rewritten when identifiers are prefixed,
/// type-erased alone when only `is_ts` is on.
fn process(
    content: &str,
    retained: Option<Retained<'_, '_>>,
    scope: &PrefixScope<'_>,
) -> RewriteResult {
    if scope.prefixes_identifiers() {
        return rewrite_expression(content, retained, scope, false);
    }
    RewriteResult {
        code: if scope.is_ts() {
            strip_typescript_from_expression(content)
        } else {
            String::from(content)
        },
        used_unref: false,
        parse_error: false,
    }
}

/// `generate_event_handler` over an `is_ref_transformed` node: strip the
/// slot-param prefixes, then re-derive the shape from the processed text
/// (the retained AST no longer applies to rewritten bytes).
pub(super) fn finish_event_handler(
    processed: String,
    scope: &PrefixScope<'_>,
    for_caching: bool,
) -> String {
    let processed = if scope.has_slot_params() {
        strip_scope_prefixes_for_slot_params(scope, processed.as_str())
    } else {
        processed
    };
    let (is_function, is_reference) = handler_shape(processed.as_str());
    if is_function {
        return processed;
    }
    if is_reference {
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

/// Prefixes the rewrite writes before a simple identifier. None of them is
/// a reserved word, so `<prefix><name>(.<name>)*` is a static member chain.
const MEMBER_ROOTS: [&str; 6] = [
    "_ctx.",
    "$setup.",
    "$props.",
    "__props.",
    "$data.",
    "$options.",
];

/// `(is_function_expression, is_simple_identifier ||
/// is_event_handler_reference_expression)` over processed handler text,
/// from at most one parse.
///
/// A bare identifier is never a function expression, and an ASCII name
/// chain under a rewrite prefix is always a static member expression, so
/// neither needs the parser. Anything else reads both shapes off the one
/// whole-expression parse the two string checks used to repeat.
fn handler_shape(processed: &str) -> (bool, bool) {
    if is_simple_identifier(processed) || is_prefixed_member_chain(processed) {
        return (false, true);
    }
    function_and_reference(processed)
}

/// Whether `text` is `<rewrite prefix><name>(.<name>)*` over ASCII names.
pub(super) fn is_prefixed_member_chain(text: &str) -> bool {
    let Some(rest) = MEMBER_ROOTS.iter().find_map(|root| text.strip_prefix(root)) else {
        return false;
    };
    rest.split('.').all(|name| {
        let mut bytes = name.bytes();
        bytes
            .next()
            .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_' || b == b'$')
            && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'$')
    })
}

#[cfg(test)]
mod tests {
    use super::super::globals::is_simple_identifier;
    use super::super::shape::{is_event_handler_reference_expression, is_function_expression};
    use super::{handler_shape, is_prefixed_member_chain};

    #[test]
    fn the_shortcut_shapes_match_the_parsed_shapes() {
        for text in [
            "h",
            "true",
            "async",
            "_ctx.h",
            "$setup.h.value",
            "$props.a.b",
            "__props.class",
            "_ctx.h()",
            "_ctx.h(1).x",
            "_ctx.a + 1",
            "() => _ctx.h()",
            "function () {}",
            "_ctx.a?.b",
            "_ctx.",
            "_ctx..a",
            "_ctx.1a",
            "_ctx.a // c",
            "_ctx.count += 1",
            "_ctx.a; _ctx.b",
        ] {
            let parsed = (
                is_function_expression(text),
                is_simple_identifier(text) || is_event_handler_reference_expression(text),
            );
            assert_eq!(handler_shape(text), parsed, "{text}");
        }
    }

    #[test]
    fn only_ascii_name_chains_under_a_rewrite_prefix_skip_the_parser() {
        assert!(is_prefixed_member_chain("_ctx.onClick"));
        assert!(is_prefixed_member_chain("$setup.a.value"));
        assert!(!is_prefixed_member_chain("ctx.onClick"));
        assert!(!is_prefixed_member_chain("_ctx."));
        assert!(!is_prefixed_member_chain("_ctx.a-b"));
        assert!(!is_prefixed_member_chain("_ctx.é"));
    }
}
