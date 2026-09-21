//! The S2 walk that consumes partition facts in page order.

use vize_s0::{Allocator, Span, Vec};
use vize_s2::op as s2;
use vize_s2_to_s3::{PartitionFacts, PartitionKind};

use super::binding::{binding_kind, binding_payload, binding_span};
use super::payload::slot_name_payload;
use super::{
    SsrPartitionSummary, SsrSegmentSource as Source, SsrStringPayload, SsrStringPayloadKind,
    SsrStringPlan, SsrStringPlanError, SsrStringPlanErrorKind, SsrStringPlanLowering,
    SsrStringSegment, SsrStringSegmentKind as Kind,
};

pub(super) fn lower<'r, 'a>(
    allocator: &'a Allocator,
    root: &'r s2::Region<'a>,
    partition: &PartitionFacts<'_>,
) -> SsrStringPlanLowering<'r, 'a> {
    let mut cx = Cx {
        partition,
        next_fact: 0,
        plan: SsrStringPlan {
            segments: Vec::new_in(&allocator),
            partition: SsrPartitionSummary::default(),
        },
        errors: Vec::new_in(&allocator),
    };
    cx.lower_region(root);
    cx.finish()
}

struct Cx<'facts, 'r, 'a> {
    partition: &'facts PartitionFacts<'facts>,
    next_fact: usize,
    plan: SsrStringPlan<'r, 'a>,
    errors: Vec<'a, SsrStringPlanError>,
}

/// One segment's plan data before it is counted and pushed.
struct Planned<'r, 'a> {
    kind: Kind,
    partition: PartitionKind,
    span: Span,
    fact: u32,
    payload: Option<SsrStringPayload<'a>>,
    source: Source<'r, 'a>,
}

impl<'facts, 'r, 'a> Cx<'facts, 'r, 'a> {
    fn finish(mut self) -> SsrStringPlanLowering<'r, 'a> {
        while let Some(extra) = self.partition.ops.get(self.next_fact) {
            self.errors.push(SsrStringPlanError {
                kind: SsrStringPlanErrorKind::ExtraPartitionFact,
                fact_index: self.next_fact as u32,
                expected_span: extra.span,
                actual_span: Some(extra.span),
            });
            self.next_fact += 1;
        }
        SsrStringPlanLowering {
            plan: self.plan,
            errors: self.errors,
        }
    }

    fn lower_region(&mut self, region: &'r s2::Region<'a>) {
        for op in &region.ops {
            self.lower_op(op);
        }
    }

    fn lower_op(&mut self, op: &'r s2::Op<'a>) {
        match op {
            s2::Op::Element(element) => {
                let element: &'r s2::ElementOp<'a> = element;
                let (partition, fact) = self.consume_fact(element.span);
                let tag = Some(payload(SsrStringPayloadKind::TagName, element.tag));
                let source = Source::Element(element);
                self.push(
                    Kind::OpenElement,
                    partition,
                    element.span,
                    fact,
                    tag,
                    source,
                );
                self.lower_attached(&element.attributes, &element.bindings, partition, fact);
                self.lower_region(&element.children);
                self.push(
                    Kind::CloseElement,
                    partition,
                    element.span,
                    fact,
                    tag,
                    source,
                );
            }
            s2::Op::Component(component) => {
                let component: &'r s2::ComponentOp<'a> = component;
                let (partition, fact) = self.consume_fact(component.span);
                let name = Some(payload(SsrStringPayloadKind::ComponentName, component.name));
                let source = Source::Component(component);
                self.push(
                    Kind::Component,
                    partition,
                    component.span,
                    fact,
                    name,
                    source,
                );
                self.lower_attached(&component.attributes, &component.bindings, partition, fact);
                self.lower_region(&component.children);
                self.push(
                    Kind::CloseComponent,
                    partition,
                    component.span,
                    fact,
                    name,
                    source,
                );
            }
            s2::Op::Text(text) => {
                let (partition, fact) = self.consume_fact(text.span);
                let content = Some(payload(SsrStringPayloadKind::Text, text.content));
                self.push(
                    Kind::Text,
                    partition,
                    text.span,
                    fact,
                    content,
                    Source::Text(text),
                );
            }
            s2::Op::Interpolation(interpolation) => {
                let (partition, fact) = self.consume_fact(interpolation.span);
                let expression = interpolation.expression.source();
                self.push(
                    Kind::DynamicText,
                    partition,
                    interpolation.span,
                    fact,
                    Some(payload(SsrStringPayloadKind::Expression, expression)),
                    Source::Interpolation(interpolation),
                );
            }
            s2::Op::Comment(comment) => {
                let (partition, fact) = self.consume_fact(comment.span);
                let content = Some(payload(SsrStringPayloadKind::Comment, comment.content));
                let source = Source::Comment(comment);
                self.push(
                    Kind::Comment,
                    partition,
                    comment.span,
                    fact,
                    content,
                    source,
                );
            }
            s2::Op::If(if_op) => {
                let if_op: &'r s2::IfOp<'a> = if_op;
                let (partition, fact) = self.consume_fact(if_op.span);
                let source = Source::If(if_op);
                self.push(Kind::If, partition, if_op.span, fact, None, source);
                // Branches are regions, not ops: they read the chain's fact.
                for branch in &if_op.branches {
                    let branch_source = Source::Branch(branch);
                    self.push(
                        Kind::Branch,
                        partition,
                        branch.span,
                        fact,
                        None,
                        branch_source,
                    );
                    self.lower_region(&branch.region);
                    self.push(
                        Kind::CloseBranch,
                        partition,
                        branch.span,
                        fact,
                        None,
                        branch_source,
                    );
                }
                self.push(Kind::CloseIf, partition, if_op.span, fact, None, source);
            }
            s2::Op::For(for_op) => {
                let for_op: &'r s2::ForOp<'a> = for_op;
                let (partition, fact) = self.consume_fact(for_op.span);
                let binding = Some(payload(
                    SsrStringPayloadKind::ForBinding,
                    for_op.binding.source.source(),
                ));
                let source = Source::For(for_op);
                self.push(Kind::For, partition, for_op.span, fact, binding, source);
                self.lower_region(&for_op.region);
                self.push(
                    Kind::CloseFor,
                    partition,
                    for_op.span,
                    fact,
                    binding,
                    source,
                );
            }
            s2::Op::Slot(slot) => {
                let slot: &'r s2::SlotOp<'a> = slot;
                let (partition, fact) = self.consume_fact(slot.span);
                let name = slot_name_payload(&slot.name);
                let source = Source::Slot(slot);
                self.push(Kind::SlotOutlet, partition, slot.span, fact, name, source);
                self.lower_attached(&slot.attributes, &slot.bindings, partition, fact);
                self.lower_region(&slot.fallback);
                self.push(Kind::CloseSlot, partition, slot.span, fact, name, source);
            }
        }
    }

