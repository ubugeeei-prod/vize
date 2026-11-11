//! The region walk: children in document order, `v-if` sibling chains
//! grouped into region-owning [`IfOp`]s, `v-for` wrapped as [`ForOp`]s.
//!
//! Grouping at lowering is the point of the region design: the shipped
//! pipeline merges `v-else` branches onto the parent's child list
//! mid-traversal; a region-owning `ui.if` is built with its branches from
//! birth, so no pass ever revisits a sibling list. Whitespace-only text
//! and comments **between** consumed branches are dropped under
//! provenance (`drop.branch-gap`); the same nodes anywhere else lower
//! normally.
//!
//! `v-if` and `v-for` on one element follow Vue 3 precedence: the
//! conditional evaluates outside the iteration, so the branch region
//! holds the `ui.for` which holds the element.

use alloc::vec::Vec as StdVec;
use vize_l2::artifact::OpFamily;

use vize_l0::{Box, Span, String, Vec, cstr, ensure_sufficient_stack};
use vize_l1::markup::{Attribute, Element, SurfaceChild};

use vize_l2::analysis::fragment_facts::{BranchFragments, FragmentKey};
use vize_l2::op::{IfBranch, IfOp, Namespace, Op, Region};
use vize_l2::provenance::ProvenanceSink;

use super::cx::{Cx, attr_slice, attr_span, element_span};
use super::element::{
    Analyzed, BranchKind, Structure, analyze, attr_text, element_core, structure, template_unwraps,
};
use super::expr::expr_at;
use super::forop::lower_for;
use super::if_keys;
use super::leaf::lower_leaf;
use super::text;

mod wrapper;

pub(crate) use wrapper::{
    capture_wrapper_attrs, capture_wrapper_key, record_template_drops_except,
    retain_dynamic_wrapper_key, runtime_directive,
};
use wrapper::{chain_once_attr, record_wrapper_key};

/// One scanned branch of a chain: its sibling index, element, its
/// reading, branch attribute, and the gap children it consumes.
type Branch<'a, 't, R = Analyzed<'a>> =
    (usize, &'t Element<'a>, R, &'t Attribute<'a>, StdVec<usize>);

/// One `v-if` chain over L1 siblings, grouped as the lowering groups it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfChain {
    /// Each branch's sibling index and kind, in authored order.
    pub branches: StdVec<(usize, BranchKind)>,
    /// Whether a branch lowers without a condition.
    pub unconditional: bool,
    /// The index just past the last consumed sibling.
    pub end: usize,
}

/// A `v-else` value lowers as its condition only when it is not blank.
fn else_condition<'a>(attr: &Attribute<'a>) -> Option<&'a str> {
    attr_text(attr).filter(|text| !text.trim().is_empty())
}

/// The chain the lowering builds from `children[start]`, when that child
/// opens one. `in_v_pre` when an ancestor carries `v-pre`.
pub fn if_chain(children: &[SurfaceChild<'_>], start: usize, in_v_pre: bool) -> Option<IfChain> {
    let Some(SurfaceChild::Element(first)) = children.get(start) else {
        return None;
    };
    let read = structure(first, in_v_pre);
    let Some((attr, BranchKind::If)) = read.branch_attr(first) else {
        return None;
    };
    let (branches, end) = scan_if_chain(
        children,
        start,
        (start, first, read, attr, StdVec::new()),
        |element| structure(element, in_v_pre),
        Structure::branch_attr,
    );
    let kinds: StdVec<(usize, BranchKind)> = branches
        .iter()
        .map(|(index, _, read, _, _)| {
            let kind = read.branch.map_or(BranchKind::If, |(_, kind)| kind);
            (*index, kind)
        })
        .collect();
    let unconditional = branches
        .iter()
        .zip(&kinds)
        .any(|((_, _, _, attr, _), (_, kind))| {
            *kind == BranchKind::Else && else_condition(attr).is_none()
        });
    Some(IfChain {
        branches: kinds,
        unconditional,
        end,
    })
}

/// Scan a chain from its opening branch: the branches plus the gap nodes
/// each consumes, and the index just past the last consumed sibling.
fn scan_if_chain<'a, 't, R>(
    children: &'t [SurfaceChild<'a>],
    start: usize,
    first: Branch<'a, 't, R>,
    read: impl Fn(&'t Element<'a>) -> R,
    branch_attr: impl Fn(&R, &'t Element<'a>) -> Option<(&'t Attribute<'a>, BranchKind)>,
) -> (StdVec<Branch<'a, 't, R>>, usize) {
    let mut branches: StdVec<Branch<'a, 't, R>> = StdVec::new();
    branches.push(first);
    let mut j = start + 1;
    let mut pending: StdVec<usize> = StdVec::new();
    while let Some(child) = children.get(j) {
        match child {
            gap if is_branch_gap(gap) => pending.push(j),
            SurfaceChild::Element(element) => {
                let reading = read(element);
                match branch_attr(&reading, element) {
                    Some((attr, kind @ (BranchKind::ElseIf | BranchKind::Else))) => {
                        branches.push((j, element, reading, attr, core::mem::take(&mut pending)));
                        j += 1;
                        if kind == BranchKind::Else {
                            break;
                        }
                        continue;
                    }
                    _ => break,
                }
            }
            _ => break,
        }
        j += 1;
    }
    // Gap nodes past the last branch were scanned but not consumed; they
    // re-enter the main loop.
    (branches, j - pending.len())
}

