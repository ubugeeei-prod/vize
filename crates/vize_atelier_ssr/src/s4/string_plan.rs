//! S2->S4 string-plan lowering for SSR.

mod binding;

use vize_s0::{Allocator, Span, Vec};
use vize_s2::op as s2;
use vize_s2_to_s3::{PartitionFacts, PartitionKind};

use binding::{binding_kind, binding_source, binding_span, slot_name};

/// SSR string plan emitted from S2 before JavaScript text generation.
#[derive(Debug)]
pub struct SsrStringPlan<'a> {
    /// Ordered plan segments. Control segments own no emitted bytes yet but pin
    /// where future SSR code generation must split runtime control flow.
    pub segments: Vec<'a, SsrStringSegment<'a>>,
    /// Summary derived from the consumed shared partition facts.
    pub partition: SsrPartitionSummary,
}

/// Static/dynamic segment counts observed while reading partition facts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SsrPartitionSummary {
    pub static_segments: u32,
    pub dynamic_segments: u32,
}

/// One planned SSR output/control segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SsrStringSegment<'a> {
    pub kind: SsrStringSegmentKind,
    pub partition: PartitionKind,
    pub span: Span,
    /// Borrowed payload for textual segments: tag names, text content, attribute
    /// names, or expression source depending on [`SsrStringSegmentKind`].
    pub source: Option<&'a str>,
}

/// The stable SSR S4 vocabulary for the first string-plan slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SsrStringSegmentKind {
    OpenElement,
    CloseElement,
    StaticAttribute,
    DynamicAttribute,
    Component,
    Text,
    DynamicText,
    RawHtml,
    Comment,
    If,
    For,
    SlotOutlet,
    Directive,
}

/// Result of lowering S2 into a string plan.
#[derive(Debug)]
pub struct SsrStringPlanLowering<'a> {
    pub plan: SsrStringPlan<'a>,
    pub errors: Vec<'a, SsrStringPlanError>,
}

/// String-plan validation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SsrStringPlanError {
    pub kind: SsrStringPlanErrorKind,
    pub fact_index: u32,
    pub expected_span: Span,
    pub actual_span: Option<Span>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SsrStringPlanErrorKind {
    MissingPartitionFact,
    PartitionSpanMismatch,
    ExtraPartitionFact,
}

/// Build the SSR S4 string plan from an S2 root and the shared S2->S3
/// partition facts. The fact stream is consumed in the S2 page-order that the
/// S3 lowering uses, so stale or mismatched facts become explicit errors.
#[must_use]
pub fn lower_s2_to_string_plan<'a>(
    allocator: &'a Allocator,
    root: &s2::Region<'a>,
    partition: &PartitionFacts<'_>,
) -> SsrStringPlanLowering<'a> {
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

struct Cx<'facts, 'a> {
    partition: &'facts PartitionFacts<'facts>,
    next_fact: usize,
    plan: SsrStringPlan<'a>,
    errors: Vec<'a, SsrStringPlanError>,
}

impl<'facts, 'a> Cx<'facts, 'a> {
    fn finish(mut self) -> SsrStringPlanLowering<'a> {
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

    fn lower_region(&mut self, region: &s2::Region<'a>) {
        for op in &region.ops {
            self.lower_op(op);
        }
    }

