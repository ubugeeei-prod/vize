//! SSR code generation from the S4 string plan.
//!
//! The emitter consumes [`SsrStringPlan`] segments in order and writes the
//! `ssrRender` body through the shared SSR output writer. It never reads the
//! legacy template AST. Every shape outside the admitted surface returns a
//! typed legacy reason before any byte is committed to the caller; every
//! broken plan invariant (stale facts, misclassified partitions, unbalanced
//! region segments) is a rejection, never a guess.

mod attrs;
mod component;
mod component_props;
mod control;
mod create_slots;
mod element;
mod fallthrough;
mod model;
mod region;
mod slot_outlet;
mod slots;
mod text;
mod vnode;
mod vnode_control;
mod vnode_create_slots;
mod vnode_props;

use vize_davinci::side_table::SideTable;
use vize_s0::{FxHashSet, String};
use vize_s1_to_s2::lower::{ForWrapper, IfFacts, TextParts, WrapperKeys};
use vize_s1_to_s2::{TransformContent, TransformExpressions};
use vize_s2::expr::ExprRef;
use vize_s2_to_s3::PartitionKind;

use super::string_plan::{
    SsrSegmentSource as Source, SsrStringPayloadKind, SsrStringPlan, SsrStringSegment,
    SsrStringSegmentKind as Kind,
};
use super::{AdmissionFailure, LegacyReason};
use crate::codegen::SsrCodegenContext;
use crate::codegen::scope_prefix::strip_scope_prefixes_for_scoped_params;
use region::is_close;

type Result<T> = core::result::Result<T, AdmissionFailure>;

/// The S2 side facts the emitter reads beside the plan.
pub(super) struct PlanFacts<'s> {
    pub(super) texts: &'s SideTable<TextParts>,
    pub(super) for_wrappers: &'s SideTable<ForWrapper>,
    pub(super) wrappers: &'s SideTable<WrapperKeys>,
    pub(super) if_facts: &'s SideTable<IfFacts>,
}

/// Write the render body for `plan` into `ctx`.
pub(super) fn emit_plan(
    ctx: &mut SsrCodegenContext<'_>,
    plan: &SsrStringPlan<'_, '_>,
    facts: &PlanFacts<'_>,
    exprs: &mut TransformExpressions<'_>,
) -> Result<()> {
    let mut emitter = Emitter {
        segments: &plan.segments,
        pos: 0,
        ctx,
        facts,
        exprs,
        scoped_params: std::vec::Vec::new(),
        select_models: std::vec::Vec::new(),
        branch_key: None,
    };
    let root = emitter.scan_region(0)?;
    let fragment = root.legacy_children > 1 && root.non_text;
    emitter.children(Flags {
        as_fragment: fragment,
        disable_nested_fragments: false,
        inherit_attrs: true,
    })?;
    if emitter.pos != emitter.segments.len() {
        return Err(AdmissionFailure::Invalid(
            "string plan has unbalanced segments",
        ));
    }
    Ok(())
}

struct Emitter<'p, 'r, 'a, 'c, 'x, 'e> {
    segments: &'p [SsrStringSegment<'r, 'a>],
    pos: usize,
    ctx: &'c mut SsrCodegenContext<'x>,
    facts: &'p PlanFacts<'p>,
    exprs: &'p mut TransformExpressions<'e>,
    /// The legacy walker's codegen scope: `v-for` / slot destructure params
    /// whose transform-applied prefixes are stripped at emission.
    scoped_params: std::vec::Vec<FxHashSet<String>>,
    /// `v-model` reads of the open `<select>` ancestors.
    select_models: std::vec::Vec<String>,
    /// The `v-if` branch key the legacy transform moved off the branch root:
    /// the next element or component skips the attribute at this span.
    branch_key: Option<vize_s0::Span>,
}

/// How the legacy walker visits one child list.
#[derive(Clone, Copy)]
struct Flags {
    as_fragment: bool,
    disable_nested_fragments: bool,
    inherit_attrs: bool,
}

const PLAIN: Flags = Flags {
    as_fragment: false,
    disable_nested_fragments: false,
    inherit_attrs: false,
};