/// Lower one children level into a region's ops, in document order.
///
/// The P2-9 text plan runs first (`lower::text::plan_whitespace` — the
/// condense decisions, computed while comments still stand in the
/// list), then the walk lowers text/interpolation children through the
/// run lane (`lower::text::lower_text_run` — merging), everything else
/// as before.
pub(crate) fn lower_children<'a>(
    cx: &mut Cx<'a, impl ProvenanceSink>,
    children: &[SurfaceChild<'a>],
    ns: Namespace,
) -> Vec<'a, Op<'a>> {
    ensure_sufficient_stack(|| lower_children_guarded(cx, children, ns))
}

fn lower_children_guarded<'a>(
    cx: &mut Cx<'a, impl ProvenanceSink>,
    children: &[SurfaceChild<'a>],
    ns: Namespace,
) -> Vec<'a, Op<'a>> {
    let plan = cx.text_lane().plan(children);
    let mut out: Vec<'a, Op<'a>> = Vec::new_in(&cx.allocator);
    let mut i = 0usize;
    while let Some(child) = children.get(i) {
        match child {
            SurfaceChild::Element(element) => {
                let analyzed = analyze(element, cx.v_pre_suppressed());
                match analyzed.branch_attr(element) {
                    Some((attr, BranchKind::If)) => {
                        i = lower_if_group(
                            cx,
                            children,
                            i,
                            (element, analyzed, attr),
                            ns,
                            &mut out,
                        );
                        continue;
                    }
                    Some((attr, BranchKind::ElseIf | BranchKind::Else)) => {
                        cx.error(
                            attr_span(cx, attr),
                            String::from("v-else/v-else-if has no adjacent v-if."),
                        );
                        cx.record(
                            "error.v-else-orphan",
                            None,
                            attr_slice(cx, attr),
                            String::default,
                            attr_span(cx, attr),
                        );
                        out.push(lower_with_for(cx, element, &analyzed, ns));
                    }
                    None => out.push(lower_with_for(cx, element, &analyzed, ns)),
                }
            }
            SurfaceChild::Text(_) | SurfaceChild::Interpolation(_) => {
                i = text::lower_text_run(cx, children, &plan, i, &mut out);
                continue;
            }
            other => lower_leaf(cx, other, &mut out),
        }
        i += 1;
    }
    text::debug_assert_text_family_gaps(&out);
    out
}

/// Whether a child may stand between two branches of one chain without
/// breaking it (and is dropped when the chain consumes it).
fn is_branch_gap(child: &SurfaceChild<'_>) -> bool {
    match child {
        SurfaceChild::Comment(_) => true,
        SurfaceChild::Text(token) => token.text.trim().is_empty(),
        _ => false,
    }
}

