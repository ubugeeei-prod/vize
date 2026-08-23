//! Static native HTML element / children emission.

use alloc::vec::Vec as StdVec;

use vize_carton::{String, ensure_sufficient_stack};
use vize_disegno::op::{Attribute, ElementOp, InterpolationOp, Namespace, Op, Region, TextOp};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::children::{children_need_text_flag, emit_interpolation, emit_text_like};
use super::js::{escape_js_string, is_valid_js_identifier};

pub(super) fn emit_root(cx: &mut EmitCx<'_>, root: &Region<'_>) -> Result<(), EmitError> {
    let mut element = None;
    let mut interpolation: Option<(&InterpolationOp<'_>, _)> = None;
    for op in root.ops.iter() {
        let id = cx.walk.mint();
        match op {
            Op::Text(text) if is_ignorable_root_text(text) => {}
            Op::Element(found) if element.is_none() && interpolation.is_none() => {
                cx.walk.skip(found.bindings.len());
                element = Some(&**found);
            }
            Op::Interpolation(found) if element.is_none() && interpolation.is_none() => {
                interpolation = Some((found, id));
            }
            _ => return Err(EmitError::Unsupported),
        }
    }
    if let Some(element) = element {
        cx.buf.use_open_block();
        cx.buf.use_create_element_block();
        cx.buf.push("(");
        cx.buf.push(Buf::open_block_alias());
        cx.buf.push("(), ");
        emit_call(cx, element, /* block */ true)?;
        cx.buf.push(")");
        return Ok(());
    }
    if let Some((interp, id)) = interpolation {
        return emit_interpolation(cx, interp, id);
    }
    Err(EmitError::Unsupported)
}

fn is_ignorable_root_text(text: &TextOp<'_>) -> bool {
    text.content.chars().all(char::is_whitespace)
}

fn emit_nested(cx: &mut EmitCx<'_>, element: &ElementOp<'_>) -> Result<(), EmitError> {
    cx.buf.use_create_element_vnode();
    emit_call(cx, element, /* block */ false)
}

fn emit_call(cx: &mut EmitCx<'_>, element: &ElementOp<'_>, block: bool) -> Result<(), EmitError> {
    admit_static_native(element)?;
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
    let hoist = block && root_props_should_hoist(element);
    // Root (block) elements with a fully hoistable static props surface
    // match the shipped `is_root` + `has_static_props` arm: hoist the
    // object, do not whole-hoist the vnode. Nested native children keep
    // inline props (`hoist_static_vnodes` is false without directives).
    // When that hoist lands and the only patch flag is TEXT, the shipped
    // block emitter omits the flag.
    if hoist {
        cx.buf
            .hoist_root_props(compact_props_object(element.attributes.iter()));
        cx.buf.push(", ");
        cx.buf.push(Buf::hoisted_props_alias());
        if has_children {
            cx.buf.push(", ");
            emit_children(cx, &element.children)?;
        }
    } else if !element.attributes.is_empty() {
        cx.buf.push(", ");
        emit_static_props_inline(cx, element.attributes.iter());
        if has_children {
            cx.buf.push(", ");
            emit_children(cx, &element.children)?;
        }
    } else if has_children {
        cx.buf.push(", null, ");
        emit_children(cx, &element.children)?;
    }
    if children_need_text_flag(&element.children) && !hoist {
        cx.buf.push(", 1 /* TEXT */");
    }
    cx.buf.push(")");
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

fn emit_children(cx: &mut EmitCx<'_>, children: &Region<'_>) -> Result<(), EmitError> {
    let ops = &children.ops;
    if ops
        .iter()
        .all(|op| matches!(op, Op::Text(_) | Op::Interpolation(_)))
    {
        return emit_text_like(cx, ops);
    }
    cx.buf.push("[");
    cx.buf.indent();
    for (i, op) in ops.iter().enumerate() {
        if i > 0 {
            cx.buf.push(",");
        }
        cx.buf.newline();
        emit_array_child(cx, op)?;
    }
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("]");
    Ok(())
}

fn emit_array_child(cx: &mut EmitCx<'_>, op: &Op<'_>) -> Result<(), EmitError> {
    let _id = cx.walk.mint();
    ensure_sufficient_stack(|| match op {
        Op::Element(element) => {
            cx.walk.skip(element.bindings.len());
            emit_nested(cx, element)
        }
        Op::Text(_)
        | Op::Component(_)
        | Op::Interpolation(_)
        | Op::If(_)
        | Op::For(_)
        | Op::Slot(_) => Err(EmitError::Unsupported),
    })
}
