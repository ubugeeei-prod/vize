//! Static native HTML element / children emission.

use vize_carton::ensure_sufficient_stack;
use vize_disegno::op::{ElementOp, Namespace, Op, Region, TextOp};

use super::EmitError;
use super::buf::Buf;
use super::js::escape_js_string;

pub(super) fn emit_root(buf: &mut Buf, root: &Region<'_>) -> Result<(), EmitError> {
    let element = unique_root_element(root)?;
    buf.use_open_block();
    buf.use_create_element_block();
    buf.push("(");
    buf.push(Buf::open_block_alias());
    buf.push("(), ");
    emit_call(buf, element, /* block */ true)?;
    buf.push(")");
    Ok(())
}

fn unique_root_element<'a>(root: &'a Region<'a>) -> Result<&'a ElementOp<'a>, EmitError> {
    let mut found = None;
    for op in root.ops.iter() {
        match op {
            Op::Text(text) if is_ignorable_root_text(text) => {}
            Op::Element(element) if found.is_none() => found = Some(&**element),
            _ => return Err(EmitError::Unsupported),
        }
    }
    found.ok_or(EmitError::Unsupported)
}

fn is_ignorable_root_text(text: &TextOp<'_>) -> bool {
    text.content.chars().all(char::is_whitespace)
}

fn emit_nested(buf: &mut Buf, element: &ElementOp<'_>) -> Result<(), EmitError> {
    buf.use_create_element_vnode();
    emit_call(buf, element, /* block */ false)
}

fn emit_call(buf: &mut Buf, element: &ElementOp<'_>, block: bool) -> Result<(), EmitError> {
    admit_static_native(element)?;
    let alias = if block {
        Buf::create_element_block_alias()
    } else {
        Buf::create_element_vnode_alias()
    };
    buf.push(alias);
    buf.push("(\"");
    buf.push(element.tag);
    buf.push("\"");
    if !element.children.ops.is_empty() {
        buf.push(", null, ");
        emit_children(buf, &element.children)?;
    }
    buf.push(")");
    Ok(())
}

fn admit_static_native(element: &ElementOp<'_>) -> Result<(), EmitError> {
    if element.namespace != Namespace::Html {
        return Err(EmitError::Unsupported);
    }
    if !element.attributes.is_empty() || !element.bindings.is_empty() {
        return Err(EmitError::Unsupported);
    }
    Ok(())
}

fn emit_children(buf: &mut Buf, children: &Region<'_>) -> Result<(), EmitError> {
    let ops = &children.ops;
    if ops.len() == 1
        && let Op::Text(text) = &ops[0]
    {
        buf.push("\"");
        buf.push(escape_js_string(text.content).as_str());
        buf.push("\"");
        return Ok(());
    }
    buf.push("[");
    buf.indent();
    for (i, op) in ops.iter().enumerate() {
        if i > 0 {
            buf.push(",");
        }
        buf.newline();
        emit_array_child(buf, op)?;
    }
    buf.deindent();
    buf.newline();
    buf.push("]");
    Ok(())
}

fn emit_array_child(buf: &mut Buf, op: &Op<'_>) -> Result<(), EmitError> {
    ensure_sufficient_stack(|| match op {
        Op::Element(element) => emit_nested(buf, element),
        Op::Text(_)
        | Op::Component(_)
        | Op::Interpolation(_)
        | Op::If(_)
        | Op::For(_)
        | Op::Slot(_) => Err(EmitError::Unsupported),
    })
}
