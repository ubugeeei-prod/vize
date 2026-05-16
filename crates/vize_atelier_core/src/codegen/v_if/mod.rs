//! v-if generation functions.
//!
//! Generates code for v-if/v-else-if/v-else conditional rendering,
//! including branch dispatch, component/element/fragment generation,
//! and props object construction with key management.

mod branch;
mod generate;

use crate::ast::{IfBranchNode, IfNode, PropNode, RuntimeHelper};

use super::{context::CodegenContext, expression::generate_expression, helpers::escape_js_string};

use branch::generate_if_branch;
use vize_carton::ToCompactString;

/// Generate if node.
pub fn generate_if(ctx: &mut CodegenContext, if_node: &IfNode<'_>) {
    ctx.use_helper(RuntimeHelper::OpenBlock);

    // Vue always imports createCommentVNode for v-if nodes
    ctx.use_helper(RuntimeHelper::CreateComment);

    for (i, branch) in if_node.branches.iter().enumerate() {
        if let Some(condition) = &branch.condition {
            if i == 0 {
                // First branch: output condition with parentheses
                ctx.push("(");
                generate_expression(ctx, condition);
                ctx.push(")");
                ctx.indent();
                ctx.newline();
                ctx.push("? ");
            } else {
                // Subsequent branches (else-if)
                ctx.newline();
                ctx.push(": (");
                generate_expression(ctx, condition);
                ctx.push(")");
                ctx.indent();
                ctx.newline();
                ctx.push("? ");
            }
        } else {
            // Else branch (no condition)
            ctx.newline();
            ctx.push(": ");
        }

        // Generate branch content based on children
        generate_if_branch(ctx, branch, i);

        if branch.condition.is_some() && i > 0 {
            ctx.deindent();
        }
    }

    // Else branch (comment node) - only if all branches have conditions
    if if_node.branches.iter().all(|b| b.condition.is_some()) {
        ctx.newline();
        ctx.push(": ");
        ctx.push(ctx.helper(RuntimeHelper::CreateComment));
        ctx.push("(\"v-if\", true)");
    }

    ctx.deindent();
}

/// Generate key for if branch.
pub fn generate_if_branch_key(
    ctx: &mut CodegenContext,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    // Check if branch has a user-provided key
    if let Some(ref user_key) = branch.user_key {
        match user_key {
            PropNode::Attribute(attr) => {
                // Static key attribute
                if let Some(ref value) = attr.value {
                    ctx.push("\"");
                    ctx.push(&escape_js_string(value.content.as_str()));
                    ctx.push("\"");
                } else {
                    ctx.push(&branch_index.to_compact_string());
                }
            }
            PropNode::Directive(dir) => {
                // Dynamic :key binding
                if let Some(ref exp) = dir.exp {
                    generate_expression(ctx, exp);
                } else {
                    ctx.push(&branch_index.to_compact_string());
                }
            }
        }
    } else {
        ctx.push(&branch_index.to_compact_string());
    }
}

// Note: v-if directive behavior is tested via SFC snapshot tests
// in tests/fixtures/sfc/patches.toml. Unit tests for AST-based functions
// require bumpalo allocation which adds complexity without significant benefit.
