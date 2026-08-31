//! Static-name component emission (`resolveComponent` / `createVNode` /
//! `createBlock`) plus slot objects from [`SlotFacts`] (implicit
//! default, named `<template>` groups, component-root `v-slot`) and
//! `createSlots` for `v-if` / `v-for` slot templates, the `v-slots`
//! spread, Vue builtins, and `<component :is>`
//! (`resolveDynamicComponent`).

mod preamble;

use alloc::vec::Vec as StdVec;

use vize_davinci::id::NodeId;
use vize_s2::op::{BindingOp, ComponentOp};

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::builtin;
use super::children::children_need_text_flag;
use super::create_slots;
use super::directive;
use super::flag::emit_patch_flag;
use super::hoist::compact_props_object;
use super::js::asset_ident;
use super::props::{admit_bindings, apply_static_ref_patch, bind_patch, emit_bind_props};
use super::props_static;
use super::props_static::PropHoistPosition;
use super::slots;

pub(super) use preamble::{collect_names, emit_resolves};

pub(super) fn emit_root(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    if super::once::has(&component.bindings) {
        return super::once::emit_component(cx, component, None, false, id);
    }
    if super::memo::has(&component.bindings) && !cx.skip_memo {
        return super::memo::emit_cached(cx, &component.bindings, |cx| {
            emit_vnode(cx, component, None, false, id)
        });
    }
    emit_block(cx, component, None, false, id)
}

fn emit_block(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    if_key: Option<&str>,
    for_item: bool,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    directive::wrap_component(cx, component, |cx| {
        cx.buf.use_open_block();
        cx.buf.use_create_block();
        cx.buf.push("(");
        cx.buf.push(Buf::open_block_alias());
        cx.buf.push("(), ");
        emit_call(cx, component, /* block */ true, if_key, for_item, id)?;
        cx.buf.push(")");
        Ok(())
    })
}

fn emit_vnode(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    if_key: Option<&str>,
    for_item: bool,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    directive::wrap_component(cx, component, |cx| {
        cx.buf.use_create_vnode();
        emit_call(cx, component, false, if_key, for_item, id)
    })
}

pub(super) fn emit_nested(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    if super::once::has(&component.bindings) {
        return super::once::emit_component(cx, component, None, false, id);
    }
    if super::memo::has(&component.bindings) && !cx.skip_memo {
        return super::memo::emit_cached(cx, &component.bindings, |cx| {
            emit_vnode(cx, component, None, false, id)
        });
    }
    if builtin::forces_block(component) {
        return emit_root(cx, component, id);
    }
    emit_vnode(cx, component, None, false, id)
}

pub(super) fn emit_if_branch(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    key: &str,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    if super::once::has(&component.bindings) {
        return super::once::emit_component(cx, component, Some(key), false, id);
    }
    emit_block(cx, component, Some(key), false, id)
}

pub(super) fn emit_for_item(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    id: Option<NodeId>,
    key: Option<&str>,
) -> Result<(), EmitError> {
    if super::once::has(&component.bindings) {
        return super::once::emit_component(cx, component, key, true, id);
    }
    if let Some(memo) = super::memo::first(&component.bindings)
        && !cx.skip_memo
    {
        return super::memo::emit_cached_with_key(cx, memo, key.unwrap_or("0"), |cx| {
            emit_block(cx, component, key, true, id)
        });
    }
    emit_block(cx, component, key, true, id)
}

