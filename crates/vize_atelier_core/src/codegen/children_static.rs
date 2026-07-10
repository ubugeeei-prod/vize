//! Static-subtree cache emission.
//!
//! Emits hoisted `_cache[...]` static vnodes for subtrees the static-hoist pass
//! marked cacheable. Split out of `children` so the children dispatch stays
//! focused on grouping and the Relief projection stream.

use crate::steps::hoist_static::is_static_node;
use crate::{ElementNode, RuntimeHelper, TemplateChildNode};

use super::children::generate_children;
use super::context::CodegenContext;
use super::element::helpers::{child_namespace, has_renderable_props};
use super::props::generate_props;
use vize_carton::ToCompactString;

pub(crate) fn is_static_cacheable_element(child: &TemplateChildNode<'_>) -> bool {
    matches!(child, TemplateChildNode::Element(_)) && is_static_node(child)
}

pub(crate) fn generate_cached_static_children_array(
    ctx: &mut CodegenContext,
    children: &[&TemplateChildNode<'_>],
) {
    let cache_index = ctx.next_cache_index();
    ctx.push("[...(_cache[");
    ctx.push(&cache_index.to_compact_string());
    ctx.push("] || (_cache[");
    ctx.push(&cache_index.to_compact_string());
    ctx.push("] = [");
    ctx.indent();

    for (i, child) in children.iter().enumerate() {
        if i > 0 {
            ctx.push(",");
        }
        ctx.newline();
        if let TemplateChildNode::Element(el) = child {
            generate_cached_static_vnode(ctx, el, true);
        }
    }

    ctx.deindent();
    ctx.newline();
    ctx.push("]))]");
}

pub(crate) fn generate_cached_static_element(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    let cache_index = ctx.next_cache_index();
    ctx.push("_cache[");
    ctx.push(&cache_index.to_compact_string());
    ctx.push("] || (_cache[");
    ctx.push(&cache_index.to_compact_string());
    ctx.push("] = ");
    generate_cached_static_vnode(ctx, el, true);
    ctx.push(")");
}

/// Emit one static element as `createElementVNode(...)`.
///
/// `cached` controls whether this vnode is the top-most cached node of a static
/// subtree (gets the `-1 /* CACHED */` patch flag) or a descendant inside an
/// already-cached subtree (plain vnode, no flag), matching how
/// @vue/compiler-core serializes a cached static subtree: a single cache entry
/// whose children are plain recursive `createElementVNode` calls.
pub(crate) fn generate_cached_static_vnode(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    cached: bool,
) {
    ctx.use_helper(RuntimeHelper::CreateElementVNode);
    ctx.push(ctx.helper(RuntimeHelper::CreateElementVNode));
    ctx.push("(\"");
    ctx.push(&el.tag);
    ctx.push("\"");

    if has_renderable_props(el) {
        ctx.push(", ");
        ctx.props_is_plain_element = true;
        generate_props(ctx, &el.props);
        ctx.props_is_plain_element = false;
    } else {
        ctx.push(", null");
    }

    if !el.children.is_empty() {
        ctx.push(", ");
        // Descendants of a cached subtree are emitted as plain vnodes: suppress
        // the cache wrapper and the per-descendant CACHED flag while recursing.
        let prev_in_cached = ctx.in_cached_static;
        ctx.in_cached_static = true;
        ctx.with_parent_namespace(child_namespace(el), |ctx| {
            generate_children(ctx, &el.children);
        });
        ctx.in_cached_static = prev_in_cached;
    } else {
        ctx.push(", null");
    }

    if cached {
        ctx.push(", -1 /* CACHED */)");
    } else {
        ctx.push(")");
    }
}
