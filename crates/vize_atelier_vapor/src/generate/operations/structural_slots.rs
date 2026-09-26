//! Runtime slot selection follows compiler-vapor's conditional and createForSlots ABI.

use super::{
    super::context::{ForScope, GenerateContext},
    component_slots::generate_slot_fn,
};
use crate::ir::{ComponentKind, CreateComponentIRNode, IRSlot, IRSlotControl, IRSlotLoop};
use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::{FxHashMap, String, cstr};

fn cache_name(component: usize, slot: usize, branch: usize) -> String {
    if branch == 0 {
        cstr!("_slot{component}_{slot}")
    } else {
        cstr!("_slot{component}_{slot}_{branch}")
    }
}

pub(super) fn declare_slot_caches(
    ctx: &mut GenerateContext,
    component: &CreateComponentIRNode<'_>,
) {
    for (index, slot) in component
        .slots
        .iter()
        .filter(|slot| slot.dynamic())
        .enumerate()
    {
        if matches!(slot.control, Some(IRSlotControl::For(_))) {
            continue;
        }
        for (branch, _) in slot.blocks().enumerate() {
            ctx.push_line(&cstr!("let {}", cache_name(component.id, index, branch)));
        }
    }
}

pub(super) fn generate_dynamic_slot(
    ctx: &mut GenerateContext,
    slot: &IRSlot<'_>,
    templates: &FxHashMap<usize, usize>,
    component: usize,
    index: usize,
) {
    ctx.push_indent();
    if let Some(IRSlotControl::For(each)) = &slot.control {
        looped(ctx, slot, each, templates);
    } else if slot.control.is_some() {
        ctx.push("() => (");
        conditional(ctx, slot, templates, component, index, 0);
        ctx.push(")");
    } else {
        ctx.push("() => (");
        payload(ctx, slot, templates, &cache_name(component, index, 0));
        ctx.push(")");
    }
}

fn payload(
    ctx: &mut GenerateContext,
    slot: &IRSlot<'_>,
    templates: &FxHashMap<usize, usize>,
    cache: &str,
) {
    ctx.push("{\n");
    ctx.indent();
    let mut name = EmitDocument::plain("name: ");
    if slot.name.is_static {
        name.push_str("\"");
        name.push_spanned(&ctx.spanned_at(
            &super::super::escape_js_string_literal(slot.name.content),
            Some(slot.name.loc.span.start),
        ));
        name.push_str("\"");
    } else {
        name.push_spanned(&ctx.spanned_expression_node(&slot.name));
    }
    name.push_str(",");
    ctx.push_line_spanned(&name);
    ctx.push_indent();
    ctx.push(&cstr!("fn: {cache} || ({cache} ="));
    generate_slot_fn(ctx, slot, templates, ComponentKind::Regular);
    ctx.push(")\n");
    ctx.deindent();
    ctx.push_indent();
    ctx.push("}");
}

fn conditional(
    ctx: &mut GenerateContext,
    slot: &IRSlot<'_>,
    templates: &FxHashMap<usize, usize>,
    component: usize,
    index: usize,
    branch: usize,
) {
    if let Some(IRSlotControl::If {
        condition,
        negative,
    }) = &slot.control
    {
        let condition = ctx.spanned_expression_node(condition);
        ctx.push_spanned(&condition);
        ctx.push(" ? ");
        payload(ctx, slot, templates, &cache_name(component, index, branch));
        ctx.push(" : ");
        if let Some(negative) = negative {
            conditional(ctx, negative, templates, component, index, branch + 1);
        } else {
            ctx.push("undefined");
        }
    } else {
        payload(ctx, slot, templates, &cache_name(component, index, branch));
    }
}

fn looped(
    ctx: &mut GenerateContext,
    slot: &IRSlot<'_>,
    each: &IRSlotLoop<'_>,
    templates: &FxHashMap<usize, usize>,
) {
    ctx.use_helper("createForSlots");
    ctx.push("_createForSlots(() => (");
    let source = ctx.spanned_expression_node(&each.source);
    ctx.push_spanned(&source);
    let depth = ctx.for_scopes.len();
    let mut params = cstr!("_for_item{depth}");
    if each.key.is_some() {
        params.push_str(&cstr!(", _for_key{depth}"));
    }
    if each.index.is_some() {
        params.push_str(&cstr!(", _for_index{depth}"));
    }
    ctx.push(&cstr!("), ({params}) =>"));
    ctx.for_scopes.push(ForScope {
        value_alias: Some(each.value.content.into()),
        key_alias: each.key.as_ref().map(|key| key.content.into()),
        index_alias: each.index.as_ref().map(|index| index.content.into()),
        depth,
    });
    generate_slot_fn(ctx, slot, templates, ComponentKind::Regular);
    // Name/key callbacks receive raw values, while the slot body owns reactive refs.
    let aliases = [
        Some(each.value.as_ref()),
        each.key.as_deref(),
        each.index.as_deref(),
    ];
    let mut raw = String::new("");
    for alias in aliases.iter().flatten() {
        if !raw.is_empty() {
            raw.push_str(", ");
        }
        raw.push_str(alias.content);
        ctx.push_slot_scope(alias.content);
    }
    ctx.push(&cstr!(", ({raw}) => ("));
    if slot.name.is_static {
        ctx.push(&cstr!(
            "\"{}\"",
            super::super::escape_js_string_literal(slot.name.content)
        ));
    } else {
        let name = ctx.spanned_expression_node(&slot.name);
        ctx.push_spanned(&name);
    }
    ctx.push(")");
    if let Some(key) = &each.key_prop {
        ctx.push(&cstr!(", ({raw}) => ("));
        let key = ctx.spanned_expression_node(key);
        ctx.push_spanned(&key);
        ctx.push(")");
    }
    for _ in aliases.iter().flatten() {
        ctx.pop_slot_scope();
    }
    ctx.for_scopes.pop();
    ctx.push(")");
}