/// Lower a `v-if` chain starting at `start`. Returns the index just past
/// the last consumed sibling.
fn lower_if_group<'a, 't>(
    cx: &mut Cx<'a, impl ProvenanceSink>,
    children: &'t [SurfaceChild<'a>],
    start: usize,
    (first, first_analyzed, first_attr): (&'t Element<'a>, Analyzed<'a>, &'t Attribute<'a>),
    ns: Namespace,
    out: &mut Vec<'a, Op<'a>>,
) -> usize {
    let in_v_pre = cx.v_pre_suppressed();
    let (branches, consumed_until) = scan_if_chain(
        children,
        start,
        (start, first, first_analyzed, first_attr, StdVec::new()),
        |element| analyze(element, in_v_pre),
        Analyzed::branch_attr,
    );

    let node = cx.mint_op();
    let last_end = branches
        .last()
        .map(|(_, element, ..)| element_span(cx, element).end)
        .unwrap_or_else(|| element_span(cx, first).end);
    let span = Span::new(cx.offset(first.open.lt_name.text), last_end);

    cx.record(
        "lower.if",
        node,
        attr_slice(cx, first_attr),
        || cstr!("ui.if branches={}", branches.len()),
        span,
    );

    let mut lowered: Vec<'a, IfBranch<'a>> = Vec::new_in(&cx.allocator);
    let mut wrapper_keys: StdVec<Option<FragmentKey>> = StdVec::new();
    let mut branch_keys: StdVec<Option<if_keys::BranchKey>> = StdVec::new();
    let mut from_template: StdVec<bool> = StdVec::new();
    let mut once = false;
    for (_, element, analyzed, attr, gaps) in branches {
        for gap in gaps {
            if let Some(SurfaceChild::Text(token) | SurfaceChild::Comment(token)) =
                children.get(gap)
            {
                let gap_span = cx.token_span(token);
                cx.record(
                    "drop.branch-gap",
                    None,
                    token.text,
                    String::default,
                    gap_span,
                );
            }
        }
        let kind = match analyzed.branch {
            Some((_, kind)) => kind,
            None => BranchKind::If,
        };
        let condition = match kind {
            BranchKind::Else => else_condition(attr).map(|text| expr_at(cx, text)),
            BranchKind::If | BranchKind::ElseIf => {
                let text = attr_text(attr);
                if text.map(str::trim).is_none_or(str::is_empty) {
                    cx.error(
                        attr_span(cx, attr),
                        String::from("v-if/v-else-if is missing expression."),
                    );
                }
                let slice = match text {
                    Some(text) => text,
                    // No authored value at all: a zero-width position at
                    // the end of the directive name, admitted through the
                    // same total rule (refused as empty).
                    None => cx.hole_at(cx.token_span(&attr.name).end),
                };
                Some(expr_at(cx, slice))
            }
        };
        let branch_span = element_span(cx, element);
        let (ops, wrapper_key, is_template) = branch_body(cx, element, &analyzed, ns);
        once |= is_template && chain_once_attr(element, &analyzed).is_some();
        if let Some(key) = &wrapper_key {
            record_wrapper_key(cx, node, key, lowered.len());
        }
        let mut branch = IfBranch {
            condition,
            region: Region { ops },
            span: branch_span,
        };
        let branch_key = wrapper_key
            .as_ref()
            .map(if_keys::from_wrapper_key)
            .or_else(|| if_keys::take_carrier_key(&mut branch));
        branch_keys.push(branch_key);
        wrapper_keys.push(wrapper_key);
        from_template.push(is_template);
        lowered.push(branch);
    }
    if_keys::attach_if_facts(cx, node, branch_keys);
    if wrapper_keys.iter().any(Option::is_some) || from_template.iter().any(|flag| *flag) {
        cx.attach_wrappers(
            node,
            BranchFragments {
                branches: wrapper_keys,
                fragment: from_template,
                once,
            },
        );
    }
    cx.observe(OpFamily::If);
    out.push(Op::If(Box::new_in(
        IfOp {
            branches: lowered,
            span,
        },
        &cx.allocator,
    )));
    consumed_until
}

/// The ops a branch's element contributes (plus its captured wrapper
/// key and whether the carrier was an unwrapped `<template>`): its
/// `v-for` wrap when it has one, its children directly when it is an
/// unwrapped `<template>`, the element op otherwise.
fn branch_body<'a>(
    cx: &mut Cx<'a, impl ProvenanceSink>,
    element: &Element<'a>,
    analyzed: &Analyzed<'a>,
    ns: Namespace,
) -> (Vec<'a, Op<'a>>, Option<FragmentKey>, bool) {
    if analyzed.vfor.is_some() {
        // Vue 3 precedence: the key belongs to `v-for`, so no wrapper
        // capture — exactly the legacy `has_v_for` guard.
        let op = lower_for(cx, element, analyzed, ns);
        let mut ops: Vec<'a, Op<'a>> = Vec::new_in(&cx.allocator);
        ops.push(op);
        return (ops, None, false);
    }
    if template_unwraps(element, analyzed.has_slot_spelling()) {
        cx.report_missing_close(element);
        let captured = capture_wrapper_key(cx, element, analyzed);
        let (skip, key) = match captured {
            Some((index, key)) => {
                retain_dynamic_wrapper_key(cx, element, index, &key);
                (Some(index), Some(key))
            }
            None => (None, None),
        };
        let once = chain_once_attr(element, analyzed);
        record_template_drops_except(cx, element, analyzed, skip, once.as_slice());
        return (lower_children(cx, &element.children, ns), key, true);
    }
    let op = element_core(cx, element, analyzed, ns);
    let mut ops: Vec<'a, Op<'a>> = Vec::new_in(&cx.allocator);
    ops.push(op);
    (ops, None, false)
}

/// Lower an element, wrapping it in its `ui.for` when it carries one.
pub(crate) fn lower_with_for<'a>(
    cx: &mut Cx<'a, impl ProvenanceSink>,
    element: &Element<'a>,
    analyzed: &Analyzed<'a>,
    ns: Namespace,
) -> Op<'a> {
    if analyzed.vfor.is_some() {
        lower_for(cx, element, analyzed, ns)
    } else {
        element_core(cx, element, analyzed, ns)
    }
}
