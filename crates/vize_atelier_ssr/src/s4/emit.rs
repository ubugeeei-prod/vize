//! SSR code generation from the S4 string plan.
//!
//! The emitter consumes [`SsrStringPlan`] segments in order and writes the
//! `ssrRender` body through the shared SSR output writer. It never reads the
//! legacy template AST. Every shape outside the admitted surface returns a
//! typed legacy reason before any byte is committed to the caller; every
//! broken plan invariant (stale facts, misclassified partitions, unbalanced
//! region segments) is a rejection, never a guess.

mod attrs;
mod control;
mod element;
mod fallthrough;
mod model;
mod text;

use vize_davinci::side_table::SideTable;
use vize_s0::{FxHashSet, String};
use vize_s1_to_s2::lower::{ForWrapper, TextParts};
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

type Result<T> = core::result::Result<T, AdmissionFailure>;

/// The S2 side facts the emitter reads beside the plan.
pub(super) struct PlanFacts<'s> {
    pub(super) texts: &'s SideTable<TextParts>,
    pub(super) for_wrappers: &'s SideTable<ForWrapper>,
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
    /// The legacy walker's codegen scope: `v-for` destructure params whose
    /// transform-applied prefixes are stripped at emission.
    scoped_params: std::vec::Vec<FxHashSet<String>>,
    /// `v-model` reads of the open `<select>` ancestors.
    select_models: std::vec::Vec<String>,
}

/// How the legacy walker visits one child list.
#[derive(Clone, Copy)]
struct Flags {
    as_fragment: bool,
    disable_nested_fragments: bool,
    inherit_attrs: bool,
}

/// A region's direct children as the legacy AST lists them.
struct RegionShape {
    /// Legacy child count: a merged S2 text run counts one child per part,
    /// since the legacy lane never merges SSR text.
    legacy_children: usize,
    non_text: bool,
    /// Plan positions of the children that may inherit `_attrs`.
    candidates: std::vec::Vec<usize>,
}

impl<'r, 'a> Emitter<'_, 'r, 'a, '_, '_, '_> {
    /// Describe the child list starting at `from`, up to its closing segment.
    fn scan_region(&self, from: usize) -> Result<RegionShape> {
        let mut shape = RegionShape {
            legacy_children: 0,
            non_text: false,
            candidates: std::vec::Vec::new(),
        };
        let mut depth = 0usize;
        for (offset, segment) in self.segments[from..].iter().enumerate() {
            let top = depth == 0;
            match segment.kind {
                Kind::OpenElement | Kind::If | Kind::For | Kind::Branch => {
                    if top {
                        shape.legacy_children += 1;
                        shape.non_text = true;
                        if matches!(segment.kind, Kind::OpenElement | Kind::If) {
                            shape.candidates.push(from + offset);
                        }
                    }
                    depth += 1;
                }
                Kind::CloseElement | Kind::CloseIf | Kind::CloseFor | Kind::CloseBranch => {
                    if top {
                        break;
                    }
                    depth -= 1;
                }
                Kind::Text if top => shape.legacy_children += 1,
                Kind::DynamicText if top => {
                    let Source::Interpolation(interpolation) = segment.source else {
                        return Err(LegacyReason::Operation.into());
                    };
                    shape.legacy_children +=
                        text::legacy_child_count(self.facts.texts, segment, interpolation)?;
                    shape.non_text = true;
                }
                _ if top => return Err(LegacyReason::Operation.into()),
                _ => {}
            }
        }
        Ok(shape)
    }

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
            let inherit = fallthrough == Some(self.pos);
            match (segment.kind, segment.source) {
                (Kind::CloseElement | Kind::CloseIf | Kind::CloseFor | Kind::CloseBranch, _) => {
                    break;
                }
                (Kind::OpenElement, Source::Element(element)) => {
                    vize_s0::ensure_sufficient_stack(|| self.element(segment, element, inherit))?;
                }
                (Kind::Text, Source::Text(_)) => {
                    self.pos += 1;
                    let content = plan_source(&segment, SsrStringPayloadKind::Text)?;
                    text::emit_text(self.ctx, content)?;
                }
                (Kind::DynamicText, Source::Interpolation(interpolation)) => {
                    self.pos += 1;
                    require_dynamic(&segment)?;
                    text::emit_interpolation(self, &segment, interpolation)?;
                }
                (Kind::If, Source::If(if_op)) => {
                    let disable = flags.disable_nested_fragments;
                    vize_s0::ensure_sufficient_stack(|| self.if_chain(if_op, disable, inherit))?;
                }
                (Kind::For, Source::For(for_op)) => {
                    let disable = flags.disable_nested_fragments;
                    vize_s0::ensure_sufficient_stack(|| self.for_loop(segment, for_op, disable))?;
                }
                (Kind::Comment | Kind::Component | Kind::SlotOutlet, _) => {
                    return Err(LegacyReason::Operation.into());
                }
                _ => {
                    return Err(AdmissionFailure::Invalid(
                        "string plan places a segment outside its region",
                    ));
                }
            }
        }
        if flags.as_fragment {
            self.ctx.push_string_part_static("<!--]-->");
        }
        Ok(())
    }

    /// Consume the closing segment `kind` whose operand is `owner`.
    fn close(&mut self, kind: Kind, owner: impl Fn(&Source<'r, 'a>) -> bool) -> Result<()> {
        match self.segments.get(self.pos) {
            Some(close) if close.kind == kind && owner(&close.source) => {
                self.pos += 1;
                Ok(())
            }
            _ => Err(AdmissionFailure::Invalid(
                "string plan region is not closed by its owner",
            )),
        }
    }

    /// An expression exactly as the legacy walker printed it: the shipped
    /// transform's rewrite, then its codegen strip for `v-for` params.
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
