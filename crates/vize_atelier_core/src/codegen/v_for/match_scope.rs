//! The lexical scope a patterned-template `v-match` lowers to (RFC 823).
//!
//! The selector result and each arm's bindings are declared once in an
//! immediately invoked scope, and the scope returns its content directly. No
//! list is rendered, so no fragment is introduced: a single-root match keeps
//! attribute fallthrough, `Transition` / `KeepAlive` still receive one child,
//! and client, server and hydrated output agree node for node.

use crate::{ElementType, ForNode, RuntimeHelper, TemplateChildNode};
use vize_s0::String;

use super::super::{
    children::is_directive_comment, context::CodegenContext, element::helpers::generate_root_node,
    expression::generate_expression, node::generate_node,
};
use super::helpers::extract_for_params;

/// The content a scope renders: the children of its synthetic `<template>`.
pub(crate) fn scope_children<'n, 'a>(for_node: &'n ForNode<'a>) -> &'n [TemplateChildNode<'a>] {
    match for_node.children.as_slice() {
        [TemplateChildNode::Element(el)] if el.tag_type == ElementType::Template => &el.children,
        children => children,
    }
}

/// Emit `(() => { const <aliases> = <source>; return <body> })()`.
pub(crate) fn generate_match_scope(
    ctx: &mut CodegenContext,
    for_node: &ForNode<'_>,
    body: &dyn Fn(&mut CodegenContext, &[TemplateChildNode<'_>]),
) {
    let mut bindings: Vec<String> = Vec::new();
    ctx.push("(() => { const ");
    if let Some(value) = &for_node.value_alias {
        generate_expression(ctx, value);
        extract_for_params(value, &mut bindings);
    }
    ctx.push(" = ");
    generate_expression(ctx, &for_node.source);
    ctx.push("; return ");
    // The bindings are scope variables for everything rendered inside: they
    // are never prefixed, and a handler that reads one is never cached.
    ctx.add_slot_params(&bindings);
    let outer = std::mem::replace(&mut ctx.in_match_scope, true);
    body(ctx, scope_children(for_node));
    ctx.in_match_scope = outer;
    ctx.remove_slot_params(&bindings);
    ctx.push(" })()");
}

/// A scope body outside a `v-if` branch: one block root, or a stable fragment.
pub(crate) fn generate_scope_body(ctx: &mut CodegenContext, children: &[TemplateChildNode<'_>]) {
    let rendered: Vec<&TemplateChildNode<'_>> = children
        .iter()
        .filter(|child| !is_directive_comment(child))
        .collect();
    match rendered.as_slice() {
        [] => ctx.push("null"),
        [child] => generate_root_node(ctx, child),
        children => {
            ctx.use_helper(RuntimeHelper::OpenBlock);
            ctx.use_helper(RuntimeHelper::CreateElementBlock);
            ctx.use_helper(RuntimeHelper::Fragment);
            ctx.push("(");
            ctx.push_vnode_helper(RuntimeHelper::OpenBlock);
            ctx.push("(), ");
            ctx.push_vnode_helper(RuntimeHelper::CreateElementBlock);
            ctx.push("(");
            ctx.push(ctx.helper(RuntimeHelper::Fragment));
            ctx.push(", null, [");
            ctx.indent();
            for (index, child) in children.iter().enumerate() {
                if index > 0 {
                    ctx.push(",");
                }
                ctx.newline();
                generate_node(ctx, child);
            }
            ctx.deindent();
            ctx.newline();
            ctx.push("], 64 /* STABLE_FRAGMENT */))");
        }
    }
}
