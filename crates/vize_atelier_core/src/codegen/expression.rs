//! Expression generation functions.
//!
//! Generates code for simple expressions, compound expressions, and event
//! handlers, including TypeScript stripping, identifier prefixing, and
//! comment conversion.

mod generate;
pub(crate) mod helpers;

use crate::ast::{CompoundExpressionChild, ExpressionNode, SimpleExpressionNode};

use super::{context::CodegenContext, helpers::escape_js_string};

use helpers::{convert_line_comments_to_block, strip_ctx_for_slot_params};
use vize_carton::String;
use vize_carton::ToCompactString;

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
            for child in comp.children.iter() {
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
                    _ => {}
                }
            }
        }
    }
}

/// Generate simple expression with static string escaping, TypeScript stripping,
/// comment conversion, and slot parameter handling.
pub fn generate_simple_expression(ctx: &mut CodegenContext, exp: &SimpleExpressionNode<'_>) {
    if exp.is_static {
        ctx.push("\"");
        ctx.push(&escape_js_string(exp.content.as_str()));
        ctx.push("\"");
    } else {
        // Strip TypeScript if needed
        let mut content: String = if ctx.options.is_ts && exp.content.contains(" as ") {
            crate::transforms::strip_typescript_from_expression(&exp.content)
        } else {
            exp.content.to_compact_string()
        };

        // Convert // line comments to /* */ block comments.
        // Template parsers may normalize newlines in attribute values to spaces,
        // which causes // comments to eat subsequent code on the same line.
        if content.contains("//") {
            content = convert_line_comments_to_block(&content);
        }

        // Replace _ctx.X with X when X is a known slot/v-for parameter.
        // This handles destructured variables that the transform phase
        // incorrectly prefixed with _ctx. because it didn't know the scope.
        if ctx.has_slot_params() && content.contains("_ctx.") {
            ctx.push(&strip_ctx_for_slot_params(ctx, &content));
        } else {
            ctx.push(&content);
        }
    }
}
