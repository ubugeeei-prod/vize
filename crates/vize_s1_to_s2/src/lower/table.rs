//! HTML table tree-construction normalizations needed before S2 ids are minted.
//!
//! Vue's DOM compiler rides the HTML parser shape: direct rows under a table
//! gain an implicit `tbody`, and direct cells under a row group gain an
//! implicit `tr`. S2 inserts those owners during lowering so synthesized op ids
//! stay in folio page order.

use vize_s0::{Box, Span, Vec, cstr};
use vize_s1::{Interpolation, SurfaceChild};
use vize_s2::op::{Attribute, BindingOp, ElementOp, Namespace, Op, Region};

use super::cx::{Cx, element_span};
use super::structural::lower_children;

pub(crate) fn lower_element_children<'a>(
    cx: &mut Cx<'a>,
    tag: &str,
    children: &[SurfaceChild<'a>],
    ns: Namespace,
) -> Vec<'a, Op<'a>> {
    match tag {
        "table" => lower_table_children(cx, children, ns),
        "thead" | "tbody" | "tfoot" => lower_row_group_children(cx, children, ns),
        _ => lower_children(cx, children, ns),
    }
}

fn lower_table_children<'a>(
    cx: &mut Cx<'a>,
    children: &[SurfaceChild<'a>],
    ns: Namespace,
) -> Vec<'a, Op<'a>> {
    lower_grouped_children(cx, children, ns, starts_implicit_tbody, |cx, segment| {
        implicit_element(cx, "tbody", "lower.table.implicit-tbody", segment, |cx| {
            lower_row_group_children(cx, segment, ns)
        })
    })
}

fn lower_row_group_children<'a>(
    cx: &mut Cx<'a>,
    children: &[SurfaceChild<'a>],
    ns: Namespace,
) -> Vec<'a, Op<'a>> {
    lower_grouped_children(cx, children, ns, starts_implicit_tr, |cx, segment| {
        implicit_element(cx, "tr", "lower.table.implicit-tr", segment, |cx| {
            lower_children(cx, segment, ns)
        })
    })
}

fn lower_grouped_children<'a>(
    cx: &mut Cx<'a>,
    children: &[SurfaceChild<'a>],
    ns: Namespace,
    starts_group: fn(&SurfaceChild<'_>) -> bool,
    mut lower_group: impl FnMut(&mut Cx<'a>, &[SurfaceChild<'a>]) -> Op<'a>,
) -> Vec<'a, Op<'a>> {
    let mut out: Vec<'a, Op<'a>> = Vec::new_in(&cx.allocator);
    let mut index = 0usize;
    while index < children.len() {
        if starts_group(&children[index]) {
            let end = scan_group(children, index, starts_group);
            out.push(lower_group(cx, &children[index..end]));
            index = end;
            continue;
        }

        let start = index;
        index += 1;
        while index < children.len() && !starts_group(&children[index]) {
            index += 1;
        }
        push_all(&mut out, lower_children(cx, &children[start..index], ns));
    }
    out
}

fn scan_group(
    children: &[SurfaceChild<'_>],
    start: usize,
    starts_group: fn(&SurfaceChild<'_>) -> bool,
) -> usize {
    let mut end = start + 1;
    let mut index = end;
    while index < children.len() {
        if starts_group(&children[index]) {
            end = index + 1;
            index += 1;
            continue;
        }
        if is_table_gap(&children[index]) {
            index += 1;
            continue;
        }
        break;
    }
    end
}

fn implicit_element<'a>(
    cx: &mut Cx<'a>,
    tag: &'static str,
    rule: &'static str,
    segment: &[SurfaceChild<'a>],
    lower_children: impl FnOnce(&mut Cx<'a>) -> Vec<'a, Op<'a>>,
) -> Op<'a> {
    let span = segment_span(cx, segment);
    let node = cx.mint_op();
    cx.record(rule, node, "", cstr!("ui.element {tag}"), span);
    Op::Element(Box::new_in(
        ElementOp {
            tag,
            namespace: Namespace::Html,
            attributes: Vec::<Attribute<'a>>::new_in(&cx.allocator),
            bindings: Vec::<BindingOp<'a>>::new_in(&cx.allocator),
            children: Region {
                ops: lower_children(cx),
            },
            span,
        },
        &cx.allocator,
    ))
}

fn push_all<'a>(out: &mut Vec<'a, Op<'a>>, ops: Vec<'a, Op<'a>>) {
    for op in ops {
        out.push(op);
    }
}

fn starts_implicit_tbody(child: &SurfaceChild<'_>) -> bool {
    match child {
        SurfaceChild::Element(element) => matches!(element.tag(), "tr" | "td" | "th"),
        _ => false,
    }
}

fn starts_implicit_tr(child: &SurfaceChild<'_>) -> bool {
    match child {
        SurfaceChild::Element(element) => matches!(element.tag(), "td" | "th"),
        _ => false,
    }
}

fn is_table_gap(child: &SurfaceChild<'_>) -> bool {
    match child {
        SurfaceChild::Comment(_) => true,
        SurfaceChild::Text(token) => token.text.trim().is_empty(),
        _ => false,
    }
}

fn segment_span(cx: &Cx<'_>, segment: &[SurfaceChild<'_>]) -> Span {
    let first = child_span(cx, &segment[0]);
    let last = child_span(cx, &segment[segment.len() - 1]);
    Span::new(first.start, last.end)
}

fn child_span(cx: &Cx<'_>, child: &SurfaceChild<'_>) -> Span {
    match child {
        SurfaceChild::Element(element) => element_span(cx, element),
        SurfaceChild::Text(token)
        | SurfaceChild::Comment(token)
        | SurfaceChild::Cdata(token)
        | SurfaceChild::ProcessingInstruction(token)
        | SurfaceChild::Unexpected(token) => cx.token_span(token),
        SurfaceChild::Interpolation(node) => interpolation_span(cx, node),
    }
}

fn interpolation_span(cx: &Cx<'_>, node: &Interpolation<'_>) -> Span {
    Span::new(cx.offset(node.open.text), cx.token_span(&node.close).end)
}
