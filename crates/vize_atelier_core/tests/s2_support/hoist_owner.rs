//! The hoist projection's per-owner verdict arms (element, component,
//! outlet), split from [`super::hoist_walk`] under the source budget.
//! [`super::hoist`]'s module docs carry the design; the short version:
//! descend where the shipped driver descended (legacy ground truth),
//! compare the fact-driven prediction against the mutation the
//! hoist-armed run actually made, honour the counted taints.

use vize_atelier_core::{ElementNode, ElementType, StaticType, TemplateChildNode, get_static_type};
use vize_davinci::side_table::SideTable;
use vize_s1_to_s2::pass::StaticFacts;
use vize_s2::folio::{FolioComponent, FolioElement, FolioOp, FolioSlot};

use super::hoist::{HoistCounters, Mode, fact_of, predict, predict_for_item, replay_or_dormant};
use super::hoist_old::{
    Decision, carries_deferred_builtin, decision_of, has_comment_child, has_directives,
};
use super::hoist_walk::walk_level;

/// One native element position; returns its upward taint.
#[expect(clippy::too_many_arguments, reason = "one recursive comparator walk")]
pub fn walk_element(
    name: &str,
    source: &str,
    o1: &TemplateChildNode<'_>,
    el1: &ElementNode<'_>,
    o2: &TemplateChildNode<'_>,
    element: &FolioElement,
    mode: Mode,
    suppressed: bool,
    next: &mut u32,
    facts: &SideTable<StaticFacts>,
    counters: &mut HoistCounters,
) -> bool {
    let owner = *next;
    *next += 1 + u32::try_from(element.bindings.len()).expect("binding count fits");
    let legacy_level = get_static_type(o1);
    let local_comment = has_comment_child(el1);
    let local_builtin = carries_deferred_builtin(el1);
    if local_comment {
        counters.comments_elements += 1;
    }
    if local_builtin {
        counters.builtins_subtrees += 1;
    }
    let fact = fact_of(facts, owner, name, source);
    let legacy_dirs = has_directives(el1);
    let legacy_nonel = el1.tag_type != ElementType::Element;

    // Descend first, per the legacy arm — ground truth — so the
    // verdict can honour descendant taints.
    let for_item = matches!(mode, Mode::Replay { for_item: true, .. });
    let sub_tainted = match (mode, legacy_level) {
        (Mode::Replay { vnodes, .. }, StaticType::NotStatic) => {
            let o2_children = match o2 {
                TemplateChildNode::Element(el2) => &el2.children[..],
                _ => panic!("a NotStatic element was whole-hoisted in {name}\n{source}"),
            };
            walk_level(
                name,
                source,
                &el1.children,
                o2_children,
                &element.children,
                Mode::Replay {
                    is_root: false,
                    vnodes: vnodes || legacy_dirs || legacy_nonel,
                    for_item: false,
                },
                suppressed || local_builtin,
                next,
                facts,
                counters,
            )
        }
        (Mode::Replay { .. }, _) if for_item => {
            // `hoist_for_children` always descends the item with vnodes on.
            let o2_children = match o2 {
                TemplateChildNode::Element(el2) => &el2.children[..],
                _ => panic!("a v-for item was whole-hoisted in {name}\n{source}"),
            };
            walk_level(
                name,
                source,
                &el1.children,
                o2_children,
                &element.children,
                Mode::Replay {
                    is_root: false,
                    vnodes: true,
                    for_item: false,
                },
                suppressed || local_builtin,
                next,
                facts,
                counters,
            )
        }
        _ => {
            // Fully-static / dynamic-text arms never recurse in the
            // shipped driver (and Dormant stays dormant): no decision
            // can exist below. A whole-hoisted subtree is gone from
            // the legacy tree; advance the S2 cursor only.
            match o2 {
                TemplateChildNode::Hoisted(_) => advance_ops(&element.children, next),
                TemplateChildNode::Element(el2) => {
                    walk_level(
                        name,
                        source,
                        &el1.children,
                        &el2.children,
                        &element.children,
                        Mode::Dormant,
                        suppressed || local_builtin,
                        next,
                        facts,
                        counters,
                    );
                }
                _ => panic!("element position mutated unexpectedly in {name}\n{source}"),
            }
            false
        }
    };

    let verdict_suppressed = suppressed || local_comment || local_builtin || sub_tainted;
    match mode {
        Mode::Replay {
            is_root,
            vnodes,
            for_item,
        } if !verdict_suppressed => {
            // The vnodes-contribution twin: what this element hands its
            // children must agree across lanes.
            assert_eq!(
                legacy_dirs || legacy_nonel,
                !element.bindings.is_empty(),
                "vnodes contribution diverged on <{}> in {name}\n{source}",
                el1.tag,
            );
            let predicted = if for_item {
                predict_for_item(fact)
            } else {
                predict(fact, is_root, vnodes)
            };
            let actual = decision_of(o2);
            assert_eq!(
                predicted, actual,
                "hoist decision diverged on <{}> in {name} (facts {fact:?}, \
                 is_root={is_root} vnodes={vnodes}, \
                 legacy level {legacy_level:?})\n{source}",
                el1.tag,
            );
            counters.elements += 1;
            match actual {
                Decision::Whole => counters.whole += 1,
                Decision::Props => counters.props += 1,
                Decision::None => {}
            }
        }
        Mode::Dormant if !verdict_suppressed => {
            assert_eq!(
                decision_of(o2),
                Decision::None,
                "a decision appeared under an undescended arm in {name}\n{source}"
            );
            counters.elements += 1;
        }
        _ => {}
    }
    local_comment || local_builtin || sub_tainted
}

