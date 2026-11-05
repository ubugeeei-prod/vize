//! Scoped expression rewriting for retained `v-for` parameter goals.

use crate::lane::TransformContext;
use crate::{ExpressionNode, RuntimeHelper, SourceLocation};

use super::bindings::{alias_needs_expression_rewrite, alias_node};

/// Prefix default initializers and computed keys inside one `v-for` alias,
/// then rebuild its parameter carrier so traversal and codegen keep reading
/// the same cooked binding names from the rewritten bytes.
pub(crate) fn process_for_alias<'a>(
    ctx: &mut TransformContext<'a>,
    alias: ExpressionNode<'a>,
    authored_expression_loc: &SourceLocation,
) -> ExpressionNode<'a> {
    let mut original = match alias {
        ExpressionNode::Simple(original) => original,
        compound @ ExpressionNode::Compound(_) => return compound,
    };
    if !ctx.options.prefix_identifiers || !alias_needs_expression_rewrite(&original) {
        return ExpressionNode::Simple(original);
    }

    let result = crate::steps::expression::rewrite_for_alias(original.content, ctx);
    if result.used_unref {
        ctx.helper(RuntimeHelper::Unref);
    }
    if result.used_is_ref {
        ctx.helper(RuntimeHelper::IsRef);
    }
    if let Some(detail) = &result.parse_error {
        let diagnostic_loc = if original.loc.span.is_empty() {
            authored_expression_loc
        } else {
            &original.loc
        };
        crate::steps::expression::report_invalid_expression(ctx, detail, diagnostic_loc);
    }
    original.is_ref_transformed = true;
    if result.code.as_str() == original.content {
        return ExpressionNode::Simple(original);
    }

    let content = ctx.allocator.alloc_str(result.code.as_str());
    let Some(ExpressionNode::Simple(mut rewritten)) = alias_node(ctx.allocator, content) else {
        return ExpressionNode::Simple(original);
    };
    rewritten.loc = original.loc.clone();
    rewritten.is_ref_transformed = true;
    ExpressionNode::Simple(rewritten)
}

/// An annotated alias (`item: Item`) enters the TS lane as its JavaScript
/// parameter, so every later reader sees the erased pattern.
pub(crate) fn erase_for_alias<'a>(
    allocator: &'a vize_l0::Allocator,
    alias: &mut Option<ExpressionNode<'a>>,
) {
    let Some(ExpressionNode::Simple(node)) = alias.as_ref() else {
        return;
    };
    let Some(js) = crate::steps::erase_typescript_params(node.content) else {
        return;
    };
    let loc = node.loc.clone();
    let content = allocator.alloc_str(js.as_str());
    if let Some(ExpressionNode::Simple(mut erased)) = alias_node(allocator, content) {
        erased.loc = loc;
        *alias = Some(ExpressionNode::Simple(erased));
    }
}
