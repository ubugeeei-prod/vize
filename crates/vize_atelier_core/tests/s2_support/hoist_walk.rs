//! The hoist projection's three-tree walk ([`super::hoist`]'s module
//! docs carry the design): run-1 legacy children (replay control),
//! run-2 legacy children (the decisions), and the S2 folio ops (the
//! facts' side), in lockstep. Split from `hoist.rs` under the source
//! budget; the element/component verdict arms live in
//! [`super::hoist_owner`].

use vize_atelier_core::{IfBranchNode, TemplateChildNode};
use vize_davinci::side_table::SideTable;
use vize_s1_to_s2::pass::StaticFacts;
use vize_s2::folio::FolioOp;

use super::hoist::{HoistCounters, Mode, replay_or_dormant, walk_for_body};
use super::hoist_old::{Decision, decision_of};
use super::hoist_owner::{walk_component, walk_element, walk_slot};

/// Structural filter over legacy children (comments, text and
/// interpolations carry no decisions).
pub fn structural<'t, 'a>(children: &'t [TemplateChildNode<'a>]) -> Vec<&'t TemplateChildNode<'a>> {
    children
        .iter()
        .filter(|child| {
            matches!(
                child,
                TemplateChildNode::Element(_)
                    | TemplateChildNode::If(_)
                    | TemplateChildNode::For(_)
                    | TemplateChildNode::Hoisted(_)
            )
        })
        .collect()
}

/// Structural filter over folio ops.
pub fn structural_s2(ops: &[FolioOp]) -> Vec<&FolioOp> {
    ops.iter()
        .filter(|op| {
            matches!(
                op,
                FolioOp::Element(_)
                    | FolioOp::Component(_)
                    | FolioOp::Slot(_)
                    | FolioOp::If(_)
                    | FolioOp::For(_)
            )
        })
        .collect()
}

