use vize_s0::{Allocator, Span, ensure_sufficient_stack};
use vize_s2::op as s2;
use vize_s3::op::{
    EdgeKind, EffectId, EffectScope, Op, OpId, OpKind, Phase, Program, Region, RegionId, StateEdge,
};

use crate::{PartitionFact, PartitionFacts, PartitionKind};

/// Result of one S2→S3 lowering.
#[derive(Debug)]
pub struct Lowered<'a> {
    /// Flat S3 program in the `built` phase.
    pub program: Program<'a>,
    /// Static/dynamic partition facts exported beside the program.
    pub partition: PartitionFacts<'a>,
}

/// Lower an S2 root region to an S3 program plus shared partition facts.
#[must_use]
pub fn lower<'a>(allocator: &'a Allocator, root: &s2::Region<'_>) -> Lowered<'a> {
    ensure_sufficient_stack(|| Cx::new(allocator).finish(root))
}

struct Cx<'a> {
    program: Program<'a>,
    partition: PartitionFacts<'a>,
    next_op: u32,
    next_region: u32,
    next_effect: u32,
    last_effectful: Option<OpId>,
}

impl<'a> Cx<'a> {
    fn new(allocator: &'a Allocator) -> Self {
        Self {
            program: Program::new(allocator, Phase::Built),
            partition: PartitionFacts::new(allocator),
            next_op: 0,
            next_region: 0,
            next_effect: 0,
            last_effectful: None,
        }
    }

