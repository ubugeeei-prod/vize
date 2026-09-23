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
    let mut rest = children;
    while let Some(first) = rest.first() {
        let grouped = starts_group(first);
        let len = if grouped {
            group_len(rest, starts_group)
        } else {
            1 + rest
                .iter()
                .skip(1)
                .take_while(|child| !starts_group(child))
                .count()
        };
        let (segment, tail) = rest.split_at_checked(len).unwrap_or((rest, &[]));
        if grouped {
            out.push(lower_group(cx, segment));
        } else {
            push_all(&mut out, lower_children(cx, segment, ns));
        }
        rest = tail;
    }
    out
}

/// The length of the group opening `children`: through the last child
/// that starts the group, across table gaps between them.
fn group_len(children: &[SurfaceChild<'_>], starts_group: fn(&SurfaceChild<'_>) -> bool) -> usize {
    let mut end = 1;
    for (index, child) in children.iter().enumerate().skip(1) {
        if starts_group(child) {
            end = index + 1;
        } else if !is_table_gap(child) {
            break;
        }
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
    match (segment.first(), segment.last()) {
        (Some(first), Some(last)) => {
            Span::new(child_span(cx, first).start, child_span(cx, last).end)
        }
        // Grouping never yields an empty segment; zero-width at the block
        // start keeps the span inside the source regardless.
        _ => cx.span_of(cx.hole_at(0)),
    }
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