/// Walk one aligned level. Returns whether the level (element-interior
/// subtrees included) carries a taint the parent's verdict must honour.
#[expect(clippy::too_many_arguments, reason = "one recursive comparator walk")]
pub fn walk_level(
    name: &str,
    source: &str,
    old1: &[TemplateChildNode<'_>],
    old2: &[TemplateChildNode<'_>],
    s2: &[FolioOp],
    mode: Mode,
    suppressed: bool,
    next: &mut u32,
    facts: &SideTable<StaticFacts>,
    counters: &mut HoistCounters,
) -> bool {
    let o1 = structural(old1);
    let o2 = structural(old2);
    let ops = structural_s2(s2);
    assert!(
        o1.len() == o2.len() && o1.len() == ops.len(),
        "hoist projection misaligned in {name} ({} / {} / {} structural positions)\n{source}",
        o1.len(),
        o2.len(),
        ops.len(),
    );
    let mut tainted = false;
    let mut cursor = 0usize;
    for op in s2 {
        // Leaf ops between structural positions still consume ids.
        if matches!(op, FolioOp::Text(_) | FolioOp::Interpolation(_)) {
            *next += 1;
            continue;
        }
        let position = cursor;
        cursor += 1;
        tainted |= walk_position(
            name,
            source,
            o1[position],
            o2[position],
            op,
            mode,
            suppressed,
            next,
            facts,
            counters,
        );
    }
    tainted
}

/// Walk one aligned structural position; returns its upward taint.
#[expect(clippy::too_many_arguments, reason = "one recursive comparator walk")]
pub fn walk_position(
    name: &str,
    source: &str,
    o1: &TemplateChildNode<'_>,
    o2: &TemplateChildNode<'_>,
    op: &FolioOp,
    mode: Mode,
    suppressed: bool,
    next: &mut u32,
    facts: &SideTable<StaticFacts>,
    counters: &mut HoistCounters,
) -> bool {
    match (o1, o2, op) {
        (TemplateChildNode::If(node1), TemplateChildNode::If(node2), FolioOp::If(if_op)) => {
            *next += 1;
            assert_eq!(
                node1.branches.len(),
                if_op.branches.len(),
                "branch count misaligned in {name}\n{source}"
            );
            for (index, branch) in if_op.branches.iter().enumerate() {
                let b1 = &node1.branches[index];
                let b2 = &node2.branches[index];
                match (&b1.children[..], &b2.children[..]) {
                    ([TemplateChildNode::Element(el1)], [TemplateChildNode::Element(el2)])
                        if b1.is_template_if =>
                    {
                        // Wrapper branch: the S2 region is the
                        // template's content; the driver decided that
                        // content with the vnodes flag on.
                        walk_level(
                            name,
                            source,
                            &el1.children,
                            &el2.children,
                            &branch.ops,
                            replay_or_dormant(mode, false, true),
                            suppressed,
                            next,
                            facts,
                            counters,
                        );
                    }
                    _ => {
                        walk_branch_roots(
                            name,
                            source,
                            b1,
                            b2,
                            &branch.ops,
                            mode,
                            suppressed,
                            next,
                            facts,
                            counters,
                        );
                    }
                }
            }
            false
        }
        (TemplateChildNode::For(node1), TemplateChildNode::For(node2), FolioOp::For(for_op)) => {
            *next += 1;
            match (&node1.children[..], &node2.children[..]) {
                ([TemplateChildNode::Element(el1)], [TemplateChildNode::Element(el2)])
                    if el1.tag == "template" =>
                {
                    // The `<template v-for>` wrapper stays in the
                    // legacy tree and is decided there; S2 keeps no
                    // wrapper position, so a legacy wrapper hoist is
                    // counted, never compared. Inner children re-enter
                    // `hoist_for_children`.
                    if el2.hoisted_props_index.is_some() {
                        counters.wrapper_hoists += 1;
                    }
                    walk_for_body(
                        name,
                        source,
                        &el1.children,
                        &el2.children,
                        &for_op.ops,
                        mode,
                        suppressed,
                        next,
                        facts,
                        counters,
                    );
                }
                _ => {
                    walk_for_body(
                        name,
                        source,
                        &node1.children,
                        &node2.children,
                        &for_op.ops,
                        mode,
                        suppressed,
                        next,
                        facts,
                        counters,
                    );
                }
            }
            false
        }
        (TemplateChildNode::Element(el1), _, FolioOp::Slot(slot)) => walk_slot(
            name, source, el1, o2, slot, mode, suppressed, next, facts, counters,
        ),
        (TemplateChildNode::Element(el1), _, FolioOp::Element(element)) => walk_element(
            name, source, o1, el1, o2, element, mode, suppressed, next, facts, counters,
        ),
        (TemplateChildNode::Element(el1), _, FolioOp::Component(component)) => walk_component(
            name, source, el1, o2, component, mode, suppressed, next, facts, counters,
        ),
        _ => panic!("hoist projection pairing broke in {name}: {o1:?} / {op:?}\n{source}"),
    }
}

/// Non-wrapper branch roots: no decisions at the roots (the shipped If
/// arm makes none), Element-kind roots have their children decided
/// with the vnodes flag on, For/If roots are skipped whole.
#[expect(clippy::too_many_arguments, reason = "one recursive comparator walk")]
fn walk_branch_roots(
    name: &str,
    source: &str,
    b1: &IfBranchNode<'_>,
    b2: &IfBranchNode<'_>,
    ops: &[FolioOp],
    mode: Mode,
    suppressed: bool,
    next: &mut u32,
    facts: &SideTable<StaticFacts>,
    counters: &mut HoistCounters,
) {
    let r1 = structural(&b1.children);
    let r2 = structural(&b2.children);
    let roots = structural_s2(ops);
    assert!(
        r1.len() == r2.len() && r1.len() == roots.len(),
        "branch roots misaligned in {name}\n{source}"
    );
    let mut cursor = 0usize;
    for op in ops {
        if matches!(op, FolioOp::Text(_) | FolioOp::Interpolation(_)) {
            *next += 1;
            continue;
        }
        let o1 = r1[cursor];
        let o2 = r2[cursor];
        cursor += 1;
        if matches!(mode, Mode::Replay { .. }) {
            assert_eq!(
                decision_of(o2),
                Decision::None,
                "a branch root carried a decision in {name}\n{source}"
            );
        }
        match (o1, o2, op) {
            (TemplateChildNode::Element(el1), TemplateChildNode::Element(el2), _) => {
                let (region, bindings) = match op {
                    FolioOp::Element(element) => (&element.children, element.bindings.len()),
                    FolioOp::Component(component) => {
                        (&component.children, component.bindings.len())
                    }
                    FolioOp::Slot(slot) => (&slot.fallback, slot.bindings.len()),
                    _ => panic!("branch root kinds misaligned in {name}\n{source}"),
                };
                *next += 1 + u32::try_from(bindings).expect("binding count fits");
                walk_level(
                    name,
                    source,
                    &el1.children,
                    &el2.children,
                    region,
                    replay_or_dormant(mode, false, true),
                    suppressed,
                    next,
                    facts,
                    counters,
                );
            }
            _ => {
                // A For/If branch root: the shipped If arm skips it
                // entirely — walk dormant for alignment.
                walk_position(
                    name,
                    source,
                    o1,
                    o2,
                    op,
                    Mode::Dormant,
                    suppressed,
                    next,
                    facts,
                    counters,
                );
            }
        }
    }
}
