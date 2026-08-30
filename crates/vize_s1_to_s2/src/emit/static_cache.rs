//! Legacy static-cache gate for child-list emission.
//!
//! The shipped codegen enables `_cache[n]` static vnode reuse when the
//! transform produced any root hoist. S2 realizes hoists directly during emit,
//! so this pre-walk reconstructs the old `!root.hoists.is_empty()` gate from
//! page-order ids plus `StaticFacts`.

use vize_s0::ensure_sufficient_stack;
use vize_s2::op::{ComponentOp, ElementOp, Op, Region};

use crate::pass::walk::PageWalk;
use crate::pass::{S2Facts, StaticFacts, StaticLevel};

pub(super) fn enabled(root: &Region<'_>, facts: &S2Facts) -> bool {
    let mut walk = PageWalk::new();
    region_has_legacy_hoist(&mut walk, &root.ops, facts, true, false)
}

fn region_has_legacy_hoist(
    walk: &mut PageWalk,
    ops: &[Op<'_>],
    facts: &S2Facts,
    is_root: bool,
    hoist_static_vnodes: bool,
) -> bool {
    for op in ops {
        if op_has_legacy_hoist(walk, op, facts, is_root, hoist_static_vnodes) {
            return true;
        }
    }
    false
}

fn op_has_legacy_hoist(
    walk: &mut PageWalk,
    op: &Op<'_>,
    facts: &S2Facts,
    is_root: bool,
    hoist_static_vnodes: bool,
) -> bool {
    let id = walk.mint();
    match op {
        Op::Element(element) => {
            walk.skip(element.bindings.len());
            let Some(fact) = id.and_then(|id| facts.static_facts.get(id)).copied() else {
                skip_region(walk, &element.children.ops);
                return false;
            };
            element_has_legacy_hoist(walk, element, fact, facts, is_root, hoist_static_vnodes)
        }
        Op::Component(component) => {
            walk.skip(component.bindings.len());
            let fact = id.and_then(|id| facts.static_facts.get(id)).copied();
            component_has_legacy_hoist(walk, component, fact, facts)
        }
        Op::If(if_op) => {
            for branch in if_op.branches.iter() {
                if branch_roots_have_legacy_hoist(walk, &branch.region.ops, facts) {
                    return true;
                }
            }
            false
        }
        Op::For(for_op) => for_children_have_legacy_hoist(walk, &for_op.region.ops, facts),
        Op::Slot(slot) => {
            walk.skip(slot.bindings.len());
            skip_region(walk, &slot.fallback.ops);
            false
        }
        Op::Text(_) | Op::Interpolation(_) => false,
    }
}

fn element_has_legacy_hoist(
    walk: &mut PageWalk,
    element: &ElementOp<'_>,
    fact: StaticFacts,
    facts: &S2Facts,
    is_root: bool,
    hoist_static_vnodes: bool,
) -> bool {
    match fact.level {
        StaticLevel::FullyStatic => {
            if is_root && fact.props_hoistable {
                return true;
            }
            if hoist_static_vnodes {
                return true;
            }
            skip_region(walk, &element.children.ops);
            false
        }
        StaticLevel::HasDynamicText => {
            if is_root && fact.props_hoistable {
                return true;
            }
            skip_region(walk, &element.children.ops);
            false
        }
        StaticLevel::NotStatic => {
            if fact.props_hoistable && (fact.foreign || fact.nested_static) {
                return true;
            }
            let child_hoist_static_vnodes = hoist_static_vnodes || !element.bindings.is_empty();
            ensure_sufficient_stack(|| {
                region_has_legacy_hoist(
                    walk,
                    &element.children.ops,
                    facts,
                    false,
                    child_hoist_static_vnodes,
                )
            })
        }
    }
}

fn component_has_legacy_hoist(
    walk: &mut PageWalk,
    component: &ComponentOp<'_>,
    fact: Option<StaticFacts>,
    facts: &S2Facts,
) -> bool {
    if fact.is_some_and(|fact| fact.props_hoistable && (fact.foreign || fact.nested_static)) {
        return true;
    }
    ensure_sufficient_stack(|| {
        region_has_legacy_hoist(walk, &component.children.ops, facts, false, true)
    })
}

fn branch_roots_have_legacy_hoist(walk: &mut PageWalk, ops: &[Op<'_>], facts: &S2Facts) -> bool {
    for op in ops {
        let id = walk.mint();
        match op {
            Op::Element(element) => {
                walk.skip(element.bindings.len());
                if ensure_sufficient_stack(|| {
                    region_has_legacy_hoist(walk, &element.children.ops, facts, false, true)
                }) {
                    return true;
                }
            }
            Op::Component(component) => {
                walk.skip(component.bindings.len());
                let fact = id.and_then(|id| facts.static_facts.get(id)).copied();
                if component_has_legacy_hoist(walk, component, fact, facts) {
                    return true;
                }
            }
            _ => skip_op_after_mint(walk, op),
        }
    }
    false
}

fn for_children_have_legacy_hoist(walk: &mut PageWalk, ops: &[Op<'_>], facts: &S2Facts) -> bool {
    match ops {
        [Op::Element(element)] => {
            let id = walk.mint();
            walk.skip(element.bindings.len());
            if id
                .and_then(|id| facts.static_facts.get(id))
                .is_some_and(|fact| fact.props_hoistable)
            {
                return true;
            }
            ensure_sufficient_stack(|| {
                region_has_legacy_hoist(walk, &element.children.ops, facts, false, true)
            })
        }
        [Op::Component(component)] => {
            let id = walk.mint();
            walk.skip(component.bindings.len());
            if id
                .and_then(|id| facts.static_facts.get(id))
                .is_some_and(|fact| fact.props_hoistable)
            {
                return true;
            }
            ensure_sufficient_stack(|| {
                region_has_legacy_hoist(walk, &component.children.ops, facts, false, true)
            })
        }
        _ => region_has_legacy_hoist(walk, ops, facts, false, true),
    }
}

fn skip_region(walk: &mut PageWalk, ops: &[Op<'_>]) {
    for op in ops {
        let _id = walk.mint();
        skip_op_after_mint(walk, op);
    }
}

fn skip_op_after_mint(walk: &mut PageWalk, op: &Op<'_>) {
    match op {
        Op::Element(element) => {
            walk.skip(element.bindings.len());
            ensure_sufficient_stack(|| skip_region(walk, &element.children.ops));
        }
        Op::Component(component) => {
            walk.skip(component.bindings.len());
            ensure_sufficient_stack(|| skip_region(walk, &component.children.ops));
        }
        Op::If(if_op) => {
            for branch in if_op.branches.iter() {
                ensure_sufficient_stack(|| skip_region(walk, &branch.region.ops));
            }
        }
        Op::For(for_op) => ensure_sufficient_stack(|| skip_region(walk, &for_op.region.ops)),
        Op::Slot(slot) => {
            walk.skip(slot.bindings.len());
            ensure_sufficient_stack(|| skip_region(walk, &slot.fallback.ops));
        }
        Op::Text(_) | Op::Interpolation(_) => {}
    }
}