/// One component position; returns its upward taint (interior taints
/// stop at the component boundary — its level contribution is fixed).
#[expect(clippy::too_many_arguments, reason = "one recursive comparator walk")]
pub fn walk_component(
    name: &str,
    source: &str,
    el1: &ElementNode<'_>,
    o2: &TemplateChildNode<'_>,
    component: &FolioComponent,
    mode: Mode,
    suppressed: bool,
    next: &mut u32,
    facts: &SideTable<StaticFacts>,
    counters: &mut HoistCounters,
) -> bool {
    let owner = *next;
    *next += 1 + u32::try_from(component.bindings.len()).expect("binding count fits");
    let local_comment = has_comment_child(el1);
    let local_builtin = carries_deferred_builtin(el1);
    if local_comment {
        counters.comments_elements += 1;
    }
    if local_builtin {
        counters.builtins_subtrees += 1;
    }
    let fact = fact_of(facts, owner, name, source);
    let o2_children = match o2 {
        TemplateChildNode::Element(el2) => &el2.children[..],
        _ => panic!("a component position was whole-hoisted in {name}\n{source}"),
    };
    // Components are NotStatic by kind in both lanes; the driver always
    // recurses with the vnodes flag forced (tag_type != Element).
    let sub_tainted = walk_level(
        name,
        source,
        &el1.children,
        o2_children,
        &component.children,
        replay_or_dormant(mode, false, true),
        suppressed || local_builtin,
        next,
        facts,
        counters,
    );
    let verdict_suppressed = suppressed || local_comment || local_builtin || sub_tainted;
    match mode {
        Mode::Replay {
            is_root,
            vnodes,
            for_item,
        } if !verdict_suppressed => {
            let predicted = if for_item {
                predict_for_item(fact)
            } else {
                predict(fact, is_root, vnodes)
            };
            let actual = decision_of(o2);
            assert_eq!(
                predicted, actual,
                "hoist decision diverged on component <{}> in {name} (facts {fact:?})\n{source}",
                el1.tag,
            );
            counters.elements += 1;
            if actual == Decision::Props {
                counters.props += 1;
            }
        }
        Mode::Dormant if !verdict_suppressed => {
            assert_eq!(
                decision_of(o2),
                Decision::None,
                "a decision appeared under an undescended arm in {name}\n{source}"
            );
            counters.elements += 1;
        }
        _ => {}
    }
    local_comment || local_builtin
}

/// One outlet position; returns its upward taint (a builtin on the
/// outlet moves its parent's nested gate; interior taints stop at the
/// outlet boundary).
#[expect(clippy::too_many_arguments, reason = "one recursive comparator walk")]
pub fn walk_slot(
    name: &str,
    source: &str,
    el1: &ElementNode<'_>,
    o2: &TemplateChildNode<'_>,
    slot: &FolioSlot,
    mode: Mode,
    suppressed: bool,
    next: &mut u32,
    facts: &SideTable<StaticFacts>,
    counters: &mut HoistCounters,
) -> bool {
    *next += 1 + u32::try_from(slot.bindings.len()).expect("binding count fits");
    let local_builtin = carries_deferred_builtin(el1);
    if local_builtin {
        counters.builtins_subtrees += 1;
    }
    if matches!(mode, Mode::Replay { .. }) && !suppressed && !local_builtin {
        // An outlet never hoists in the shipped lane
        // (`has_static_props` refuses slots); its position is still a
        // compared verdict.
        assert_eq!(
            decision_of(o2),
            Decision::None,
            "an outlet position hoisted in {name}\n{source}"
        );
        counters.elements += 1;
    }
    let o2_children = match o2 {
        TemplateChildNode::Element(el2) => &el2.children[..],
        _ => panic!("outlet position mutated unexpectedly in {name}\n{source}"),
    };
    // The driver recurses into the outlet's fallback with the vnodes
    // flag forced (tag_type != Element).
    walk_level(
        name,
        source,
        &el1.children,
        o2_children,
        &slot.fallback,
        replay_or_dormant(mode, false, true),
        suppressed || local_builtin,
        next,
        facts,
        counters,
    );
    local_builtin
}

/// Advance the page-order cursor across a subtree without walking a
/// legacy counterpart (a legacy whole-hoist consumed it).
pub fn advance_ops(ops: &[FolioOp], next: &mut u32) {
    for op in ops {
        match op {
            FolioOp::Element(element) => {
                *next += 1 + u32::try_from(element.bindings.len()).expect("binding count fits");
                advance_ops(&element.children, next);
            }
            FolioOp::Component(component) => {
                *next += 1 + u32::try_from(component.bindings.len()).expect("binding count fits");
                advance_ops(&component.children, next);
            }
            FolioOp::Slot(slot) => {
                *next += 1 + u32::try_from(slot.bindings.len()).expect("binding count fits");
                advance_ops(&slot.fallback, next);
            }
            FolioOp::If(if_op) => {
                *next += 1;
                for branch in if_op.branches.iter() {
                    advance_ops(&branch.ops, next);
                }
            }
            FolioOp::For(for_op) => {
                *next += 1;
                advance_ops(&for_op.ops, next);
            }
            FolioOp::Text(_) | FolioOp::Interpolation(_) => *next += 1,
        }
    }
}
