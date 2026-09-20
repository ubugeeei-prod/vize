//! `v-once` on conditional content.
//!
//! On the `v-if` element itself the whole chain is cached, so a later change of
//! the condition is ignored as well (`generate_if` owns that case). On content
//! *inside* a branch only that content is cached: the condition stays live and
//! the cached tree is reused whenever the branch renders again. The content of
//! a patterned-template arm is the second case, which keeps an arm's bindings
//! frozen while other arms still select normally.

use crate::{ElementNode, ElementType, ForNode, IfBranchNode, IfNode, TemplateChildNode};

use super::super::{
    context::CodegenContext,
    element::{generate_v_once_cached, has_v_once},
    v_for::match_scope::generate_match_scope,
};
use super::branch::generate_if_branch_nodes;

/// Whether `v-once` sits on the chain's own `v-if` element.
pub(super) fn chain_is_once(if_node: &IfNode<'_>) -> bool {
    matches!(
        if_node.branches.first().map(|branch| branch.children.as_slice()),
        Some([TemplateChildNode::Element(el)]) if has_v_once(el)
    )
}

/// A branch's content. `<template v-if>` around one `v-once` element caches
/// that element.
pub(super) fn generate_if_branch_content(
    ctx: &mut CodegenContext,
    children: &[TemplateChildNode<'_>],
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    let once = matches!(children, [TemplateChildNode::Element(el)] if wrapped_once(el));
    generate(ctx, children, branch, branch_index, once);
}

/// A patterned-template scope that is the branch: its content carries the
/// branch key. `v-once` on a `<template v-when>` arm lands on the scope and
/// caches the scope with its bindings; on an element arm it caches the element.
pub(super) fn generate_scope_branch(
    ctx: &mut CodegenContext,
    for_node: &ForNode<'_>,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    let scope = |ctx: &mut CodegenContext| {
        generate_match_scope(ctx, for_node, &|ctx, children| {
            let once = matches!(children, [TemplateChildNode::Element(el)] if has_v_once(el));
            generate(ctx, children, branch, branch_index, once);
        });
    };
    let scope_once = matches!(
        for_node.children.as_slice(),
        [TemplateChildNode::Element(el)] if has_v_once(el)
    );
    if scope_once {
        generate_v_once_cached(ctx, scope);
    } else {
        scope(ctx);
    }
}

fn generate(
    ctx: &mut CodegenContext,
    children: &[TemplateChildNode<'_>],
    branch: &IfBranchNode<'_>,
    branch_index: usize,
    once: bool,
) {
    if once {
        generate_v_once_cached(ctx, |ctx| {
            generate_if_branch_nodes(ctx, children, branch, branch_index)
        });
    } else {
        generate_if_branch_nodes(ctx, children, branch, branch_index);
    }
}

fn wrapped_once(el: &ElementNode<'_>) -> bool {
    el.tag_type == ElementType::Template
        && matches!(
            el.children.as_slice(),
            [TemplateChildNode::Element(inner)] if has_v_once(inner)
        )
}
