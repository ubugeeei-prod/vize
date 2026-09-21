//! Element-tree walks over an S2 artifact.
//!
//! Scopes are regions in S2; the element tree the facade presents keeps the
//! authored carriers visible: a `ui.if` branch or `ui.for` region unwrapped
//! from a `<template>` surfaces that template (read off S1) around its region.

use super::{S2ElementOp, S2Markup};
use crate::markup::element::MarkupElement;
use vize_s0::Span;
use vize_s2::op::Op;

/// One step of a region walk: an element to enter (and later exit), or an
/// unwrapped `<template>` carrier around a scope region.
pub(in crate::markup) enum S2Step<'a> {
    /// An element-shaped op (or a carrier) and the region it owns.
    Element(MarkupElement<'a>, &'a [Op<'a>]),
    /// Region content with no authored carrier.
    Region(&'a [Op<'a>]),
}

/// The step a scope region contributes: its unwrapped carrier when S1
/// records one, the bare region otherwise.
pub(in crate::markup) fn scope_region<'a>(
    doc: &'a S2Markup<'a>,
    span: Span,
    region: &'a [Op<'a>],
) -> S2Step<'a> {
    match doc.unwrapped_carrier(span, region) {
        Some(carrier) => S2Step::Element(
            MarkupElement::from_s2_carrier(carrier, doc, region, span),
            region,
        ),
        None => S2Step::Region(region),
    }
}

/// Walk every element of `doc` in tree order, calling `enter` / `exit`.
pub(in crate::markup) fn walk_tree<'a>(
    doc: &'a S2Markup<'a>,
    enter: &mut impl FnMut(MarkupElement<'a>),
    exit: &mut impl FnMut(MarkupElement<'a>),
) {
    walk_region(doc, &doc.root.ops, enter, exit);
}

fn walk_step<'a>(
    doc: &'a S2Markup<'a>,
    step: S2Step<'a>,
    enter: &mut impl FnMut(MarkupElement<'a>),
    exit: &mut impl FnMut(MarkupElement<'a>),
) {
    match step {
        S2Step::Element(element, region) => {
            enter(element);
            walk_region(doc, region, enter, exit);
            exit(element);
        }
        S2Step::Region(region) => walk_region(doc, region, enter, exit),
    }
}

fn walk_region<'a>(
    doc: &'a S2Markup<'a>,
    ops: &'a [Op<'a>],
    enter: &mut impl FnMut(MarkupElement<'a>),
    exit: &mut impl FnMut(MarkupElement<'a>),
) {
    for op in ops {
        match op {
            Op::Element(_) | Op::Component(_) | Op::Slot(_) => {
                let Some(element) = S2ElementOp::from_op(op) else {
                    continue;
                };
                let step =
                    S2Step::Element(MarkupElement::from_s2(element, doc), element.children());
                walk_step(doc, step, enter, exit);
            }
            Op::If(if_op) => {
                for branch in if_op.branches.iter() {
                    let step = scope_region(doc, branch.span, &branch.region.ops);
                    walk_step(doc, step, enter, exit);
                }
            }
            Op::For(for_op) => {
                let step = scope_region(doc, for_op.span, &for_op.region.ops);
                walk_step(doc, step, enter, exit);
            }
            Op::Text(_) | Op::Interpolation(_) | Op::Comment(_) => {}
        }
    }
}
