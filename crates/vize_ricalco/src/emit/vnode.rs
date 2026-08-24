//! Static native HTML element / children emission.

use vize_carton::{String, ensure_sufficient_stack};
use vize_disegno::op::{Attribute, BindingOp, ElementOp, Op, Region};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::children::{children_need_text_flag, emit_create_text_vnode, emit_text_like};
use super::directive;
use super::flag::emit_patch_flag;
use super::hoist::{compact_props_object, push_attr_pair, unique_attrs};
use super::namespace;
use super::props::{admit_bindings, bind_patch, emit_bind_props};

pub(super) fn emit_unique_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
) -> Result<(), EmitError> {
    directive::wrap_element(cx, element, |cx| {
        cx.buf.use_open_block();
        cx.buf.use_create_element_block();
        cx.buf.push("(");
        cx.buf.push(Buf::open_block_alias());
        cx.buf.push("(), ");
        emit_call(
            cx, element, /* block */ true, None, /* hoist */ true,
            /* for_item */ false,
        )?;
        cx.buf.push(")");
        Ok(())
    })
}

pub(super) fn emit_fragment_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
) -> Result<(), EmitError> {
    if namespace::crosses_boundary(cx, element) {
        return emit_nested_block(cx, element);
    }
    directive::wrap_element(cx, element, |cx| {
        cx.buf.use_create_element_vnode();
        emit_call(
            cx, element, /* block */ false, None, /* hoist */ true,
            /* for_item */ false,
        )
    })
}

fn emit_nested(cx: &mut EmitCx<'_>, element: &ElementOp<'_>) -> Result<(), EmitError> {
    if namespace::crosses_boundary(cx, element) {
        return emit_nested_block(cx, element);
    }
    directive::wrap_element(cx, element, |cx| {
        cx.buf.use_create_element_vnode();
        emit_call(
            cx, element, /* block */ false, None, /* hoist */ false,
            /* for_item */ false,
        )
    })
}

fn emit_nested_block(cx: &mut EmitCx<'_>, element: &ElementOp<'_>) -> Result<(), EmitError> {
    directive::wrap_element(cx, element, |cx| {
        cx.buf.use_open_block();
        cx.buf.use_create_element_block();
        cx.buf.push("(");
        cx.buf.push(Buf::open_block_alias());
        cx.buf.push("(), ");
        emit_call(
            cx, element, /* block */ true, None, /* hoist */ false,
            /* for_item */ false,
        )?;
        cx.buf.push(")");
        Ok(())
    })
}

pub(super) fn emit_if_branch_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    key: &str,
) -> Result<(), EmitError> {
    directive::wrap_element(cx, element, |cx| {
        cx.buf.use_open_block();
        cx.buf.use_create_element_block();
        cx.buf.push("(");
        cx.buf.push(Buf::open_block_alias());
        cx.buf.push("(), ");
        emit_call(
            cx,
            element,
            /* block */ true,
            Some(key),
            /* hoist */ false,
            /* for_item */ false,
        )?;
        cx.buf.push(")");
        Ok(())
    })
}

pub(super) fn emit_for_item_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    stable: bool,
    key: Option<&str>,
) -> Result<(), EmitError> {
    directive::wrap_element(cx, element, |cx| {
        if stable {
            cx.buf.use_create_element_vnode();
            return emit_call(
                cx, element, /* block */ false, key, /* hoist */ false,
                /* for_item */ true,
            );
        }
        cx.buf.use_open_block();
        cx.buf.use_create_element_block();
        cx.buf.push("(");
        cx.buf.push(Buf::open_block_alias());
        cx.buf.push("(), ");
        emit_call(
            cx, element, /* block */ true, key, /* hoist */ false,
            /* for_item */ true,
        )?;
        cx.buf.push(")");
        Ok(())
    })
}

fn emit_call(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    block: bool,
    if_key: Option<&str>,
    allow_hoist: bool,
    for_item: bool,
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
    let has_custom = directive::has_custom(&element.bindings);
    let has_binds = has_prop_bindings(&element.bindings);
    let hoist = allow_hoist && if_key.is_none() && root_props_should_hoist(element);
    let patch = bind_patch(&element.bindings, false);
    let text_flag = children_need_text_flag(&element.children);
    let mut flag = patch.flag;
    if text_flag {
        flag |= 1;
    }
    if for_item {
        flag &= !512;
    }
    let omit_text_only = hoist && block && flag == 1;
    let emit_flag = flag != 0 && !omit_text_only;
    let empty_custom_for =
        for_item && has_custom && !has_binds && element.attributes.is_empty() && if_key.is_none();
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
        emit_static_props_inline(cx, element.attributes.iter());
    } else if empty_custom_for {
        cx.buf.push(", { }");
    } else if has_children || emit_flag {
        cx.buf.push(", null");
    }
    if has_children {
        cx.buf.push(", ");
        namespace::with_child(cx, element, |cx| {
            emit_children(cx, &element.children, has_custom && allow_hoist && block)
        })?;
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

fn root_props_should_hoist(element: &ElementOp<'_>) -> bool {
    element.bindings.is_empty()
        && !element.attributes.is_empty()
        && element
            .attributes
            .iter()
            .all(|attribute| attribute.name != "ref")
}

fn emit_static_props_inline<'a>(
    cx: &mut EmitCx<'_>,
    attributes: impl Iterator<Item = &'a Attribute<'a>>,
) {
    let unique = unique_attrs(attributes);
    let multiline = unique.len() > 1;
    if multiline {
        cx.buf.push("{");
        cx.buf.indent();
    } else {
        cx.buf.push("{ ");
    }
    for (i, attr) in unique.iter().enumerate() {
        if i > 0 {
            cx.buf.push(",");
        }
        if multiline {
            cx.buf.newline();
        } else if i > 0 {
            cx.buf.push(" ");
        }
        let mut pair = String::default();
        push_attr_pair(&mut pair, attr);
        cx.buf.push(pair.as_str());
    }
    if multiline {
        cx.buf.deindent();
        cx.buf.newline();
        cx.buf.push("}");
    } else {
        cx.buf.push(" }");
    }
}

fn admit_native(element: &ElementOp<'_>) -> Result<(), EmitError> {
    admit_bindings(&element.attributes, &element.bindings)
}

fn has_prop_bindings(bindings: &[BindingOp<'_>]) -> bool {
    bindings.iter().any(|binding| {
        matches!(
            binding,
            BindingOp::Bind(_) | BindingOp::On(_) | BindingOp::Model(_)
        )
    })
}

fn emit_children(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
    force_array: bool,
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
        emit_array_child(cx, &ops[i])?;
        i += 1;
    }
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("]");
    Ok(())
}

pub(super) fn emit_array_child(cx: &mut EmitCx<'_>, op: &Op<'_>) -> Result<(), EmitError> {
    let id = cx.walk.mint();
    ensure_sufficient_stack(|| match op {
        Op::Element(element) => {
            cx.walk.skip(element.bindings.len());
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
        Op::Text(_) | Op::Interpolation(_) => Err(EmitError::Unsupported),
    })
}
