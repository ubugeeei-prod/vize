//! Expression transform steps for identifier prefixing and TS cleanup.

mod collector;
mod collector_targets;
mod inline_handler;
pub(crate) mod nesting;
mod parse_checks;
pub(crate) mod prefix;
mod reparse;
mod retained_rewrite;
mod rewrite;
mod shape_checks;
mod splice;
mod typescript;

use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_s0::{Allocator, Box, String};

use crate::{ConstantType, ExpressionNode, SimpleExpressionNode, lane::TransformContext};

pub use inline_handler::process_inline_handler;
pub use nesting::{
    MAX_EXPRESSION_NESTING_DEPTH, expression_exceeds_max_depth, expression_has_balanced_delimiters,
    expression_is_safe_to_parse, expression_nesting_depth,
};
pub use prefix::{is_simple_identifier, prefix_identifiers_in_expression};
use rewrite::rewrite_expression;
pub use shape_checks::{is_event_handler_reference_node, is_function_expression_node};
use shape_checks::{is_function_shape, is_handler_reference_shape};
pub use typescript::strip_typescript_from_expression;

/// Returns true if an expression is a callable reference that should be passed
/// through directly as an event handler, not wrapped as `$event => (...)`.
pub fn is_event_handler_reference_expression(content: &str) -> bool {
    if !expression_is_safe_to_parse(content) {
        return false;
    }
    let allocator = crate::expr_parse_probe::parse_arena();
    let parser = Parser::new(&allocator, content, SourceType::default().with_module(true));
    let Ok(expr) = parser.parse_expression() else {
        return false;
    };
    is_handler_reference_shape(&expr)
}

/// Returns true if the whole expression is a function / arrow function expression.
pub fn is_function_expression(content: &str) -> bool {
    if !expression_is_safe_to_parse(content) {
        return false;
    }
    let allocator = crate::expr_parse_probe::parse_arena();
    let parser = Parser::new(&allocator, content, SourceType::default().with_module(true));
    let Ok(expr) = parser.parse_expression() else {
        return false;
    };
    is_function_shape(&expr)
}

/// Rewrite Vue 2 pipe filters in `exp` in place, mirroring
/// `@vue/compiler-core`'s `transformFilter` / `wrapFilter`.
///
/// Returns `true` and rewrites `exp.content` to the unprefixed
/// `_filter_<name>(base,args)` form (registering each filter and the
/// `_resolveFilter` helper) when the dialect supports filters and the content
/// actually contains a top-level filter pipe. Returns `false` — leaving `exp`
/// untouched — otherwise, so non-filter expressions (and every Vue 3 source)
/// are byte-identical to before.
#[cfg(feature = "legacy")]
fn rewrite_filters_in_place<'a>(
    ctx: &mut TransformContext<'a>,
    exp: &mut Box<'a, SimpleExpressionNode<'a>>,
) -> bool {
    use crate::steps::legacy_filters::{filter_name, parse_filters, wrap_filter};

    if !ctx.supports_filters() {
        return false;
    }

    let Some(parsed) = parse_filters(exp.content) else {
        return false;
    };

    // Validate every filter name up front; if any is malformed, bail out and
    // leave the original expression untouched (matches Vue resolving named
    // filters only).
    let mut names: std::vec::Vec<(String, String)> = std::vec::Vec::new();
    for filter in &parsed.filters {
        let Some(name) = filter_name(filter) else {
            return false;
        };
        let id = crate::codegen::to_valid_asset_identifier("filter", name);
        names.push((String::new(name), id));
    }

    // Build the wrapped expression from the inside out: each filter wraps the
    // running expression, outermost filter last.
    let mut wrapped = parsed.base;
    for (filter, (name, id)) in parsed.filters.iter().zip(names.iter()) {
        let Some(next) = wrap_filter(wrapped.as_str(), filter, id.as_str()) else {
            return false;
        };
        wrapped = next;
        ctx.add_filter(name.clone());
    }

    ctx.helper(crate::RuntimeHelper::ResolveFilter);
    exp.content = ctx.allocator.alloc_str(&wrapped);
    // The content was rewritten from source text, so any cached parse is stale.
    exp.js_ast = None;
    exp.is_ref_transformed = false;
    true
}

