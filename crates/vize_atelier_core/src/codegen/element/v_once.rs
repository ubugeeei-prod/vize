//! v-once element generation.
//!
//! Generates cache-wrapped elements for the `v-once` directive, which
//! renders elements once and caches them for subsequent renders.

use crate::{ElementNode, ElementType, RuntimeHelper};

use super::super::context::CodegenContext;
use vize_l0::ToCompactString;

/// Generate v-once element with cache wrapper
pub fn generate_v_once_element(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    generate_v_once_cached(ctx, |ctx| generate_v_once_vnode(ctx, el));
}

/// Wrap `body` in the `v-once` cache expression: it renders on the first pass
/// only, with block tracking paused so the cached tree is never patched.
pub fn generate_v_once_cached(ctx: &mut CodegenContext, body: impl FnOnce(&mut CodegenContext)) {
    let cache_index = ctx.next_cache_index();

    ctx.use_helper(RuntimeHelper::SetBlockTracking);

    // _cache[0] || (...)
    ctx.push("_cache[");
    ctx.push(&cache_index.to_compact_string());
    ctx.push("] || (");
    ctx.indent();
    ctx.newline();

    // _setBlockTracking(-1, true),
    ctx.push(ctx.helper(RuntimeHelper::SetBlockTracking));
    ctx.push("(-1, true),");
    ctx.newline();

    // (_cache[0] = <body>).cacheIndex = 0,
    ctx.push("(_cache[");
    ctx.push(&cache_index.to_compact_string());
    ctx.push("] = ");
    body(ctx);
    ctx.push(").cacheIndex = ");
    ctx.push(&cache_index.to_compact_string());
    ctx.push(",");
    ctx.newline();

    // _setBlockTracking(1),
    ctx.push(ctx.helper(RuntimeHelper::SetBlockTracking));
    ctx.push("(1),");
    ctx.newline();

    // _cache[0]
    ctx.push("_cache[");
    ctx.push(&cache_index.to_compact_string());
    ctx.push("]");

    ctx.deindent();
    ctx.newline();
    ctx.push(")");
}

/// The vnode a `v-once` element caches: the same vnode as without `v-once`,
/// minus handler caching.
fn generate_v_once_vnode(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    let in_v_once = core::mem::replace(&mut ctx.in_v_once, true);
    if el.tag_type == ElementType::Component {
        super::inline::generate_component_vnode(ctx, el);
    } else {
        super::inline::generate_native_vnode(ctx, el);
    }
    ctx.in_v_once = in_v_once;
}