    /// Static attributes and attached bindings, interleaved in authored
    /// order. Bindings consume their own facts as they are reached.
    fn lower_attached(
        &mut self,
        attributes: &'r [s2::Attribute<'a>],
        bindings: &'r [s2::BindingOp<'a>],
        owner_partition: PartitionKind,
        owner_fact: u32,
    ) {
        let (mut attrs, mut binds) = (attributes.iter().peekable(), bindings.iter().peekable());
        loop {
            let attr_first = match (attrs.peek(), binds.peek()) {
                (Some(attr), Some(binding)) => attr.span.start <= binding_span(binding).start,
                (Some(_), None) => true,
                (None, Some(_)) => false,
                (None, None) => break,
            };
            if attr_first {
                let Some(attr) = attrs.next() else { break };
                let name = Some(payload(SsrStringPayloadKind::AttributeName, attr.name));
                self.push_planned(Planned {
                    kind: Kind::StaticAttribute,
                    partition: owner_partition,
                    span: attr.span,
                    fact: owner_fact,
                    payload: name,
                    source: Source::Attribute(attr),
                });
            } else {
                let Some(binding) = binds.next() else { break };
                let span = binding_span(binding);
                let (partition, fact) = self.consume_fact(span);
                self.push_planned(Planned {
                    kind: binding_kind(binding),
                    partition,
                    span,
                    fact,
                    payload: binding_payload(binding),
                    source: Source::Binding(binding),
                });
            }
        }
    }

    fn consume_fact(&mut self, expected_span: Span) -> (PartitionKind, u32) {
        let fact_index = self.next_fact;
        self.next_fact += 1;
        let Some(fact) = self.partition.ops.get(fact_index) else {
            self.errors.push(SsrStringPlanError {
                kind: SsrStringPlanErrorKind::MissingPartitionFact,
                fact_index: fact_index as u32,
                expected_span,
                actual_span: None,
            });
            return (PartitionKind::Dynamic, fact_index as u32);
        };
        if fact.span != expected_span {
            self.errors.push(SsrStringPlanError {
                kind: SsrStringPlanErrorKind::PartitionSpanMismatch,
                fact_index: fact_index as u32,
                expected_span,
                actual_span: Some(fact.span),
            });
        }
        (fact.kind, fact_index as u32)
    }

    fn push(
        &mut self,
        kind: Kind,
        partition: PartitionKind,
        span: Span,
        fact: u32,
        payload: Option<SsrStringPayload<'a>>,
        source: Source<'r, 'a>,
    ) {
        self.push_planned(Planned {
            kind,
            partition,
            span,
            fact,
            payload,
            source,
        });
    }

    fn push_planned(&mut self, planned: Planned<'r, 'a>) {
        let summary = &mut self.plan.partition;
        if planned.partition.is_dynamic() {
            summary.dynamic_segments = summary.dynamic_segments.saturating_add(1);
        } else {
            summary.static_segments = summary.static_segments.saturating_add(1);
        }
        self.plan.segments.push(SsrStringSegment {
            kind: planned.kind,
            partition: planned.partition,
            span: planned.span,
            fact: planned.fact,
            payload: planned.payload,
            source: planned.source,
        });
    }
}

const fn payload(kind: SsrStringPayloadKind, source: &str) -> SsrStringPayload<'_> {
    SsrStringPayload::new(kind, source)
}
