//! Static-name component emission (`resolveComponent` / `createVNode` /
//! `createBlock`) plus slot objects from [`SlotFacts`] (implicit
//! default, named `<template>` groups, component-root `v-slot`) and
//! `createSlots` for `v-if` / `v-for` slot templates, the `v-slots`
//! spread, Vue builtins, and `<component :is>`
//! (`resolveDynamicComponent`).

mod call_props;
mod checks;
mod preamble;

use call_props::{
    emit_dynamic_props, has_rendered_attrs, has_rendered_binds, rendered_hoist_attrs,
};
use checks::{admit, has_dynamic_key_binding};

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
use super::js::asset_ident;
use super::props::{
    BindPropsOptions, apply_static_ref_patch, bind_patch, emit_bind_props,
    prune_legacy_patchless_dynamic_props,
};
use super::props_static;
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
    if has_dynamic_key_binding(component) {
        return emit_block(cx, component, None, false, id);
    }
    emit_vnode(cx, component, None, false, id)
}

pub(super) fn emit_if_branch(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    key: &str,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    emit_block(cx, component, Some(key), false, id)
}

pub(super) fn emit_for_item(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    id: Option<NodeId>,
    key: Option<&str>,
) -> Result<(), EmitError> {
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
    let filler_default_props_placeholder =
        !array && !has_slots && slots::filler_default_needs_props_placeholder(&component.children);
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
    let has_binds = has_rendered_binds(component, skip_is);
    let has_attrs = has_rendered_attrs(component, skip_is);
    let has_custom = directive::has_custom(&component.bindings);
    let has_component_root_slot = component
        .bindings
        .iter()
        .any(|binding| matches!(binding, BindingOp::SlotContent(_)));
    let hoist_attrs = rendered_hoist_attrs(component, skip_is);
    let static_nested = builtin::has_static_nested(&component.children);
    let hoistable_static_props =
        call_props::hoistable_static_props(component, skip_is, &hoist_attrs)?;
    if for_item
        && !has_custom
        && let Some(props) = hoistable_static_props.as_ref()
        && props.non_key
        && (props_static::should_hoist(cx, id, props_static::PropHoistPosition::ForItem)
            || (props.dynamic_values && !has_slots && !has_array))
    {
        cx.buf.push_hoist(props.source.clone());
    }
    let static_props_hoist_context =
        cx.hoist_static_vnodes || cx.slot_param_depth > 0 || cx.in_v_for;
    let can_hoist_static_props = !has_custom
        && !for_item
        && if_key.is_none()
        && hoistable_static_props.is_some()
        && hoistable_static_props.as_ref().is_some_and(|props| {
            props_static::should_hoist(cx, id, props_static::PropHoistPosition::Nested)
                || id.is_some_and(|id| {
                    cx.template_for_item_root_id == Some(id)
                        && props_static::props_hoistable(cx, Some(id))
                        && !directive::has_runtime(&component.bindings)
                })
                || (!props.dynamic_values
                    && props.valued_prop
                    && static_props_hoist_context
                    && !has_slots)
                || (props.dynamic_values
                    && cx.slot_param_depth == 0
                    && !cx.in_v_for
                    && (!has_slots || slots::has_text_only_implicit_default(&component.children)))
        });
    let foreign_static_props = id
        .and_then(|id| cx.facts.static_facts.get(id))
        .is_some_and(|fact| fact.foreign && fact.props_hoistable);
    let hoisted_static_props = if can_hoist_static_props
        && ((!array && (facts.is_some() || create || foreign_static_props))
            || (array && static_nested))
    {
        Some(
            cx.buf.push_hoist(
                hoistable_static_props
                    .as_ref()
                    .expect("checked hoisted props")
                    .source
                    .clone(),
            ),
        )
    } else {
        None
    };
    let branch_unused_hoist = !has_custom
        && !for_item
        && if_key.is_some()
        && cx.template_if_branch_root
        && hoistable_static_props.is_some()
        && static_nested;
    let unused_hoist = hoisted_static_props.is_none()
        && ((can_hoist_static_props && static_nested) || branch_unused_hoist);
    if unused_hoist {
        cx.buf.push_hoist(
            hoistable_static_props
                .as_ref()
                .expect("checked hoisted props")
                .source
                .clone(),
        );
    }
    let mut patch = bind_patch(&component.bindings, true, if_key, for_item);
    if skip_is {
        patch.dynamic_props.retain(|name| name.as_str() != "is");
        if patch.dynamic_props.is_empty() {
            patch.flag &= !8;
        }
        if directive::has_runtime(&component.bindings) && patch.flag & (2 | 4 | 8 | 16) == 0 {
            patch.flag |= 512;
        }
    }
    if has_slots {
        prune_legacy_patchless_dynamic_props(&component.bindings, &mut patch.dynamic_props);
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
            BindPropsOptions {
                if_key,
                skip_is,
                for_item,
                is_plain_element: false,
                once_layout: false,
                once_cache_initializer: false,
                force_multiline: for_item && if_key.is_some() && has_component_root_slot,
            },
        )?;
    } else if for_item && directive::has_custom(&component.bindings) {
        cx.buf.push(", { }");
    } else if emit_flag || has_slots || has_array || filler_default_props_placeholder {
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
            let result = slots::emit_slots(
                cx,
                &component.children,
                facts,
                spread.as_ref(),
                &component.bindings,
            );
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
    emit_dynamic_props(cx, &patch.dynamic_props);
    cx.buf.push(")");
    Ok(())
}
