//! Static native HTML element / children emission.

use vize_davinci::id::NodeId;
use vize_s0::ensure_sufficient_stack;
use vize_s2::op::{BindingOp, ElementOp, Op};

use super::buf::Buf;
use super::children::children_need_text_flag;
use super::directive;
use super::flag::emit_patch_flag;
use super::namespace;
use super::props::{admit_element_bindings, apply_static_ref_patch, bind_patch, emit_bind_props};
use super::props_static::PropHoistPosition;
use super::vnode_children::emit_children;
use super::{EmitCx, EmitError, UnsupportedReason as Reason};
use crate::pass::StaticLevel;

pub(super) fn emit_unique_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    if super::once::has(&element.bindings) {
        return super::once::emit_element(cx, element, None, false);
    }
    super::memo::emit_cached(cx, &element.bindings, |cx| {
        directive::wrap_element(cx, element, |cx| {
            cx.buf.use_open_block();
            cx.buf.use_create_element_block();
            cx.buf.push("(");
            cx.buf.push(Buf::open_block_alias());
            cx.buf.push("(), ");
            emit_call(
                cx,
                element,
                true,
                None,
                (true, id, PropHoistPosition::Root),
                false,
                false,
            )?;
            cx.buf.push(")");
            Ok(())
        })
    })
}

fn emit_block(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    if_key: Option<&str>,
    for_item: bool,
    hoist: (bool, Option<NodeId>, PropHoistPosition),
) -> Result<(), EmitError> {
    directive::wrap_element(cx, element, |cx| {
        cx.buf.use_open_block();
        cx.buf.use_create_element_block();
        cx.buf.push("(");
        cx.buf.push(Buf::open_block_alias());
        cx.buf.push("(), ");
        emit_call(cx, element, true, if_key, hoist, for_item, false)?;
        cx.buf.push(")");
        Ok(())
    })
}

pub(super) fn emit_fragment_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    if super::once::has(&element.bindings) {
        return super::once::emit_element(cx, element, None, false);
    }
    if namespace::crosses_boundary(cx, element) {
        return emit_nested_block(cx, element, id);
    }
    super::memo::emit_cached(cx, &element.bindings, |cx| {
        directive::wrap_element(cx, element, |cx| {
            cx.buf.use_create_element_vnode();
            emit_call(
                cx,
                element,
                false,
                None,
                (true, id, PropHoistPosition::Root),
                false,
                false,
            )
        })
    })
}

fn emit_nested(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    if super::once::has(&element.bindings) {
        return super::once::emit_element(cx, element, None, false);
    }
    if namespace::crosses_boundary(cx, element) {
        return emit_nested_block(cx, element, id);
    }
    super::memo::emit_cached(cx, &element.bindings, |cx| {
        directive::wrap_element(cx, element, |cx| {
            cx.buf.use_create_element_vnode();
            emit_call(
                cx,
                element,
                false,
                None,
                (true, id, PropHoistPosition::Nested),
                false,
                false,
            )
        })
    })
}

fn emit_nested_block(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    super::memo::emit_cached(cx, &element.bindings, |cx| {
        emit_block(
            cx,
            element,
            None,
            false,
            (true, id, PropHoistPosition::Nested),
        )
    })
}

pub(super) fn emit_if_branch_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    key: &str,
) -> Result<(), EmitError> {
    emit_block(
        cx,
        element,
        Some(key),
        false,
        (true, None, PropHoistPosition::Nested),
    )
}