fn emit_call(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    block: bool,
    if_key: Option<&str>,
    for_item: bool,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    admit(cx, component)?;
    let facts = id.and_then(|id| cx.facts.slot_facts.get(id));
    let create = create_slots::needs_create_slots(cx, &component.children);
    let spread = slots::slots_spread(&component.bindings)?;
    let array = builtin::array_children(component.name);
    if array && (create || spread.is_some()) {
        return Err(EmitError::unsupported_at(
            Reason::ArrayBuiltinCannotUseSlotObject,
            component.span,
        ));
    }
    let has_array = array && slots::has_implicit_default(&component.children);
    let has_slots = !array && (facts.is_some() || create || spread.is_some());
    let dynamic_names = create || facts.is_some_and(slots::has_dynamic_names) || spread.is_some();
    let transition_slot_root = builtin::transition_slot_root(component.name);
    let alias = if block {
        Buf::create_block_alias()
    } else {
        Buf::create_vnode_alias()
    };
    cx.buf.push(alias);
    cx.buf.push("(");
    if builtin::emit_dynamic_tag(cx, component)? {
    } else if let Some(helper) = builtin::helper(component.name) {
        cx.buf.use_helper(helper);
        cx.buf.push(helper.alias());
    } else {
        cx.buf
            .push(asset_ident("component", component.name).as_str());
    }
    let skip_is = builtin::is_dynamic_component(component);
    let has_binds = component.bindings.iter().any(|binding| {
        !(matches!(binding, BindingOp::SlotContent(_))
            || slots::is_slots_spread(binding)
            || directive::is_runtime(binding)
            || super::memo::is_memo(binding)
            || super::once::is_once(binding)
            || matches!(binding, BindingOp::VueCloak(_))
            || (skip_is && builtin::is_is_bind(binding)))
    });
    let has_attrs = component
        .attributes
        .iter()
        .any(|attr| !skip_is || attr.name != "is");
    let has_custom = directive::has_custom(&component.bindings);
    let hoist_attrs: StdVec<_> = component
        .attributes
        .iter()
        .filter(|attr| !skip_is || attr.name != "is")
        .collect();
    let has_hoist_attrs = !hoist_attrs.is_empty();
    let static_nested = builtin::has_static_nested(&component.children);
    let builtin_helper = builtin::helper(component.name).is_some();
    let hoistable_static_props = if skip_is {
        has_hoist_attrs.then(|| compact_props_object(hoist_attrs.iter().copied()))
    } else {
        props_static::root_hoist_props(&component.attributes, &component.bindings)?
    };
    if for_item
        && !has_custom
        && hoistable_static_props.is_some()
        && props_static::should_hoist(cx, id, PropHoistPosition::ForItem)
    {
        cx.buf.push_hoist(
            hoistable_static_props
                .clone()
                .expect("checked hoisted props"),
        );
    }
    let can_hoist_static_props = !has_custom
        && !for_item
        && if_key.is_none()
        && hoistable_static_props.is_some()
        && props_static::should_hoist(cx, id, PropHoistPosition::Nested);
    let hoisted_static_props = if can_hoist_static_props
        && ((!array && (facts.is_some() || create) && (!builtin_helper || static_nested))
            || (array && static_nested))
    {
        Some(
            cx.buf.push_hoist(
                hoistable_static_props
                    .clone()
                    .expect("checked hoisted props"),
            ),
        )
    } else {
        None
    };
    let unused_hoist = hoisted_static_props.is_none() && can_hoist_static_props && static_nested;
    if unused_hoist {
        cx.buf
            .push_hoist(hoistable_static_props.expect("checked hoisted props"));
    }
    let mut patch = bind_patch(&component.bindings, true, if_key, for_item);
    if skip_is {
        patch.dynamic_props.retain(|name| name.as_str() != "is");
        if patch.dynamic_props.is_empty() {
            patch.flag &= !8;
        }
    }
    let mut flag = patch.flag;
    if array && children_need_text_flag(&component.children) {
        flag |= 1;
    }
    if (cx.in_v_for && has_slots)
        || dynamic_names
        || builtin::always_dynamic_slots(component.name)
        || (cx.slot_param_depth > 0 && super::outlet::has_forwarded_outlet(&component.children))
    {
        flag |= 1024;
    }
    apply_static_ref_patch(&component.attributes, &mut flag);
    if for_item {
        flag &= !512;
    }
    let emit_flag = flag != 0;
    if let Some(alias) = hoisted_static_props.as_ref() {
        cx.buf.push(", ");
        cx.buf.push(alias.as_str());
    } else if if_key.is_some() || has_binds || has_attrs {
        cx.buf.push(", ");
        emit_bind_props(
            cx,
            &component.attributes,
            &component.bindings,
            if_key,
            skip_is,
            for_item,
            false,
        )?;
    } else if for_item && directive::has_custom(&component.bindings) {
        cx.buf.push(", { }");
    } else if emit_flag || has_slots || has_array {
        cx.buf.push(", null");
    }
    if array {
        if has_array {
            cx.buf.push(", ");
            cx.with_static_vnode_hoist(true, |cx| {
                builtin::emit_array_children(cx, &component.children, if_key.is_some())
            })?;
        } else if emit_flag {
            cx.buf.push(", null");
        }
    } else if create {
        cx.buf.push(", ");
        cx.with_static_vnode_hoist(true, |cx| {
            create_slots::emit_create_slots(cx, &component.children, spread.as_ref())
        })?;
    } else if let Some(facts) = facts {
        cx.buf.push(", ");
        cx.with_static_vnode_hoist(true, |cx| {
            let previous = cx.transition_slot_root;
            cx.transition_slot_root = previous || transition_slot_root;
            let result = slots::emit_slots(cx, &component.children, facts, spread.as_ref());
            cx.transition_slot_root = previous;
            result
        })?;
    } else if let Some(spread) = spread {
        cx.buf.push(", ");
        cx.buf.push(spread.as_str());
    } else if emit_flag {
        cx.buf.push(", null");
    }
    if emit_flag {
        emit_patch_flag(cx, flag);
    }
    if !patch.dynamic_props.is_empty() {
        cx.buf.push(", [");
        for (i, name) in patch.dynamic_props.iter().enumerate() {
            if i > 0 {
                cx.buf.push(", ");
            }
            cx.buf.push("\"");
            cx.buf.push(name.as_str());
            cx.buf.push("\"");
        }
        cx.buf.push("]");
    }
    cx.buf.push(")");
    Ok(())
}

fn admit(cx: &EmitCx<'_>, component: &ComponentOp<'_>) -> Result<(), EmitError> {
    if create_slots::needs_create_slots(cx, &component.children)
        || slots::has_implicit_default(&component.children)
    {
        slots::admit_default(&component.children)?;
    }
    admit_bindings(&component.attributes, &component.bindings)
}
