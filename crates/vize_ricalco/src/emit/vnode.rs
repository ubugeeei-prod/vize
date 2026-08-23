//! Static native HTML element / children emission.

use alloc::vec::Vec as StdVec;

use vize_carton::{String, ensure_sufficient_stack};
use vize_disegno::op::{Attribute, ElementOp, Namespace, Op, Region, TextOp};

use super::EmitError;
use super::buf::Buf;
use super::js::{escape_js_string, is_valid_js_identifier};

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
    let has_children = !element.children.ops.is_empty();
    // Root (block) elements with a fully hoistable static props surface
    // match the shipped `is_root` + `has_static_props` arm: hoist the
    // object, do not whole-hoist the vnode. Nested native children keep
    // inline props (`hoist_static_vnodes` is false without directives).
    if block && root_props_should_hoist(element) {
        buf.hoist_root_props(compact_props_object(element.attributes.iter()));
        buf.push(", ");
        buf.push(Buf::hoisted_props_alias());
        if has_children {
            buf.push(", ");
            emit_children(buf, &element.children)?;
        }
    } else if !element.attributes.is_empty() {
        buf.push(", ");
        emit_static_props_inline(buf, element.attributes.iter());
        if has_children {
            buf.push(", ");
            emit_children(buf, &element.children)?;
        }
    } else if has_children {
        buf.push(", null, ");
        emit_children(buf, &element.children)?;
    }
    buf.push(")");
    Ok(())
}

fn root_props_should_hoist(element: &ElementOp<'_>) -> bool {
    !element.attributes.is_empty()
        && element
            .attributes
            .iter()
            .all(|attribute| attribute.name != "ref")
}

/// First-occurrence static attrs as a single-line object, matching
/// hoisted `JsChildNode::Object` emission.
fn compact_props_object<'a>(attributes: impl Iterator<Item = &'a Attribute<'a>>) -> String {
    let unique = unique_attrs(attributes);
    let mut out = String::from("{ ");
    for (i, attr) in unique.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        push_attr_pair(&mut out, attr);
    }
    out.push_str(" }");
    out
}

/// First-occurrence static attrs, matching
/// `vize_atelier_core::codegen::props::generate::try_generate_static_attrs`.
fn emit_static_props_inline<'a>(
    buf: &mut Buf,
    attributes: impl Iterator<Item = &'a Attribute<'a>>,
) {
    let unique = unique_attrs(attributes);
    let multiline = unique.len() > 1;
    if multiline {
        buf.push("{");
        buf.indent();
    } else {
        buf.push("{ ");
    }
    for (i, attr) in unique.iter().enumerate() {
        if i > 0 {
            buf.push(",");
        }
        if multiline {
            buf.newline();
        } else if i > 0 {
            buf.push(" ");
        }
        let mut pair = String::default();
        push_attr_pair(&mut pair, attr);
        buf.push(pair.as_str());
    }
    if multiline {
        buf.deindent();
        buf.newline();
        buf.push("}");
    } else {
        buf.push(" }");
    }
}

fn unique_attrs<'a>(
    attributes: impl Iterator<Item = &'a Attribute<'a>>,
) -> StdVec<&'a Attribute<'a>> {
    let mut unique: StdVec<&Attribute<'_>> = StdVec::new();
    for attr in attributes {
        if unique.iter().any(|seen| seen.name == attr.name) {
            continue;
        }
        unique.push(attr);
    }
    unique
}

fn push_attr_pair(out: &mut String, attr: &Attribute<'_>) {
    let quoted = !is_valid_js_identifier(attr.name);
    if quoted {
        out.push('"');
    }
    out.push_str(attr.name);
    if quoted {
        out.push('"');
    }
    out.push_str(": \"");
    if let Some(value) = attr.value {
        out.push_str(escape_js_string(value).as_str());
    }
    out.push('"');
}

fn admit_static_native(element: &ElementOp<'_>) -> Result<(), EmitError> {
    if element.namespace != Namespace::Html {
        return Err(EmitError::Unsupported);
    }
    if !element.bindings.is_empty() {
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