/// Process expression with identifier prefixing and TypeScript stripping
pub fn process_expression<'a>(
    ctx: &mut TransformContext<'a>,
    exp: &ExpressionNode<'a>,
    as_params: bool,
) -> ExpressionNode<'a> {
    let allocator = ctx.allocator;

    // `mut` is only consumed by the legacy filter rewrite below; without the
    // `legacy` feature that block is cfg'd out and the binding is never mutated.
    #[cfg_attr(not(feature = "legacy"), allow(unused_mut))]
    let mut normalized = normalize_expression(exp, allocator, ctx.source);

    // Vue 2 pipe filters (`{{ msg | capitalize }}`): split the top-level `|`
    // chain into an unprefixed `_filter_<name>(base,args)` rewrite, register
    // the filters, and pull in `_resolveFilter`. Identifier prefixing then runs
    // over the rewritten text below (skipping the `_filter_` ids). Legacy-only
    // and dialect-gated: never entered for the default Vue 3 dialect, where a
    // `|`-containing expression stays byte-identical to today's bitwise-or.
    #[cfg(feature = "legacy")]
    if !normalized.is_static && rewrite_filters_in_place(ctx, &mut normalized) {
        // The content now contains `_filter_*(...)` calls. Fall through to the
        // identifier-prefixing pass (which handles `_ctx.`/`.value` on the base
        // and args) when prefixing/TS is on; otherwise return the rewrite as-is.
        if !ctx.options.prefix_identifiers && !ctx.options.is_ts {
            return ExpressionNode::Simple(normalized);
        }
    }

    // If not prefixing identifiers and not TypeScript, just clone
    if !ctx.options.prefix_identifiers && !ctx.options.is_ts {
        return ExpressionNode::Simple(normalized);
    }

    if normalized.is_static {
        return ExpressionNode::Simple(normalized);
    }

    // Skip if already processed for ref transformation
    if normalized.is_ref_transformed {
        return ExpressionNode::Simple(normalized);
    }

    let content = &normalized.content;

    // Empty content
    if content.is_empty() {
        return ExpressionNode::Simple(normalized);
    }

    // Strip TypeScript if needed, then optionally prefix identifiers
    let processed = if ctx.options.prefix_identifiers {
        // rewrite_expression handles both TS stripping and prefixing; the
        // retained AST rides along when it still describes these bytes (P1-7).
        let retained = crate::retained::retained_whole_expression(&normalized);
        let result = rewrite_expression(content, ctx, as_params, retained);
        if result.used_unref {
            ctx.helper(crate::RuntimeHelper::Unref);
        }
        // The expression failed to parse entirely and was passed through
        // raw — report it instead of silently emitting broken render code.
        // Matches `@vue/compiler-core`'s `X_INVALID_EXPRESSION`.
        if let Some(detail) = &result.parse_error {
            rewrite::report_invalid_expression(ctx, detail, &normalized.loc);
        }
        result.code
    } else if ctx.options.is_ts {
        // Only strip TypeScript, no prefixing
        strip_typescript_from_expression(content)
    } else {
        String::new(content)
    };

    ExpressionNode::Simple(Box::new_in(
        SimpleExpressionNode {
            content: allocator.alloc_str(&processed),
            is_static: false,
            const_type: normalized.const_type,
            loc: normalized.loc.clone(),
            js_ast: None,
            hoisted: None,
            identifiers: None,
            is_handler_key: normalized.is_handler_key,
            is_ref_transformed: true,
        },
        &allocator,
    ))
}

/// Clone an expression node.
///
/// Compound expressions are flattened to a [`SimpleExpressionNode`] whose
/// content is the original source text. This mirrors the strategy used by
/// [`normalize_expression`] and `lane::context::clone_expression`,
/// and avoids the previous behavior of producing a `Compound` node with
/// an empty `children` list (which silently dropped the expression).
pub(crate) fn clone_expression<'a>(
    exp: &ExpressionNode<'a>,
    allocator: &'a Allocator,
    source: &'a str,
) -> ExpressionNode<'a> {
    match exp {
        ExpressionNode::Simple(simple) => ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode {
                content: simple.content,
                is_static: simple.is_static,
                const_type: simple.const_type,
                loc: simple.loc.clone(),
                // Byte-identical clone: the retained parse still applies (P1-7).
                js_ast: simple.js_ast,
                hoisted: None,
                identifiers: None,
                is_handler_key: simple.is_handler_key,
                is_ref_transformed: simple.is_ref_transformed,
            },
            &allocator,
        )),
        ExpressionNode::Compound(compound) => ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode {
                content: compound.loc.span.slice(source),
                is_static: false,
                const_type: ConstantType::NotConstant,
                loc: compound.loc.clone(),
                js_ast: None,
                hoisted: None,
                identifiers: None,
                is_handler_key: compound.is_handler_key,
                is_ref_transformed: false,
            },
            &allocator,
        )),
    }
}

pub(crate) fn normalize_expression<'a>(
    exp: &ExpressionNode<'a>,
    allocator: &'a Allocator,
    source: &'a str,
) -> Box<'a, SimpleExpressionNode<'a>> {
    match exp {
        ExpressionNode::Simple(simple) => Box::new_in(
            SimpleExpressionNode {
                content: simple.content,
                is_static: simple.is_static,
                const_type: simple.const_type,
                loc: simple.loc.clone(),
                // Byte-identical clone: the retained parse still applies (P1-7).
                js_ast: simple.js_ast,
                hoisted: None,
                identifiers: None,
                is_handler_key: simple.is_handler_key,
                is_ref_transformed: simple.is_ref_transformed,
            },
            &allocator,
        ),
        ExpressionNode::Compound(compound) => Box::new_in(
            SimpleExpressionNode {
                content: compound.loc.span.slice(source),
                is_static: false,
                const_type: ConstantType::NotConstant,
                loc: compound.loc.clone(),
                js_ast: None,
                hoisted: None,
                identifiers: None,
                is_handler_key: compound.is_handler_key,
                is_ref_transformed: false,
            },
            &allocator,
        ),
    }
}

#[cfg(test)]
mod tests;

// Note: Multiline arrow function handling and ES6 shorthand expansion
// are tested via SFC snapshot tests in tests/fixtures/sfc/patches.pkl.
