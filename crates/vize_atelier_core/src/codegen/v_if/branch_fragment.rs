//! Fragment-shaped v-if branch emission.
//!
//! Emits the template-fragment and multi-child fragment branch shapes and the
//! shared branch-children serialization. Split out of `branch` to keep that
//! file focused on branch dispatch and the component/element shapes.

use crate::{IfBranchNode, RuntimeHelper, TemplateChildNode};

use super::{
    super::{
        children::generate_children_force_array, context::CodegenContext,
        helpers::escape_js_string, interpolation::push_interpolation_value,
    },
    generate_if_branch_key,
};

/// Generate template fragment for if branch (multiple children from template).
pub(super) fn generate_if_branch_template_fragment(
    ctx: &mut CodegenContext,
    children: &[TemplateChildNode<'_>],
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    ctx.use_helper(RuntimeHelper::CreateElementBlock);
    ctx.use_helper(RuntimeHelper::Fragment);
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
    ctx.push("(), ");
    ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::Fragment));
    ctx.push(", { key: ");
    generate_if_branch_key(ctx, branch, branch_index);
    ctx.push(" }, ");
    generate_children_force_array(ctx, children);
    ctx.push(", 64 /* STABLE_FRAGMENT */))");
}

/// Generate fragment wrapper for if branch with multiple children.
pub(super) fn generate_if_branch_fragment(
    ctx: &mut CodegenContext,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    ctx.use_helper(RuntimeHelper::CreateElementBlock);
    ctx.use_helper(RuntimeHelper::Fragment);
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
    ctx.push("(), ");
    ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::Fragment));
    ctx.push(", { key: ");
    generate_if_branch_key(ctx, branch, branch_index);
    ctx.push(" }, ");
    generate_children_force_array(ctx, &branch.children);
    ctx.push(", 64 /* STABLE_FRAGMENT */))");
}

/// Generate children for if branch element.
pub(super) fn generate_if_branch_children(
    ctx: &mut CodegenContext,
    children: &[TemplateChildNode<'_>],
) {
    if children.is_empty() {
        return;
    }

    // Check if all children are simple (text or interpolation)
    let has_only_text_or_interpolation = children.iter().all(|c| {
        matches!(
            c,
            TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
        )
    });

    if has_only_text_or_interpolation {
        // Use string concatenation for text/interpolation mix
        for (i, child) in children.iter().enumerate() {
            if i > 0 {
                ctx.push(" + ");
            }
            match child {
                TemplateChildNode::Interpolation(interp) => {
                    push_interpolation_value(ctx, interp);
                }
                TemplateChildNode::Text(text) => {
                    ctx.push("\"");
                    ctx.push(&escape_js_string(text.content.as_str()));
                    ctx.push("\"");
                }
                _ => {}
            }
        }
    } else {
        // Mixed children in block-optimized branches must emit text as createTextVNode,
        // otherwise Vue skips child normalization and raw strings become invalid VNodes.
        generate_children_force_array(ctx, children);
    }
}