    fn finish(mut self, root: &s2::Region<'_>) -> Lowered<'a> {
        let root_span = region_span(root, Span::new(0, 0));
        let root_id = self.mint_region();
        debug_assert_eq!(root_id, RegionId::ROOT);
        self.program.push_region(Region::root(root_span));
        self.lower_ops(root, root_id, PartitionKind::Static);
        Lowered {
            program: self.program,
            partition: self.partition,
        }
    }

    fn lower_region(
        &mut self,
        region: &s2::Region<'_>,
        parent: RegionId,
        owner: OpId,
        fallback_span: Span,
        inherited: PartitionKind,
    ) {
        let id = self.mint_region();
        let span = region_span(region, fallback_span);
        self.program
            .push_region(Region::child(id, parent, owner, span));
        self.lower_ops(region, id, inherited);
    }

    fn lower_ops(
        &mut self,
        region: &s2::Region<'_>,
        target_region: RegionId,
        inherited: PartitionKind,
    ) {
        ensure_sufficient_stack(|| {
            let mut previous = None;
            for op in &region.ops {
                let id = self.lower_op(op, target_region, inherited);
                if let Some(from) = previous {
                    self.program
                        .push_edge(StateEdge::new(from, id, EdgeKind::DomOrder));
                }
                previous = Some(id);
            }
        });
    }

    fn lower_op(
        &mut self,
        op: &s2::Op<'_>,
        target_region: RegionId,
        inherited: PartitionKind,
    ) -> OpId {
        match op {
            s2::Op::Element(element) => {
                let kind = inherited.join(PartitionKind::Static);
                let id = self.push_op(OpKind::InsertNode, target_region, element.span, kind);
                self.lower_bindings(&element.bindings, target_region, inherited);
                self.lower_region(
                    &element.children,
                    target_region,
                    id,
                    element.span,
                    inherited,
                );
                id
            }
            s2::Op::Component(component) => {
                let kind = inherited.join(PartitionKind::Dynamic);
                let id = self.push_op(OpKind::CreateComponent, target_region, component.span, kind);
                self.lower_bindings(&component.bindings, target_region, kind);
                self.lower_region(&component.children, target_region, id, component.span, kind);
                id
            }
            s2::Op::Text(text) => {
                self.push_op(OpKind::SetText, target_region, text.span, inherited)
            }
            s2::Op::Interpolation(interpolation) => self.push_op(
                OpKind::SetText,
                target_region,
                interpolation.span,
                PartitionKind::Dynamic,
            ),
            s2::Op::Comment(comment) => {
                self.push_op(OpKind::InsertNode, target_region, comment.span, inherited)
            }
            s2::Op::If(if_op) => {
                let id = self.push_op(
                    OpKind::If,
                    target_region,
                    if_op.span,
                    PartitionKind::Dynamic,
                );
                for branch in &if_op.branches {
                    self.lower_region(
                        &branch.region,
                        target_region,
                        id,
                        branch.span,
                        PartitionKind::Dynamic,
                    );
                }
                id
            }
            s2::Op::For(for_op) => {
                let id = self.push_op(
                    OpKind::For,
                    target_region,
                    for_op.span,
                    PartitionKind::Dynamic,
                );
                self.lower_region(
                    &for_op.region,
                    target_region,
                    id,
                    for_op.span,
                    PartitionKind::Dynamic,
                );
                id
            }
            s2::Op::Slot(slot) => {
                let id = self.push_op(
                    OpKind::SlotOutlet,
                    target_region,
                    slot.span,
                    PartitionKind::Dynamic,
                );
                self.lower_bindings(&slot.bindings, target_region, PartitionKind::Dynamic);
                self.lower_region(
                    &slot.fallback,
                    target_region,
                    id,
                    slot.span,
                    PartitionKind::Dynamic,
                );
                id
            }
        }
    }

    fn lower_bindings(
        &mut self,
        bindings: &[s2::BindingOp<'_>],
        target_region: RegionId,
        inherited: PartitionKind,
    ) {
        let mut previous = None;
        for binding in bindings {
            let span = binding_span(binding);
            let partition = inherited.join(binding_partition(binding));
            let id = self.push_op(binding_kind(binding), target_region, span, partition);
            if let Some(from) = previous {
                self.program
                    .push_edge(StateEdge::new(from, id, EdgeKind::DomOrder));
            }
            previous = Some(id);
        }
    }

    fn push_op(
        &mut self,
        kind: OpKind,
        region: RegionId,
        span: Span,
        partition: PartitionKind,
    ) -> OpId {
        let id = self.mint_op();
        let mut op = Op::new(id, kind, region, span);
        if partition.is_dynamic() {
            let effect = self.mint_effect();
            op = op.with_effect(effect);
            self.program.push_effect(EffectScope {
                id: effect,
                owner: id,
                region,
                span,
            });
            if let Some(previous) = self.last_effectful {
                self.program
                    .push_edge(StateEdge::new(previous, id, EdgeKind::EffectOrder));
            }
            self.last_effectful = Some(id);
        }
        self.program.push_op(op);
        self.partition.push(PartitionFact {
            op: id,
            kind: partition,
            span,
        });
        id
    }

    fn mint_op(&mut self) -> OpId {
        let id = OpId::new(self.next_op);
        self.next_op = self.next_op.saturating_add(1);
        id
    }

    fn mint_region(&mut self) -> RegionId {
        let id = RegionId::new(self.next_region);
        self.next_region = self.next_region.saturating_add(1);
        id
    }

    fn mint_effect(&mut self) -> EffectId {
        let id = EffectId::new(self.next_effect);
        self.next_effect = self.next_effect.saturating_add(1);
        id
    }
}

fn binding_kind(binding: &s2::BindingOp<'_>) -> OpKind {
    match binding {
        s2::BindingOp::Bind(bind) if bind.name.is_none() => OpKind::SetDynamicProps,
        s2::BindingOp::Bind(_) => OpKind::SetProp,
        s2::BindingOp::On(_) => OpKind::SetEvent,
        s2::BindingOp::Model(_) | s2::BindingOp::VueSync(_) => OpKind::SetProp,
        s2::BindingOp::SlotContent(_) | s2::BindingOp::VueSlotScope(_) => OpKind::SlotOutlet,
        s2::BindingOp::VueDirective(_)
        | s2::BindingOp::VueOnce(_)
        | s2::BindingOp::VueMemo(_)
        | s2::BindingOp::VueShow(_)
        | s2::BindingOp::VueCloak(_) => OpKind::Directive,
        s2::BindingOp::VueCssBind(_) => OpKind::SetDynamicProps,
        s2::BindingOp::VueHtml(_) => OpKind::SetHtml,
        s2::BindingOp::VueText(_) => OpKind::SetText,
    }
}

fn binding_partition(binding: &s2::BindingOp<'_>) -> PartitionKind {
    match binding {
        s2::BindingOp::VueOnce(_) | s2::BindingOp::VueCloak(_) => PartitionKind::Static,
        _ => PartitionKind::Dynamic,
    }
}

fn binding_span(binding: &s2::BindingOp<'_>) -> Span {
    match binding {
        s2::BindingOp::Bind(op) => op.span,
        s2::BindingOp::On(op) => op.span,
        s2::BindingOp::Model(op) => op.span,
        s2::BindingOp::SlotContent(op) => op.span,
        s2::BindingOp::VueDirective(op) => op.span,
        s2::BindingOp::VueCssBind(op) => op.span,
        s2::BindingOp::VueSync(op) => op.span,
        s2::BindingOp::VueSlotScope(op) => op.span,
        s2::BindingOp::VueOnce(op) => op.span,
        s2::BindingOp::VueMemo(op) => op.span,
        s2::BindingOp::VueShow(op) => op.span,
        s2::BindingOp::VueHtml(op) => op.span,
        s2::BindingOp::VueText(op) => op.span,
        s2::BindingOp::VueCloak(op) => op.span,
    }
}

fn region_span(region: &s2::Region<'_>, fallback: Span) -> Span {
    let mut span = None;
    for op in &region.ops {
        span = Some(match span {
            Some(current) => union_span(current, op_span(op)),
            None => op_span(op),
        });
    }
    span.unwrap_or(fallback)
}

fn op_span(op: &s2::Op<'_>) -> Span {
    match op {
        s2::Op::Element(op) => op.span,
        s2::Op::Component(op) => op.span,
        s2::Op::Text(op) => op.span,
        s2::Op::Interpolation(op) => op.span,
        s2::Op::Comment(op) => op.span,
        s2::Op::If(op) => op.span,
        s2::Op::For(op) => op.span,
        s2::Op::Slot(op) => op.span,
    }
}

fn union_span(left: Span, right: Span) -> Span {
    Span::new(left.start.min(right.start), left.end.max(right.end))
}
