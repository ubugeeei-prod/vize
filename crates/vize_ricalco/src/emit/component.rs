//! Static-name component emission (`resolveComponent` / `createVNode` /
//! `createBlock`). Slot children, builtins, and `<component :is>` stay
//! unsupported this installment.

use alloc::vec::Vec as StdVec;

use vize_disegno::op::{ComponentOp, Op, Region, TextOp};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::flag::emit_patch_flag;
use super::js::asset_ident;
use super::props::{admit_bindings, bind_patch, emit_bind_props};

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

pub(super) fn emit_root(cx: &mut EmitCx<'_>, component: &ComponentOp<'_>) -> Result<(), EmitError> {
    cx.buf.use_open_block();
    cx.buf.use_create_block();
    cx.buf.push("(");
    cx.buf.push(Buf::open_block_alias());
    cx.buf.push("(), ");
    emit_call(cx, component, /* block */ true, None)?;
    cx.buf.push(")");
    Ok(())
}

pub(super) fn emit_nested(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
) -> Result<(), EmitError> {
    cx.buf.use_create_vnode();
    emit_call(cx, component, /* block */ false, None)
}

pub(super) fn emit_if_branch(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    key: &str,
) -> Result<(), EmitError> {
    cx.buf.use_open_block();
    cx.buf.use_create_block();
    cx.buf.push("(");
    cx.buf.push(Buf::open_block_alias());
    cx.buf.push("(), ");
    emit_call(cx, component, /* block */ true, Some(key))?;
    cx.buf.push(")");
    Ok(())
}

pub(super) fn emit_for_item(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
) -> Result<(), EmitError> {
    cx.buf.use_open_block();
    cx.buf.use_create_block();
    cx.buf.push("(");
    cx.buf.push(Buf::open_block_alias());
    cx.buf.push("(), ");
    emit_call(cx, component, /* block */ true, None)?;
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
) -> Result<(), EmitError> {
    admit(component)?;
    let alias = if block {
        Buf::create_block_alias()
    } else {
        Buf::create_vnode_alias()
    };
    cx.buf.push(alias);
    cx.buf.push("(");
    cx.buf
        .push(asset_ident("component", component.name).as_str());
    let has_binds = !component.bindings.is_empty();
    let patch = bind_patch(&component.bindings, true);
    let emit_flag = patch.flag != 0;
    let has_props = !component.attributes.is_empty() || has_binds || if_key.is_some();
    if if_key.is_some() || has_binds || !component.attributes.is_empty() {
        cx.buf.push(", ");
        emit_bind_props(cx, &component.attributes, &component.bindings, if_key)?;
    } else if emit_flag {
        cx.buf.push(", null");
    }
    if emit_flag && has_props {
        cx.buf.push(", null");
    }
    if emit_flag {
        emit_patch_flag(cx, patch.flag);
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
    if is_builtin(component.name) {
        return Err(EmitError::Unsupported);
    }
    admit_empty(&component.children)?;
    admit_bindings(&component.attributes, &component.bindings)
}

fn admit_empty(children: &Region<'_>) -> Result<(), EmitError> {
    if children.ops.iter().all(is_ignorable_child) {
        Ok(())
    } else {
        Err(EmitError::Unsupported)
    }
}

fn is_ignorable_child(op: &Op<'_>) -> bool {
    matches!(op, Op::Text(text) if is_whitespace_text(text))
}

fn is_whitespace_text(text: &TextOp<'_>) -> bool {
    text.content.chars().all(char::is_whitespace)
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
