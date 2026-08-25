//! Identifier prefixing for dynamic `v-bind` / `v-on` arguments.

use vize_relief::SimpleExpressionNode;

use super::super::{
    context::CodegenContext,
    expression::{
        generate_simple_expression, prefix_context::prefix_identifiers_with_context_node,
    },
};
use crate::steps::is_simple_identifier;

/// Emit a dynamic directive argument (`:[expr]`, `@[expr]`).
///
/// Simple identifiers keep the historical `_ctx.` prepend so existing
/// snapshots and slot-param special cases stay byte-identical. Compound
/// keys (`prefix+suffix`, `foo.bar`, `keyOf(item)`) walk identifiers the
/// same way template-literal keys already do — the previous heuristic
/// either prepended `_ctx.` to the whole string or emitted the raw text,
/// which crashes at runtime under SFC `prefix_identifiers`.
pub(super) fn emit_dynamic_directive_arg(ctx: &mut CodegenContext, exp: &SimpleExpressionNode<'_>) {
    let content = exp.content;
    if let Some(local) = content
        .strip_prefix("_ctx.")
        .filter(|local| ctx.is_slot_param(local))
    {
        ctx.push(local);
        return;
    }
    if ctx.is_slot_param(content) {
        ctx.push(content);
        return;
    }
    if is_simple_identifier(content) {
        ctx.push("_ctx.");
        ctx.push(content);
        return;
    }
    if content.starts_with('_') || content.starts_with('$') {
        generate_simple_expression(ctx, exp);
        return;
    }
    if content.starts_with('`') {
        ctx.push("(");
        ctx.push(&prefix_identifiers_with_context_node(exp, ctx));
        ctx.push(")");
        return;
    }
    if ctx.options.prefix_identifiers {
        ctx.push(&prefix_identifiers_with_context_node(exp, ctx));
    } else {
        generate_simple_expression(ctx, exp);
    }
}
