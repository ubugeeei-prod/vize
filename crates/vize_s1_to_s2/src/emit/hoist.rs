//! Static vnode hoist (`/*#__PURE__*/ _createElementVNode(...)`).
//!
//! Shared by implicit default-slot children and static `ui.for` items.
//! Named / scoped slots and nested hoist-out-of-dynamic-parents stay
//! with later installments.

mod cached_props;
mod kids;
mod props;

pub(super) use props::{compact_props_object, push_attr_pair, unique_attrs};

use vize_s0::{String, ToCompactString};
use vize_s2::op::{ElementOp, Op};

use super::buf::Buf;
use super::{EmitCx, EmitError, UnsupportedReason as Reason};
pub(super) use kids::push_spaces;
use kids::{append_cached_kids, append_hoist_kids, hoist_needs_create_text, renderable_children};

pub(super) fn emit_hoisted_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
) -> Result<(), super::EmitError> {
    let _id = cx.walk.mint();
    cx.walk.skip(element.bindings.len());
    let alias = hoist_static_element(cx, element);
    cx.buf.push(alias.as_str());
    Ok(())
}

pub(super) fn emit_cached_element(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
) -> Result<(), super::EmitError> {
    let _id = cx.walk.mint();
    cx.walk.skip(element.bindings.len());
    walk_hoisted(cx, element);
    cx.buf.use_create_element_vnode();
    if hoist_needs_create_text(element) {
        cx.buf.use_create_text();
    }
    let cache_index = cx.once_cache_index;
    cx.once_cache_index += 1;
    let cache_index = cache_index.to_compact_string();
    cx.buf.push("_cache[");
    cx.buf.push(cache_index.as_str());
    cx.buf.push("] || (_cache[");
    cx.buf.push(cache_index.as_str());
    cx.buf.push("] = ");
    cx.buf
        .push(cached_element_rhs(element, true, cx.buf.indent_width()).as_str());
    cx.buf.push(")");
    Ok(())
}

pub(super) fn cacheable_elements_array(ops: &[Op<'_>]) -> bool {
    !ops.is_empty()
        && ops.iter().all(|op| match op {
            Op::Element(element) => is_hoistable(element),
            _ => false,
        })
}

pub(super) fn emit_cached_elements_array(
    cx: &mut EmitCx<'_>,
    ops: &[Op<'_>],
) -> Result<(), EmitError> {
    let cache_index = cx.once_cache_index;
    cx.once_cache_index += 1;
    let cache_index = cache_index.to_compact_string();
    cx.buf.push("[...(_cache[");
    cx.buf.push(cache_index.as_str());
    cx.buf.push("] || (_cache[");
    cx.buf.push(cache_index.as_str());
    cx.buf.push("] = [");
    cx.buf.indent();

    for (i, op) in ops.iter().enumerate() {
        if i > 0 {
            cx.buf.push(",");
        }
        cx.buf.newline();
        let Op::Element(element) = op else {
            return Err(EmitError::unsupported_op(Reason::ArrayChildTextRun, op));
        };
        let _id = cx.walk.mint();
        cx.walk.skip(element.bindings.len());
        walk_hoisted(cx, element);
        cx.buf.use_create_element_vnode();
        if hoist_needs_create_text(element) {
            cx.buf.use_create_text();
        }
        cx.buf
            .push(cached_element_rhs(element, true, cx.buf.indent_width()).as_str());
    }

    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("]))]");
    Ok(())
}

pub(super) fn hoist_static_element(cx: &mut EmitCx<'_>, element: &ElementOp<'_>) -> String {
    walk_hoisted(cx, element);
    cx.buf.use_create_element_vnode();
    if hoist_needs_create_text(element) {
        cx.buf.use_create_text();
    }
    cx.buf.push_hoist(hoist_element_rhs(element, true))
}

pub(super) fn is_hoistable(element: &ElementOp<'_>) -> bool {
    is_static_element_tree(element)
}

pub(super) fn is_static_element_tree(element: &ElementOp<'_>) -> bool {
    super::vnode_static::can_whole_hoist_static_element(element)
}

fn walk_hoisted(cx: &mut EmitCx<'_>, element: &ElementOp<'_>) {
    for op in element.children.ops.iter() {
        match op {
            Op::Text(_) | Op::Interpolation(_) => {
                let _id = cx.walk.mint();
            }
            Op::Element(child) => {
                let _id = cx.walk.mint();
                cx.walk.skip(child.bindings.len());
                walk_hoisted(cx, child);
            }
            _ => {}
        }
    }
}

fn hoist_element_rhs(element: &ElementOp<'_>, pure: bool) -> String {
    let mut out = String::default();
    if pure {
        out.push_str("/*#__PURE__*/ ");
    }
    out.push_str(Buf::create_element_vnode_alias());
    out.push('(');
    out.push('"');
    out.push_str(element.tag);
    out.push('"');
    let kids = renderable_children(&element.children);
    let props = static_vnode_props(element, true);
    if props.is_some() || !kids.is_empty() {
        out.push_str(", ");
        if let Some(props) = props {
            out.push_str(props.as_str());
        } else {
            out.push_str("null");
        }
    }
    if !kids.is_empty() {
        out.push_str(", ");
        append_hoist_kids(&mut out, &kids);
    }
    out.push(')');
    out
}

fn cached_element_rhs(element: &ElementOp<'_>, cached: bool, line_indent: usize) -> String {
    let mut out = String::default();
    append_cached_element_rhs(&mut out, element, cached, line_indent);
    out
}

fn append_cached_element_rhs(
    out: &mut String,
    element: &ElementOp<'_>,
    cached: bool,
    line_indent: usize,
) {
    out.push_str(Buf::create_element_vnode_alias());
    out.push('(');
    out.push('"');
    out.push_str(element.tag);
    out.push('"');
    out.push_str(", ");
    if element.bindings.is_empty() && !element.attributes.is_empty() {
        cached_props::push_object(out, element.attributes.iter(), line_indent);
    } else if let Some(props) = cached_static_vnode_props(element, line_indent) {
        out.push_str(props.as_str());
    } else {
        out.push_str("null");
    }
    out.push_str(", ");
    let kids = renderable_children(&element.children);
    if kids.is_empty() {
        out.push_str("null");
    } else {
        append_cached_kids(out, &kids, line_indent);
    }
    if cached {
        out.push_str(", -1 /* CACHED */");
    }
    out.push(')');
}

fn static_vnode_props(element: &ElementOp<'_>, include_bindings: bool) -> Option<String> {
    if include_bindings {
        return super::props_static::root_hoist_props(&element.attributes, &element.bindings)
            .ok()
            .flatten();
    }
    if element.attributes.is_empty() {
        return None;
    }
    Some(compact_props_object(element.attributes.iter()))
}

fn cached_static_vnode_props(element: &ElementOp<'_>, line_indent: usize) -> Option<String> {
    super::props_static::cached_root_hoist_props(
        &element.attributes,
        &element.bindings,
        line_indent,
    )
    .ok()
    .flatten()
}
