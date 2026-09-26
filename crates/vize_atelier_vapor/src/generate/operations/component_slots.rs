use crate::ir::{ComponentKind, IRSlot};
use vize_carton::{FxHashMap, String, cstr};

use super::super::{context::GenerateContext, generate_block};

/// Generate a single slot function body
pub(super) fn generate_slot_fn(
    ctx: &mut GenerateContext,
    slot: &IRSlot<'_>,
    element_template_map: &FxHashMap<usize, usize>,
    kind: ComponentKind,
) {
    let slot_props_var = slot
        .fn_exp
        .as_ref()
        .map(|fn_exp| ctx.push_slot_scope(fn_exp.content));
    let keep_alive = kind == ComponentKind::KeepAlive;
    if keep_alive {
        ctx.use_helper("extend");
        let param = slot_props_var
            .as_ref()
            .map(|v| cstr!(" _extend(({}) => {{\n", v))
            .unwrap_or_else(|| String::from(" _extend(() => {\n"));
        ctx.push(&param);
    } else {
        // The slot function opens at the authored `<template #slot>`.
        let param: String = slot_props_var
            .as_ref()
            .map(|v| cstr!("({}) => {{\n", v))
            .unwrap_or_else(|| String::from("() => {\n"));
        ctx.push(" ");
        let unit = ctx.unit_of(&slot.name);
        let opened = ctx.spanned_at(&param, unit);
        ctx.push_spanned(&opened);
    }
    ctx.indent();
    ctx.push_component_scope();
    let previous = ctx.keep_alive_slot;
    ctx.keep_alive_slot = keep_alive;
    generate_block(ctx, &slot.block, element_template_map);
    ctx.keep_alive_slot = previous;
    ctx.pop_component_scope();
    ctx.deindent();
    ctx.push_indent();
    ctx.push("}");
    if keep_alive {
        ctx.push(", { _: 1 })");
    }
    if slot_props_var.is_some() {
        ctx.pop_slot_scope();
    }
}