impl<'r, 'a> Emitter<'_, 'r, 'a, '_, '_, '_> {
    /// `process_children_with_fallthrough_attrs` over the plan: emit the
    /// child list at the cursor up to (not including) its closing segment.
    fn children(&mut self, flags: Flags) -> Result<()> {
        let shape = self.scan_region(self.pos)?;
        let fallthrough = match shape.candidates.as_slice() {
            [only] if flags.inherit_attrs && !flags.as_fragment => Some(*only),
            _ => None,
        };
        if flags.as_fragment {
            self.ctx.push_string_part_static("<!--[-->");
        }
        while let Some(segment) = self.segments.get(self.pos).copied() {
            if is_close(segment.kind) {
                break;
            }
            let inherit = fallthrough == Some(self.pos);
            self.child(segment, flags, inherit)?;
        }
        if flags.as_fragment {
            self.ctx.push_string_part_static("<!--]-->");
        }
        Ok(())
    }

    /// `process_child` for the child at the cursor.
    fn child(
        &mut self,
        segment: SsrStringSegment<'r, 'a>,
        flags: Flags,
        inherit: bool,
    ) -> Result<()> {
        let disable = flags.disable_nested_fragments;
        match (segment.kind, segment.source) {
            (Kind::OpenElement, Source::Element(element)) => {
                vize_s0::ensure_sufficient_stack(|| self.element(segment, element, inherit))
            }
            (Kind::Component, Source::Component(component)) => {
                vize_s0::ensure_sufficient_stack(|| self.component(segment, component, inherit))
            }
            (Kind::SlotOutlet, Source::Slot(slot)) => {
                vize_s0::ensure_sufficient_stack(|| self.slot_outlet(segment, slot))
            }
            (Kind::Text, Source::Text(_)) => {
                self.pos += 1;
                let content = plan_source(&segment, SsrStringPayloadKind::Text)?;
                text::emit_text(self.ctx, content)
            }
            (Kind::DynamicText, Source::Interpolation(interpolation)) => {
                self.pos += 1;
                require_dynamic(&segment)?;
                text::emit_interpolation(self, &segment, interpolation)
            }
            (Kind::If, Source::If(if_op)) => {
                vize_s0::ensure_sufficient_stack(|| self.if_chain(if_op, disable, inherit))
            }
            (Kind::For, Source::For(for_op)) => {
                vize_s0::ensure_sufficient_stack(|| self.for_loop(segment, for_op, disable))
            }
            (Kind::Comment, _) => Err(LegacyReason::Operation.into()),
            _ => Err(AdmissionFailure::Invalid(
                "string plan places a segment outside its region",
            )),
        }
    }

    /// An expression exactly as the legacy walker printed it: the shipped
    /// transform's rewrite, then its codegen strip for scoped params.
    fn expr(&self, expr: &ExprRef<'_>, content: TransformContent) -> Result<String> {
        let rewritten = self
            .exprs
            .expr(expr, content)
            .map_err(|_| LegacyReason::ExpressionOrEncoding)?;
        self.consume(rewritten)
    }

    /// [`Self::expr`] for fact-derived text with no retained AST.
    fn text_expr(&self, text: &str) -> Result<String> {
        let rewritten = self
            .exprs
            .text(text)
            .map_err(|_| LegacyReason::ExpressionOrEncoding)?;
        self.consume(rewritten)
    }

    fn consume(&self, rewritten: vize_s1_to_s2::TransformedExpr) -> Result<String> {
        // `_unref` only arises for inline render closures, which this lane
        // refuses up front; reaching it anyway is a broken option gate.
        if rewritten.used_unref {
            return Err(LegacyReason::ExpressionOrEncoding.into());
        }
        Ok(strip_scope_prefixes_for_scoped_params(
            &self.scoped_params,
            &rewritten.text,
        ))
    }
}

/// A segment that renders a runtime expression must read a dynamic fact:
/// the static partition is a promise that its bytes are known at compile time.
fn require_dynamic(segment: &SsrStringSegment<'_, '_>) -> Result<()> {
    if segment.partition == PartitionKind::Dynamic {
        Ok(())
    } else {
        Err(AdmissionFailure::Invalid(
            "partition fact classifies an expression segment as static",
        ))
    }
}

/// The segment's typed plan payload. Emission reads names and text from the
/// plan, never by re-inferring them from the segment kind.
fn plan_source<'a>(
    segment: &SsrStringSegment<'_, 'a>,
    kind: SsrStringPayloadKind,
) -> Result<&'a str> {
    match segment.payload {
        Some(payload) if payload.kind == kind => Ok(payload.source),
        _ => Err(AdmissionFailure::Invalid(
            "string plan segment lacks its typed payload",
        )),
    }
}