    fn lower_op(&mut self, op: &s2::Op<'a>) {
        match op {
            s2::Op::Element(element) => {
                let partition = self.consume_fact(element.span);
                self.push(
                    SsrStringSegmentKind::OpenElement,
                    partition,
                    element.span,
                    Some(element.tag),
                );
                for attr in &element.attributes {
                    self.push(
                        SsrStringSegmentKind::StaticAttribute,
                        partition,
                        attr.span,
                        Some(attr.name),
                    );
                }
                self.lower_bindings(&element.bindings);
                self.lower_region(&element.children);
                self.push(
                    SsrStringSegmentKind::CloseElement,
                    partition,
                    element.span,
                    Some(element.tag),
                );
            }
            s2::Op::Component(component) => {
                let partition = self.consume_fact(component.span);
                self.push(
                    SsrStringSegmentKind::Component,
                    partition,
                    component.span,
                    Some(component.name),
                );
                for attr in &component.attributes {
                    self.push(
                        SsrStringSegmentKind::StaticAttribute,
                        partition,
                        attr.span,
                        Some(attr.name),
                    );
                }
                self.lower_bindings(&component.bindings);
                self.lower_region(&component.children);
            }
            s2::Op::Text(text) => {
                let partition = self.consume_fact(text.span);
                self.push(
                    SsrStringSegmentKind::Text,
                    partition,
                    text.span,
                    Some(text.content),
                );
            }
            s2::Op::Interpolation(interpolation) => {
                let partition = self.consume_fact(interpolation.span);
                self.push(
                    SsrStringSegmentKind::DynamicText,
                    partition,
                    interpolation.span,
                    Some(interpolation.expression.source()),
                );
            }
            s2::Op::Comment(comment) => {
                let partition = self.consume_fact(comment.span);
                self.push(
                    SsrStringSegmentKind::Comment,
                    partition,
                    comment.span,
                    Some(comment.content),
                );
            }
            s2::Op::If(if_op) => {
                let partition = self.consume_fact(if_op.span);
                self.push(SsrStringSegmentKind::If, partition, if_op.span, None);
                for branch in &if_op.branches {
                    self.lower_region(&branch.region);
                }
            }
            s2::Op::For(for_op) => {
                let partition = self.consume_fact(for_op.span);
                self.push(
                    SsrStringSegmentKind::For,
                    partition,
                    for_op.span,
                    Some(for_op.binding.source.source()),
                );
                self.lower_region(&for_op.region);
            }
            s2::Op::Slot(slot) => {
                let partition = self.consume_fact(slot.span);
                self.push(
                    SsrStringSegmentKind::SlotOutlet,
                    partition,
                    slot.span,
                    slot_name(&slot.name),
                );
                for attr in &slot.attributes {
                    self.push(
                        SsrStringSegmentKind::StaticAttribute,
                        partition,
                        attr.span,
                        Some(attr.name),
                    );
                }
                self.lower_bindings(&slot.bindings);
                self.lower_region(&slot.fallback);
            }
        }
    }

    fn lower_bindings(&mut self, bindings: &[s2::BindingOp<'a>]) {
        for binding in bindings {
            let span = binding_span(binding);
            let partition = self.consume_fact(span);
            self.push(
                binding_kind(binding),
                partition,
                span,
                binding_source(binding),
            );
        }
    }

    fn consume_fact(&mut self, expected_span: Span) -> PartitionKind {
        let fact_index = self.next_fact;
        self.next_fact += 1;
        let Some(fact) = self.partition.ops.get(fact_index) else {
            self.errors.push(SsrStringPlanError {
                kind: SsrStringPlanErrorKind::MissingPartitionFact,
                fact_index: fact_index as u32,
                expected_span,
                actual_span: None,
            });
            return PartitionKind::Dynamic;
        };
        if fact.span != expected_span {
            self.errors.push(SsrStringPlanError {
                kind: SsrStringPlanErrorKind::PartitionSpanMismatch,
                fact_index: fact_index as u32,
                expected_span,
                actual_span: Some(fact.span),
            });
        }
        fact.kind
    }

    fn push(
        &mut self,
        kind: SsrStringSegmentKind,
        partition: PartitionKind,
        span: Span,
        source: Option<&'a str>,
    ) {
        if partition.is_dynamic() {
            self.plan.partition.dynamic_segments =
                self.plan.partition.dynamic_segments.saturating_add(1);
        } else {
            self.plan.partition.static_segments =
                self.plan.partition.static_segments.saturating_add(1);
        }
        self.plan.segments.push(SsrStringSegment {
            kind,
            partition,
            span,
            source,
        });
    }
}

#[cfg(test)]
mod tests;