pub(super) fn emit_call(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    block: bool,
    if_key: Option<&str>,
    hoist: (bool, Option<NodeId>, PropHoistPosition),
    for_item: bool,
    once: bool,
) -> Result<(), EmitError> {
    admit_element_bindings(&element.attributes, &element.bindings)?;
    let (allow_hoist, id, prop_hoist) = hoist;
    let alias = if block {
        Buf::create_element_block_alias()
    } else {
        Buf::create_element_vnode_alias()
    };
    cx.buf.push(alias);
    cx.buf.push("(\"");
    cx.buf.push(element.tag);
    cx.buf.push("\"");
    let has_children = !element.children.ops.is_empty();
    let hoist_static_children = should_hoist_static_children(
        cx,
        element,
        id,
        allow_hoist,
        if_key.is_some() && !for_item,
        for_item,
    );
    let has_memo = super::memo::has(&element.bindings);
    let memo_block = block && has_memo && !(if_key.is_some() && !for_item);
    let force_array_children = once
        || memo_block
        || (directive::has_custom(&element.bindings) && allow_hoist && block && if_key.is_none());
    let cache_static_children = allow_hoist
        && cx.static_cache
        && !hoist_static_children
        && !force_array_children
        && !for_item
        && !cx.in_v_for
        && cx.slot_param_depth == 0;
    let has_binds = has_prop_bindings(&element.bindings);
    let hoisted_props =
        if allow_hoist && if_key.is_none() && super::props_static::should_hoist(cx, id, prop_hoist)
        {
            super::props_static::root_hoist_props(&element.attributes, &element.bindings)?
        } else {
            None
        };
    let hoist = hoisted_props.is_some();
    let patch = bind_patch(&element.bindings, false, if_key, for_item);
    let text_flag = !once && !memo_block && children_need_text_flag(&element.children);
    let mut flag = patch.flag;
    if text_flag {
        flag |= 1;
    }
    apply_static_ref_patch(&element.attributes, &mut flag);
    if for_item {
        flag &= !512;
    }
    if once {
        flag &= 2 | 4;
    }
    if memo_block {
        flag = 0;
    }
    let omit_text_only = hoist && block && flag == 1;
    let emit_flag = flag != 0 && !omit_text_only;
    let empty_runtime_for = for_item
        && (directive::has_runtime(&element.bindings) || has_cloak(&element.bindings))
        && !has_binds
        && element.attributes.is_empty()
        && if_key.is_none();
    if hoist {
        let props_alias = cx
            .buf
            .hoist_root_props(hoisted_props.expect("checked hoisted props"));
        cx.buf.push(", ");
        cx.buf.push(props_alias.as_str());
    } else if if_key.is_some() || has_binds {
        cx.buf.push(", ");
        emit_bind_props(
            cx,
            &element.attributes,
            &element.bindings,
            if_key,
            false,
            for_item,
            true,
        )?;
    } else if !element.attributes.is_empty() {
        cx.buf.push(", ");
        super::props_static::emit_inline(cx, element.attributes.iter());
    } else if empty_runtime_for {
        cx.buf.push(", { }");
    } else if has_children || emit_flag {
        cx.buf.push(", null");
    }
    if has_children {
        cx.buf.push(", ");
        namespace::with_child(cx, element, |cx| {
            emit_children(
                cx,
                &element.children,
                force_array_children,
                hoist_static_children,
                cache_static_children,
            )
        })?;
    } else if emit_flag {
        cx.buf.push(", null");
    }
    if emit_flag {
        emit_patch_flag(cx, flag);
    }
    let suppress_memo_for_item_dynamic_props = memo_block && cx.skip_memo && for_item;
    if !once && !suppress_memo_for_item_dynamic_props && !patch.dynamic_props.is_empty() {
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

fn should_hoist_static_children(
    cx: &EmitCx<'_>,
    element: &ElementOp<'_>,
    id: Option<NodeId>,
    allow_hoist: bool,
    branch_root: bool,
    for_item: bool,
) -> bool {
    let requested =
        cx.hoist_static_vnodes || (allow_hoist && (branch_root || !element.bindings.is_empty()));
    if !requested {
        return false;
    }
    if branch_root || for_item {
        return true;
    }
    id.and_then(|id| cx.facts.static_facts.get(id))
        .is_some_and(|fact| fact.level == StaticLevel::NotStatic)
}

fn has_prop_bindings(bindings: &[BindingOp<'_>]) -> bool {
    bindings.iter().any(|binding| {
        matches!(
            binding,
            BindingOp::Bind(_)
                | BindingOp::On(_)
                | BindingOp::Model(_)
                | BindingOp::VueHtml(_)
                | BindingOp::VueText(_)
        )
    })
}

fn has_cloak(bindings: &[BindingOp<'_>]) -> bool {
    bindings
        .iter()
        .any(|binding| matches!(binding, BindingOp::VueCloak(_)))
}

pub(super) fn emit_array_child(
    cx: &mut EmitCx<'_>,
    op: &Op<'_>,
    hoist_static_children: bool,
    cache_static_children: bool,
) -> Result<(), EmitError> {
    let hoist_static_children = hoist_static_children || cx.hoist_static_vnodes;
    if hoist_static_children
        && let Op::Element(element) = op
        && super::hoist::is_hoistable(element)
    {
        return super::hoist::emit_hoisted_element(cx, element);
    }
    if cache_static_children
        && let Op::Element(element) = op
        && super::hoist::is_hoistable(element)
    {
        return super::hoist::emit_cached_element(cx, element);
    }
    let id = cx.walk.mint();
    cx.with_static_vnode_hoist(hoist_static_children, |cx| {
        ensure_sufficient_stack(|| match op {
            Op::Element(element) if super::slots::is_slot_template(element) => {
                cx.walk.skip(element.bindings.len());
                super::tpl::emit_inline(cx, &element.children.ops)
            }
            Op::Element(element) => {
                cx.walk.skip(element.bindings.len());
                if super::once::emit_hoisted_child(cx, element)? {
                    return Ok(());
                }
                emit_nested(cx, element, id)
            }
            Op::Component(component) => {
                cx.walk.skip(component.bindings.len());
                super::component::emit_nested(cx, component, id)
            }
            Op::If(if_op) => super::emit_if_op(cx, if_op, id),
            Op::For(for_op) => super::emit_for_op(cx, for_op, id, None),
            Op::Slot(slot) => {
                cx.walk.skip(slot.bindings.len());
                super::outlet::emit_outlet(cx, slot, None, false)
            }
            Op::Text(_) | Op::Interpolation(_) => {
                Err(EmitError::unsupported_op(Reason::ArrayChildTextRun, op))
            }
        })
    })
}
