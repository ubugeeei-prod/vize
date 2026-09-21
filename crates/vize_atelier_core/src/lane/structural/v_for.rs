//! `v-for`, and the single-binding scope patterned templates lower to.

use vize_s0::{Box, Vec};

use crate::errors::ErrorCode;
use crate::{ExpressionNode, ForNode, ForParseResult, RuntimeHelper, TemplateChildNode};

use super::super::context::clone_expression;
use super::super::{ExitFns, TransformContext};
use super::SimpleExpressionContent;

/// `raw_name` of the synthetic `v-for` a `v-match` / `v-when` lowers to. The
/// parser never produces it, so it cannot collide with an authored directive.
pub(in crate::lane) const MATCH_SCOPE_RAW_NAME: &str = "v-match-scope";

/// Transform v-for directive
pub fn transform_v_for<'a>(
    ctx: &mut TransformContext<'a>,
    exp: Option<&SimpleExpressionContent<'a>>,
) -> Option<ExitFns<'a>> {
    transform_for_scope(ctx, exp, false)
}

/// [`transform_v_for`], or the lexical scope a patterned-template `v-match`
/// lowers to when `match_scope` is set: same aliases and traversal, but the
/// node binds its single-item source once instead of rendering a list.
pub(in crate::lane) fn transform_for_scope<'a>(
    ctx: &mut TransformContext<'a>,
    exp: Option<&SimpleExpressionContent<'a>>,
    match_scope: bool,
) -> Option<ExitFns<'a>> {
    let allocator = ctx.allocator;

    let Some(exp) = exp else {
        ctx.on_error(ErrorCode::VForNoExpression, None);
        return None;
    };

    let Some(parse_result) = crate::steps::parse_for_expression_with_options(
        allocator,
        exp.content,
        &exp.loc,
        ctx.template_syntax_quirks(),
    ) else {
        ctx.on_error(ErrorCode::VForMalformedExpression, Some(exp.loc.clone()));
        return None;
    };

    // Take the current element from parent
    let taken = ctx.take_current_node();
    let taken_node = taken?;

    let element_loc = match &taken_node {
        TemplateChildNode::Element(el) => el.loc.clone(),
        _ => return None,
    };

    let mut source = parse_result.source;
    let value_alias = parse_result.value;
    let key_alias = parse_result.key;
    let index_alias = parse_result.index;

    // Process source expression with binding-aware identifier prefixing
    // This ensures imports and refs are correctly handled (e.g., _unref(PRESETS) instead of _ctx.PRESETS)
    if ctx.options.prefix_identifiers || ctx.options.is_ts {
        use crate::steps::process_expression;
        // Process the source expression through the binding-aware transform
        let processed = process_expression(ctx, &source, false);
        source = processed;
    }

    // Create ForNode children with taken element
    let mut for_children = Vec::new_in(&allocator);
    for_children.push(taken_node);

    // Create parse result (clone expressions for parse_result)
    let src = ctx.source;
    let clone = |e: &ExpressionNode<'a>| clone_expression(allocator, e, src);
    let mut parse_result = ForParseResult::new(
        clone_expression(allocator, &source, src),
        value_alias.as_ref().map(clone),
        key_alias.as_ref().map(clone),
        index_alias.as_ref().map(clone),
    );
    parse_result.match_scope = match_scope;

    let for_node = ForNode {
        source,
        value_alias,
        key_alias,
        object_index_alias: index_alias,
        parse_result,
        children: for_children,
        loc: element_loc,
    };

    // Replace placeholder with ForNode
    ctx.replace_node(TemplateChildNode::For(Box::new_in(for_node, &allocator)));

    // Add helpers
    if !match_scope {
        ctx.helper(RuntimeHelper::RenderList);
        ctx.helper(RuntimeHelper::Fragment);
    }
    ctx.helper(RuntimeHelper::OpenBlock);
    ctx.helper(RuntimeHelper::CreateBlock);
    ctx.helper(RuntimeHelper::CreateElementBlock);

    None
}
