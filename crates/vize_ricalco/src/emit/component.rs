//! Static-name component emission (`resolveComponent` / `createVNode` /
//! `createBlock`) plus slot objects from [`SlotFacts`] (implicit
//! default, named `<template>` groups, component-root `v-slot`) and
//! `createSlots` for `v-if` / `v-for` slot templates, the `v-slots`
//! spread, and Vue builtins (`Teleport` / `KeepAlive` / `Transition` /
//! `Suspense` / `TransitionGroup` / `BaseTransition`). `<component :is>`
//! stays unsupported.

use alloc::vec::Vec as StdVec;

use vize_davinci::id::NodeId;
use vize_disegno::op::{BindingOp, ComponentOp, Op, Region};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::builtin;
use super::children::children_need_text_flag;
use super::create_slots;
use super::flag::emit_patch_flag;
use super::hoist::compact_props_object;
use super::js::asset_ident;
use super::props::{admit_bindings, bind_patch, emit_bind_props};
use super::slots;

pub(super) fn collect_names<'a>(root: &Region<'a>) -> StdVec<&'a str> {
    let mut names = StdVec::new();
    collect_from(root, &mut names);
    names
}

pub(super) fn emit_resolves(cx: &mut EmitCx<'_>, names: &[&str]) {
    cx.buf.use_resolve_component();
    for name in names {
        cx.buf.push("const ");
        cx.buf.push(asset_ident("component", name).as_str());
        cx.buf.push(" = ");
        cx.buf.push(Buf::resolve_component_alias());
        cx.buf.push("(\"");
        cx.buf.push(name);
        cx.buf.push("\")");
        cx.buf.newline();
    }
}

pub(super) fn emit_root(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    cx.buf.use_open_block();
    cx.buf.use_create_block();
    cx.buf.push("(");
    cx.buf.push(Buf::open_block_alias());
    cx.buf.push("(), ");
    emit_call(
        cx, component, /* block */ true, None, /* for_item */ false, id,
    )?;
    cx.buf.push(")");
    Ok(())
}

pub(super) fn emit_nested(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    if builtin::forces_block(component.name) {
        return emit_root(cx, component, id);
    }
    cx.buf.use_create_vnode();
    emit_call(
        cx, component, /* block */ false, None, /* for_item */ false, id,
    )
}

pub(super) fn emit_if_branch(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    key: &str,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    cx.buf.use_open_block();
    cx.buf.use_create_block();
    cx.buf.push("(");
    cx.buf.push(Buf::open_block_alias());
    cx.buf.push("(), ");
    emit_call(
        cx,
        component,
        /* block */ true,
        Some(key),
        /* for_item */ false,
        id,
    )?;
    cx.buf.push(")");
    Ok(())
}

pub(super) fn emit_for_item(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    cx.buf.use_open_block();
    cx.buf.use_create_block();
    cx.buf.push("(");
    cx.buf.push(Buf::open_block_alias());
    cx.buf.push("(), ");
    emit_call(
        cx, component, /* block */ true, None, /* for_item */ true, id,
    )?;
    cx.buf.push(")");
    Ok(())
}

fn collect_from<'a>(region: &Region<'a>, names: &mut StdVec<&'a str>) {
    for op in region.ops.iter() {
        match op {
            Op::Element(element) => collect_from(&element.children, names),
            Op::Component(component) => {
                collect_from(&component.children, names);
                if !is_builtin(component.name) && !names.iter().any(|seen| *seen == component.name)
                {
                    names.push(component.name);
                }
            }
            Op::If(if_op) => {
                for branch in if_op.branches.iter() {
                    collect_from(&branch.region, names);
                }
            }
            Op::For(for_op) => collect_from(&for_op.region, names),
            _ => {}
        }
    }
}

fn emit_call(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    block: bool,
    if_key: Option<&str>,
    for_item: bool,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    admit(component)?;
    let facts = id.and_then(|id| cx.facts.slot_facts.get(id));
    let create = create_slots::needs_create_slots(&component.children);
    let spread = slots::slots_spread(&component.bindings)?;
    let array = builtin::array_children(component.name);
    if (create && spread.is_some()) || (array && (create || spread.is_some())) {
        return Err(EmitError::Unsupported);
    }
    let has_array = array && slots::has_implicit_default(&component.children);
    let has_slots = !array && (facts.is_some() || create || spread.is_some());
    let dynamic_names = create || facts.is_some_and(slots::has_dynamic_names) || spread.is_some();
    let alias = if block {
        Buf::create_block_alias()
    } else {
        Buf::create_vnode_alias()
    };
    cx.buf.push(alias);
    cx.buf.push("(");
    if let Some(helper) = builtin::helper(component.name) {
        cx.buf.use_helper(helper);
        cx.buf.push(helper.alias());
    } else {
        cx.buf
            .push(asset_ident("component", component.name).as_str());
    }
    let has_binds = component.bindings.iter().any(|binding| {
        !matches!(binding, BindingOp::SlotContent(_)) && !slots::is_slots_spread(binding)
    });
    let hoisted_static_props = if !array
        && (facts.is_some() || create)
        && !has_binds
        && if_key.is_none()
        && !component.attributes.is_empty()
    {
        Some(
            cx.buf
                .push_hoist(compact_props_object(component.attributes.iter())),
        )
    } else {
        None
    };
    let unused_hoist = hoisted_static_props.is_none()
        && !has_binds
        && if_key.is_none()
        && !component.attributes.is_empty()
        && builtin::has_static_nested(&component.children);
    if unused_hoist {
        cx.buf
            .push_hoist(compact_props_object(component.attributes.iter()));
    }
    let patch = bind_patch(&component.bindings, true);
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
    let emit_flag = flag != 0;
    if let Some(alias) = hoisted_static_props.as_ref() {
        cx.buf.push(", ");
        cx.buf.push(alias.as_str());
    } else if if_key.is_some() || has_binds || !component.attributes.is_empty() {
        cx.buf.push(", ");
        emit_bind_props(cx, &component.attributes, &component.bindings, if_key)?;
    } else if emit_flag || has_slots || has_array || for_item {
        // Vue's v-for item `createBlock` keeps an explicit null props
        // even when the component has no props, slots, or patch flag.
        cx.buf.push(", null");
    }
    if array {
        if has_array {
            cx.buf.push(", ");
            builtin::emit_array_children(cx, &component.children, if_key.is_some())?;
        } else if emit_flag {
            cx.buf.push(", null");
        }
    } else if create {
        cx.buf.push(", ");
        create_slots::emit_create_slots(cx, &component.children)?;
    } else if let Some(facts) = facts {
        cx.buf.push(", ");
        slots::emit_slots(cx, &component.children, facts, spread)?;
    } else if let Some(spread) = spread {
        cx.buf.push(", ");
        cx.buf.push(spread);
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

fn admit(component: &ComponentOp<'_>) -> Result<(), EmitError> {
    if component.name == "component" {
        return Err(EmitError::Unsupported);
    }
    if create_slots::needs_create_slots(&component.children)
        || slots::has_implicit_default(&component.children)
    {
        slots::admit_default(&component.children)?;
    }
    admit_bindings(&component.attributes, &component.bindings)
}

fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "Teleport"
            | "teleport"
            | "Suspense"
            | "suspense"
            | "KeepAlive"
            | "keep-alive"
            | "Transition"
            | "transition"
            | "TransitionGroup"
            | "transition-group"
            | "BaseTransition"
            | "component"
    )
}
