//! S2->S4 string-plan lowering for SSR.

mod binding;
mod lower;
mod payload;

use vize_s0::{Allocator, Span, Vec};
use vize_s2::op as s2;
use vize_s2_to_s3::{PartitionFacts, PartitionKind};

pub use payload::{SsrStringPayload, SsrStringPayloadKind};

/// SSR string plan emitted from S2 before JavaScript text generation.
#[derive(Debug)]
pub struct SsrStringPlan<'r, 'a> {
    /// Ordered plan segments. Control segments own no emitted bytes yet but pin
    /// where future SSR code generation must split runtime control flow.
    pub segments: Vec<'a, SsrStringSegment<'r, 'a>>,
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
///
/// Segments are in S2 page order, except that an element's static
/// attributes and attached bindings interleave in authored order (the order
/// SSR attribute output follows). Binding facts are still consumed in page
/// order because both lists keep their own relative order in the merge.
#[derive(Debug, Clone, Copy)]
pub struct SsrStringSegment<'r, 'a> {
    pub kind: SsrStringSegmentKind,
    pub partition: PartitionKind,
    pub span: Span,
    /// Index of the partition fact this segment read: the owning op's S2
    /// page-order id (attributes and close segments read their owner's).
    pub fact: u32,
    /// Borrowed payload for textual segments, typed before code generation so
    /// callers do not need to infer meaning from [`SsrStringSegmentKind`].
    pub payload: Option<SsrStringPayload<'a>>,
    /// The S2 construct this segment plans, for emitters that need the whole
    /// typed operand (attribute values, binding names, retained expressions).
    pub source: SsrSegmentSource<'r, 'a>,
}

/// The S2 construct behind one plan segment.
///
/// The component, comment, control-flow, and slot operands are planned for
/// the emitter slices that own those shapes; until then the emitter refuses
/// them by segment kind without reading the operand.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum SsrSegmentSource<'r, 'a> {
    Element(&'r s2::ElementOp<'a>),
    Component(&'r s2::ComponentOp<'a>),
    Attribute(&'r s2::Attribute<'a>),
    Binding(&'r s2::BindingOp<'a>),
    Text(&'r s2::TextOp<'a>),
    Interpolation(&'r s2::InterpolationOp<'a>),
    Comment(&'r s2::CommentOp<'a>),
    If(&'r s2::IfOp<'a>),
    Branch(&'r s2::IfBranch<'a>),
    For(&'r s2::ForOp<'a>),
    Slot(&'r s2::SlotOp<'a>),
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
    /// Opens one `v-if` branch region (branches read their chain's fact).
    Branch,
    CloseBranch,
    CloseIf,
    For,
    CloseFor,
    SlotOutlet,
    Directive,
}

/// Result of lowering S2 into a string plan.
#[derive(Debug)]
pub struct SsrStringPlanLowering<'r, 'a> {
    pub plan: SsrStringPlan<'r, 'a>,
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
pub fn lower_s2_to_string_plan<'r, 'a>(
    allocator: &'a Allocator,
    root: &'r s2::Region<'a>,
    partition: &PartitionFacts<'_>,
) -> SsrStringPlanLowering<'r, 'a> {
    lower::lower(allocator, root, partition)
}

#[cfg(test)]
mod tests;
