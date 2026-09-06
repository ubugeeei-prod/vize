use vize_s0::ensure_sufficient_stack;
use vize_s2::op::Op;

use crate::pass::walk::PageWalk;
use crate::pass::{S2Facts, StaticLevel};

pub(super) fn region_has_non_branch_foreign_props_hoist(
    walk: &mut PageWalk,
    ops: &[Op<'_>],
    facts: &S2Facts,
    branch_root: bool,
) -> bool {
    for op in ops {
        let id = walk.mint();
        match op {
            Op::Element(element) => {
                walk.skip(element.bindings.len());
                if !branch_root
                    && id
                        .and_then(|id| facts.static_facts.get(id))
                        .is_some_and(|fact| {
                            fact.props_hoistable
                                && fact.foreign
                                && fact.level == StaticLevel::NotStatic
                        })
                {
                    super::skip_region(walk, &element.children.ops);
                    return true;
                }
                if ensure_sufficient_stack(|| {
                    region_has_non_branch_foreign_props_hoist(
                        walk,
                        &element.children.ops,
                        facts,
                        false,
                    )
                }) {
                    return true;
                }
            }
            Op::Component(component) => {
                walk.skip(component.bindings.len());
                if ensure_sufficient_stack(|| {
                    region_has_non_branch_foreign_props_hoist(
                        walk,
                        &component.children.ops,
                        facts,
                        false,
                    )
                }) {
                    return true;
                }
            }
            Op::If(if_op) => {
                for branch in if_op.branches.iter() {
                    if ensure_sufficient_stack(|| {
                        region_has_non_branch_foreign_props_hoist(
                            walk,
                            &branch.region.ops,
                            facts,
                            true,
                        )
                    }) {
                        return true;
                    }
                }
            }
            Op::For(for_op) => {
                if ensure_sufficient_stack(|| {
                    region_has_non_branch_foreign_props_hoist(
                        walk,
                        &for_op.region.ops,
                        facts,
                        false,
                    )
                }) {
                    return true;
                }
            }
            Op::Slot(slot) => {
                walk.skip(slot.bindings.len());
                if ensure_sufficient_stack(|| {
                    region_has_non_branch_foreign_props_hoist(
                        walk,
                        &slot.fallback.ops,
                        facts,
                        false,
                    )
                }) {
                    return true;
                }
            }
            Op::Text(_) | Op::Interpolation(_) | Op::Comment(_) => {}
        }
    }
    false
}
