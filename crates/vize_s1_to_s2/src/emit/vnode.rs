//! Static native HTML element / children emission.

use vize_s0::ensure_sufficient_stack;
use vize_s2::op::{BindingOp, ElementOp, Op, Region};

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::children::{children_need_text_flag, emit_create_text_vnode, emit_text_like};
use super::directive;
use super::flag::emit_patch_flag;
use super::hoist::compact_props_object;
use super::namespace;
use super::props::{admit_element_bindings, apply_static_ref_patch, bind_patch, emit_bind_props};

pub(super) fn emit_unique_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
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
            emit_call(cx, element, true, None, true, false, false)?;
            cx.buf.push(")");
            Ok(())
        })
    })
}

fn emit_block(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    allow_hoist: bool,
    if_key: Option<&str>,
    for_item: bool,
) -> Result<(), EmitError> {
    directive::wrap_element(cx, element, |cx| {
        cx.buf.use_open_block();
        cx.buf.use_create_element_block();
        cx.buf.push("(");
        cx.buf.push(Buf::open_block_alias());
        cx.buf.push("(), ");
        emit_call(cx, element, true, if_key, allow_hoist, for_item, false)?;
        cx.buf.push(")");
        Ok(())
    })
}

pub(super) fn emit_fragment_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
) -> Result<(), EmitError> {
    if super::once::has(&element.bindings) {
        return super::once::emit_element(cx, element, None, false);
    }
    if namespace::crosses_boundary(cx, element) {
        return emit_nested_block(cx, element);
    }
    super::memo::emit_cached(cx, &element.bindings, |cx| {
        directive::wrap_element(cx, element, |cx| {
            cx.buf.use_create_element_vnode();
            emit_call(cx, element, false, None, true, false, false)
        })
    })
}

fn emit_nested(cx: &mut EmitCx<'_>, element: &ElementOp<'_>) -> Result<(), EmitError> {
    if super::once::has(&element.bindings) {
        return super::once::emit_element(cx, element, None, false);
    }
    if namespace::crosses_boundary(cx, element) {
        return emit_nested_block(cx, element);
    }
    super::memo::emit_cached(cx, &element.bindings, |cx| {
        directive::wrap_element(cx, element, |cx| {
            cx.buf.use_create_element_vnode();
            emit_call(cx, element, false, None, false, false, false)
        })
    })
}

fn emit_nested_block(cx: &mut EmitCx<'_>, element: &ElementOp<'_>) -> Result<(), EmitError> {
    super::memo::emit_cached(cx, &element.bindings, |cx| {
        emit_block(cx, element, false, None, false)
    })
}

pub(super) fn emit_if_branch_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    key: &str,
) -> Result<(), EmitError> {
    emit_block(cx, element, false, Some(key), false)
}

pub(super) fn emit_for_item_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    stable: bool,
    key: Option<&str>,
) -> Result<(), EmitError> {
    if super::once::has(&element.bindings) {
        return super::once::emit_element(cx, element, key, true);
    }
    super::memo::emit_cached(cx, &element.bindings, |cx| {
        if stable {
            return directive::wrap_element(cx, element, |cx| {
                cx.buf.use_create_element_vnode();
                emit_call(cx, element, false, key, false, true, false)
            });
        }
        emit_block(cx, element, false, key, true)
    })
}

pub(super) fn emit_call(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    block: bool,
    if_key: Option<&str>,
    allow_hoist: bool,
    for_item: bool,
    once: bool,
) -> Result<(), EmitError> {
    admit_native(element)?;
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
    let has_runtime_child_hoist = allow_hoist
        && block
        && (directive::has_runtime(&element.bindings)
            || super::model::first_runtime_model(element).is_some());
    let has_memo = super::memo::has(&element.bindings);
    let memo_block = block && has_memo && !(if_key.is_some() && !for_item);
    let force_array_children =
        once || memo_block || (directive::has_custom(&element.bindings) && allow_hoist && block);
    let has_binds = has_prop_bindings(&element.bindings);
    let hoist = allow_hoist && if_key.is_none() && super::props_static::root_should_hoist(element);
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
            .hoist_root_props(compact_props_object(element.attributes.iter()));
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
                has_runtime_child_hoist,
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

fn admit_native(element: &ElementOp<'_>) -> Result<(), EmitError> {
    admit_element_bindings(&element.attributes, &element.bindings)
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

fn emit_children(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
    force_array: bool,
    hoist_static_children: bool,
) -> Result<(), EmitError> {
    let ops = &children.ops;
    if !force_array
        && ops
            .iter()
            .all(|op| matches!(op, Op::Text(_) | Op::Interpolation(_)))
    {
        return emit_text_like(cx, ops);
    }
    cx.buf.push("[");
    cx.buf.indent();
    let mut i = 0;
    let mut first = true;
    while i < ops.len() {
        if matches!(ops[i], Op::Text(_) | Op::Interpolation(_)) {
            let start = i;
            while i < ops.len() && matches!(ops[i], Op::Text(_) | Op::Interpolation(_)) {
                i += 1;
            }
            if !first {
                cx.buf.push(",");
            }
            cx.buf.newline();
            first = false;
            emit_create_text_vnode(cx, &ops[start..i])?;
            continue;
        }
        if !first {
            cx.buf.push(",");
        }
        cx.buf.newline();
        first = false;
        emit_array_child(cx, &ops[i], hoist_static_children)?;
        i += 1;
    }
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("]");
    Ok(())
}

pub(super) fn emit_array_child(
    cx: &mut EmitCx<'_>,
    op: &Op<'_>,
    hoist_static_children: bool,
) -> Result<(), EmitError> {
    if hoist_static_children
        && let Op::Element(element) = op
        && super::hoist::is_hoistable(element)
    {
        return super::hoist::emit_hoisted_element(cx, element);
    }
    let id = cx.walk.mint();
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
            emit_nested(cx, element)
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
}
