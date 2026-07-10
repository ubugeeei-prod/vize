//! Interpolation value codegen, shared by the Relief child paths and the
//! Relief-projection-driven node dispatcher.

use crate::{ExpressionNode, InterpolationNode, RuntimeHelper};

use super::context::CodegenContext;
use super::expression::generate_expression;

/// Emit an interpolation as a value expression.
///
/// A plain `{{ expr }}` is escaped through `_toDisplayString(expr)`. A Vue 1.x
/// raw-HTML interpolation (`{{{ expr }}}`, the pre-Vue-2 `v-html` equivalent;
/// only producible behind the `legacy` feature) renders unescaped.
pub fn push_interpolation_value(ctx: &mut CodegenContext, interp: &InterpolationNode<'_>) {
    #[cfg(feature = "legacy")]
    let raw = interp.raw;
    #[cfg(not(feature = "legacy"))]
    let raw = false;
    emit_interpolation_value(ctx, &interp.content, raw);
}

/// Emit an interpolation from its expression and raw flag (the Relief-projection-facing
/// core of [`push_interpolation_value`]).
pub(crate) fn emit_interpolation_value(
    ctx: &mut CodegenContext,
    content: &ExpressionNode<'_>,
    raw: bool,
) {
    #[cfg(feature = "legacy")]
    if raw {
        generate_expression(ctx, content);
        return;
    }
    #[cfg(not(feature = "legacy"))]
    let _ = raw;
    let helper = ctx.helper(RuntimeHelper::ToDisplayString);
    ctx.use_helper(RuntimeHelper::ToDisplayString);
    ctx.push(helper);
    ctx.push("(");
    generate_expression(ctx, content);
    ctx.push(")");
}
