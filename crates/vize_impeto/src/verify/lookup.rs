use vize_s0::Span;

use crate::op::{EffectId, EffectScope, Op, OpId, Program, Region, RegionId};

pub(super) fn contains(owner: Span, child: Span) -> bool {
    owner.start <= child.start && child.end <= owner.end
}

pub(super) fn op<'a>(program: &'a Program<'_>, id: OpId) -> Option<&'a Op> {
    program.ops.iter().find(|op| op.id == id)
}

pub(super) fn op_index(program: &Program<'_>, id: OpId) -> Option<usize> {
    program.ops.iter().position(|op| op.id == id)
}

pub(super) fn region<'a>(program: &'a Program<'_>, id: RegionId) -> Option<&'a Region> {
    program.regions.iter().find(|region| region.id == id)
}

pub(super) fn effect_scope<'a>(program: &'a Program<'_>, id: EffectId) -> Option<&'a EffectScope> {
    program.effects.iter().find(|effect| effect.id == id)
}

pub(super) fn region_has_cycle(program: &Program<'_>, start: RegionId) -> bool {
    let mut current = region(program, start).and_then(|region| region.parent);
    let mut steps = 0usize;
    while let Some(id) = current {
        if id == start {
            return true;
        }
        steps += 1;
        if steps > program.regions.len() {
            return true;
        }
        current = region(program, id).and_then(|region| region.parent);
    }
    false
}

pub(super) fn region_is_or_descendant(
    program: &Program<'_>,
    child: RegionId,
    ancestor: RegionId,
) -> bool {
    let mut current = Some(child);
    let mut steps = 0usize;
    while let Some(id) = current {
        if id == ancestor {
            return true;
        }
        steps += 1;
        if steps > program.regions.len() {
            return false;
        }
        current = region(program, id).and_then(|region| region.parent);
    }
    false
}

pub(super) fn op_inside_region(program: &Program<'_>, id: OpId, ancestor: RegionId) -> bool {
    op(program, id)
        .map(|op| region_is_or_descendant(program, op.region, ancestor))
        .unwrap_or(false)
}
