use vize_s0::{Allocator, Span, Vec};

use super::{EffectId, OpId, OpKind, Phase, Region, RegionId, StateEdge};

/// One flat Impeto operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Op {
    pub id: OpId,
    pub kind: OpKind,
    pub region: RegionId,
    pub effect: Option<EffectId>,
    pub span: Span,
}

impl Op {
    /// A new op in `region`.
    #[must_use]
    pub const fn new(id: OpId, kind: OpKind, region: RegionId, span: Span) -> Self {
        Self {
            id,
            kind,
            region,
            effect: None,
            span,
        }
    }

    /// Attach the op to an effect scope.
    #[must_use]
    pub const fn with_effect(mut self, effect: EffectId) -> Self {
        self.effect = Some(effect);
        self
    }
}

/// Effect-scope metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectScope {
    pub id: EffectId,
    pub owner: OpId,
    pub region: RegionId,
    pub span: Span,
}

/// One S3 program artifact.
#[derive(Debug)]
pub struct Program<'a> {
    pub phase: Phase,
    pub regions: Vec<'a, Region>,
    pub ops: Vec<'a, Op>,
    pub edges: Vec<'a, StateEdge>,
    pub effects: Vec<'a, EffectScope>,
}

impl<'a> Program<'a> {
    /// Empty program for `phase`.
    #[must_use]
    pub fn new(allocator: &'a Allocator, phase: Phase) -> Self {
        Self {
            phase,
            regions: Vec::new_in(&allocator),
            ops: Vec::new_in(&allocator),
            edges: Vec::new_in(&allocator),
            effects: Vec::new_in(&allocator),
        }
    }

    pub fn push_region(&mut self, region: Region) {
        self.regions.push(region);
    }

    pub fn push_op(&mut self, op: Op) {
        self.ops.push(op);
    }

    pub fn push_edge(&mut self, edge: StateEdge) {
        self.edges.push(edge);
    }

    pub fn push_effect(&mut self, effect: EffectScope) {
        self.effects.push(effect);
    }
}
