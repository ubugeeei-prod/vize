use vize_s0::Span;

use crate::op::{EffectId, EffectScope, Op, OpId, Program, Region, RegionId};

pub(super) fn contains(owner: Span, child: Span) -> bool {
    owner.start <= child.start && child.end <= owner.end
}

/// The program with each table's density decided once. When every id equals
/// its position (as the S3 producers mint them), ids are unique and the first
/// record with an id is the one at that index; otherwise lookups scan for the
/// first record, exactly as before.
pub(super) struct Tables<'p, 'a> {
    program: &'p Program<'a>,
    pub(super) dense_ops: bool,
    pub(super) dense_regions: bool,
    pub(super) dense_effects: bool,
}

impl<'p, 'a> Tables<'p, 'a> {
    pub(super) fn new(program: &'p Program<'a>) -> Self {
        Self {
            program,
            dense_ops: (program.ops.iter().enumerate()).all(|(i, op)| op.id.index() as usize == i),
            dense_regions: (program.regions.iter().enumerate())
                .all(|(i, region)| region.id.index() as usize == i),
            dense_effects: (program.effects.iter().enumerate())
                .all(|(i, effect)| effect.id.index() as usize == i),
        }
    }
}

impl<'a> core::ops::Deref for Tables<'_, 'a> {
    type Target = Program<'a>;

    fn deref(&self) -> &Program<'a> {
        self.program
    }
}

pub(super) fn op<'t>(program: &'t Tables<'_, '_>, id: OpId) -> Option<&'t Op> {
    program.ops.get(op_index(program, id)?)
}

pub(super) fn op_index(program: &Tables<'_, '_>, id: OpId) -> Option<usize> {
    if program.dense_ops {
        let index = id.index() as usize;
        return (index < program.ops.len()).then_some(index);
    }
    program.ops.iter().position(|op| op.id == id)
}

pub(super) fn region<'t>(program: &'t Tables<'_, '_>, id: RegionId) -> Option<&'t Region> {
    if program.dense_regions {
        return program.regions.get(id.index() as usize);
    }
    program.regions.iter().find(|region| region.id == id)
}

pub(super) fn effect_scope<'t>(
    program: &'t Tables<'_, '_>,
    id: EffectId,
) -> Option<&'t EffectScope> {
    if program.dense_effects {
        return program.effects.get(id.index() as usize);
    }
    program.effects.iter().find(|effect| effect.id == id)
}

pub(super) fn region_has_cycle(program: &Tables<'_, '_>, start: RegionId) -> bool {
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
    program: &Tables<'_, '_>,
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

pub(super) fn op_inside_region(program: &Tables<'_, '_>, id: OpId, ancestor: RegionId) -> bool {
    op(program, id)
        .map(|op| region_is_or_descendant(program, op.region, ancestor))
        .unwrap_or(false)
}
