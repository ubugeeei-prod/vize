//! `<slot/>` outlets that are themselves the `v-for` item.
//!
//! A slot outlet renders through `renderSlot` rather than as a vnode, so it
//! never takes the element/component path in
//! [`generate_for_item`](super::generate::generate_for_item). It is separated
//! here for the same reason `slots/outlet.rs` is separate from
//! `slots/generate.rs`: outlet rendering and vnode generation share nothing but
//! the codegen context.

use crate::{ElementNode, RuntimeHelper};

use super::super::{
    children::is_directive_comment,
    context::CodegenContext,
    node::generate_node,
    slots::{generate_slot_outlet_name, generate_slot_outlet_props, has_slot_outlet_props},
};

pub(super) fn generate_for_slot_outlet(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    ctx.use_helper(RuntimeHelper::RenderSlot);
    ctx.push(ctx.helper(RuntimeHelper::RenderSlot));
    ctx.push("(_ctx.$slots, ");
    generate_slot_outlet_name(ctx, el);

    let has_slot_props = has_slot_outlet_props(el);
    let filtered: Vec<_> = el
        .children
        .iter()
        .filter(|c| !is_directive_comment(c))
        .collect();

    if !filtered.is_empty() {
        if has_slot_props {
            ctx.push(", ");
            generate_slot_outlet_props(ctx, el);
        } else {
            ctx.push(", {}");
        }
        ctx.push(", () => [");
        ctx.indent();
        for (i, child) in filtered.iter().enumerate() {
            if i > 0 {
                ctx.push(",");
            }
            ctx.newline();
            generate_node(ctx, child);
        }
        ctx.deindent();
        ctx.newline();
        ctx.push("])");
    } else if has_slot_props {
        ctx.push(", ");
        generate_slot_outlet_props(ctx, el);
        ctx.push(")");
    } else {
        ctx.push(")");
    }
}
