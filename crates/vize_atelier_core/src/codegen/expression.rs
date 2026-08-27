//! Expression generation functions.
//!
//! Generates code for simple expressions, compound expressions, and event
//! handlers, including TypeScript stripping, identifier prefixing, and
//! comment conversion.

mod comment_rewrite;
mod generate;
pub(crate) mod helpers;
pub(crate) mod prefix_context;
mod prefix_visitor;
pub(crate) mod scope_prefix;

use crate::{
    CompoundExpressionChild, CompoundExpressionNode, ExpressionNode, SimpleExpressionNode,
};

use super::{context::CodegenContext, helpers::escape_js_string};

use comment_rewrite::convert_line_comments_to_block;
use scope_prefix::{contains_slot_param_scope_prefix, strip_scope_prefixes_for_slot_params};
use vize_s0::String;
use vize_s0::ToCompactString;

#[allow(unused_imports)]
pub use generate::{
    generate_event_handler, generate_simple_expression_with_prefix, is_inline_handler,
    is_simple_member_expression,
};

/// Generate expression node (simple or compound).
pub fn generate_expression(ctx: &mut CodegenContext, expr: &ExpressionNode<'_>) {
    match expr {
        ExpressionNode::Simple(exp) => {
            generate_simple_expression(ctx, exp);
        }
        ExpressionNode::Compound(comp) => {
            generate_compound_expression(ctx, comp);
        }
    }
}

/// Generate a compound expression used directly in child position.
pub fn generate_compound_expression(
    ctx: &mut CodegenContext,
    compound: &CompoundExpressionNode<'_>,
) {
    for child in compound.children.iter() {
        match child {
            CompoundExpressionChild::Simple(exp) => {
                generate_simple_expression(ctx, exp);
            }
            CompoundExpressionChild::String(s) => {
                ctx.push(s);
            }
            CompoundExpressionChild::Symbol(helper) => {
                ctx.push(ctx.helper(*helper));
            }
            CompoundExpressionChild::Compound(_)
            | CompoundExpressionChild::Interpolation(_)
            | CompoundExpressionChild::Text(_) => {}
        }
    }
}

/// Generate simple expression with static string escaping, TypeScript stripping,
/// comment conversion, and slot parameter handling.
pub fn generate_simple_expression(ctx: &mut CodegenContext, exp: &SimpleExpressionNode<'_>) {
    if exp.is_static {
        ctx.push("\"");
        ctx.push(&escape_js_string(exp.content));
        ctx.push("\"");
    } else {
        // Strip TypeScript if needed
        let mut content: String = if ctx.options.is_ts && exp.content.contains(" as ") {
            crate::steps::strip_typescript_from_expression(exp.content)
        } else {
            exp.content.to_compact_string()
        };

        // Convert // line comments to /* */ block comments.
        // Template parsers may normalize newlines in attribute values to spaces,
        // which causes // comments to eat subsequent code on the same line.
        if content.contains("//") {
            content = convert_line_comments_to_block(&content);
        }

        // Record a source-map anchor from this generated expression back to its
        // template position before any of its bytes are written. Dynamic
        // expressions are the highest-value mapping target (a debugger steps
        // from generated `_ctx.foo` back to template `foo`), and this is the
        // single chokepoint every dynamic expression flows through. No-op unless
        // the `source_map` flag is on.
        ctx.record_mapping(exp.loc.span.start);

        // Replace generated scope prefixes when X is a known slot/v-for parameter.
        // This handles destructured variables that the transform phase
        // incorrectly prefixed because it didn't know the scope.
        if ctx.has_slot_params() && contains_slot_param_scope_prefix(&content) {
            ctx.push(&strip_scope_prefixes_for_slot_params(ctx, &content));
        } else {
            ctx.push(&content);
        }
    }
}
