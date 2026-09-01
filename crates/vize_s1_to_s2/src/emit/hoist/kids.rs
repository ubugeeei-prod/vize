use alloc::vec::Vec as StdVec;

use vize_s0::String;
use vize_s2::op::{ElementOp, Op, Region};

use super::super::buf::Buf;
use super::super::js::escape_js_string;

pub(super) fn hoist_needs_create_text(element: &ElementOp<'_>) -> bool {
    let kids = renderable_children(&element.children);
    let has_text = kids.iter().any(|op| matches!(op, Op::Text(_)));
    let has_other = kids.iter().any(|op| !matches!(op, Op::Text(_)));
    (has_text && has_other)
        || kids.iter().any(|op| match op {
            Op::Element(child) => hoist_needs_create_text(child),
            _ => false,
        })
}

pub(super) fn append_hoist_kids(out: &mut String, kids: &[&Op<'_>]) {
    if kids.iter().all(|op| matches!(op, Op::Text(_))) {
        out.push('"');
        for op in kids.iter() {
            if let Op::Text(text) = op {
                out.push_str(escape_js_string(text.content).as_str());
            }
        }
        out.push('"');
        return;
    }
    out.push('[');
    for (i, op) in kids.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        match op {
            Op::Text(text) => {
                out.push_str(Buf::create_text_alias());
                out.push('(');
                out.push('"');
                out.push_str(escape_js_string(text.content).as_str());
                out.push('"');
                out.push(')');
            }
            Op::Element(element) => {
                out.push_str(hoist_descendant_element_rhs(element).as_str());
            }
            _ => {}
        }
    }
    out.push(']');
}

fn hoist_descendant_element_rhs(element: &ElementOp<'_>) -> String {
    let mut out = String::default();
    out.push_str(Buf::create_element_vnode_alias());
    out.push('(');
    out.push('"');
    out.push_str(element.tag);
    out.push('"');
    let kids = renderable_children(&element.children);
    let props = super::static_vnode_props(element, false);
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

pub(super) fn append_cached_kids(out: &mut String, kids: &[&Op<'_>], line_indent: usize) {
    if kids.iter().all(|op| matches!(op, Op::Text(_))) {
        out.push('"');
        for op in kids.iter() {
            if let Op::Text(text) = op {
                out.push_str(escape_js_string(text.content).as_str());
            }
        }
        out.push('"');
        return;
    }
    out.push('[');
    for (i, op) in kids.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('\n');
        push_spaces(out, line_indent + 2);
        match op {
            Op::Text(text) => {
                push_cached_create_text_call(out, text.content);
            }
            Op::Element(element) => {
                super::append_cached_element_rhs(out, element, false, line_indent + 2);
            }
            _ => {}
        }
    }
    out.push('\n');
    push_spaces(out, line_indent);
    out.push(']');
}

fn push_cached_create_text_call(out: &mut String, content: &str) {
    out.push_str(Buf::create_text_alias());
    if content == " " {
        out.push_str("()");
        return;
    }
    out.push('(');
    out.push('"');
    out.push_str(escape_js_string(content).as_str());
    out.push('"');
    out.push(')');
}

pub(in crate::emit) fn push_spaces(out: &mut String, width: usize) {
    out.extend(core::iter::repeat_n(' ', width));
}

pub(super) fn renderable_children<'a>(children: &'a Region<'a>) -> StdVec<&'a Op<'a>> {
    // S2 lowering has already applied legacy condense/drop decisions; every
    // remaining text op is renderable, including a single-space separator.
    children.ops.iter().collect()
}
