//! SSR code generation from the S4 string plan.
//!
//! The emitter consumes [`SsrStringPlan`] segments in order and writes the
//! `ssrRender` body through the shared SSR output writer. It never reads the
//! legacy template AST. Every shape outside the admitted surface returns a
//! typed legacy reason before any byte is committed to the caller; every
//! broken plan invariant (stale facts, misclassified partitions, unbalanced
//! element segments) is a rejection, never a guess.

mod attrs;
mod text;

use vize_davinci::side_table::SideTable;
use vize_s1_to_s2::TransformExpressions;
use vize_s1_to_s2::lower::TextParts;
use vize_s2::op as s2;
use vize_s2_to_s3::PartitionKind;

use super::string_plan::{
    SsrSegmentSource as Source, SsrStringPayloadKind, SsrStringPlan, SsrStringSegment,
    SsrStringSegmentKind as Kind,
};
use super::{AdmissionFailure, LegacyReason};
use crate::codegen::SsrCodegenContext;

type Result<T> = core::result::Result<T, AdmissionFailure>;

/// Elements whose content model or runtime helpers are outside this slice:
/// raw-text / RCDATA parents, `<template>`, and the `v-model` form parents.
const REFUSED_TAGS: &[&str] = &[
    "template",
    "slot",
    "component",
    "script",
    "style",
    "textarea",
    "title",
    "xmp",
    "iframe",
    "noembed",
    "noframes",
    "noscript",
    "plaintext",
    "select",
    "option",
];

/// Write the render body for `plan` into `ctx`.
pub(super) fn emit_plan(
    ctx: &mut SsrCodegenContext<'_>,
    plan: &SsrStringPlan<'_, '_>,
    texts: &SideTable<TextParts>,
    exprs: &TransformExpressions<'_>,
) -> Result<()> {
    let mut emitter = Emitter {
        segments: &plan.segments,
        pos: 0,
        ctx,
        texts,
        exprs,
    };
    let root = emitter.root_shape()?;
    if root.fragment {
        emitter.ctx.push_string_part_static("<!--[-->");
    }
    emitter.region(root.fallthrough)?;
    if root.fragment {
        emitter.ctx.push_string_part_static("<!--]-->");
    }
    if emitter.pos != emitter.segments.len() {
        return Err(AdmissionFailure::Invalid(
            "string plan has unbalanced segments",
        ));
    }
    Ok(())
}

struct Emitter<'p, 'r, 'a, 'c, 'x> {
    segments: &'p [SsrStringSegment<'r, 'a>],
    pos: usize,
    ctx: &'c mut SsrCodegenContext<'x>,
    texts: &'p SideTable<TextParts>,
    exprs: &'p TransformExpressions<'p>,
}

/// What the legacy root walk decides before it visits any child.
struct RootShape<'r, 'a> {
    /// `<!--[-->` / `<!--]-->` around the root children.
    fragment: bool,
    /// The single root element that inherits `_attrs`.
    fallthrough: Option<&'r s2::ElementOp<'a>>,
}

impl<'r, 'a> Emitter<'_, 'r, 'a, '_, '_> {
    /// Root children as the legacy AST lists them: a merged text run counts
    /// one child per part, since the legacy lane never merges SSR text.
    fn root_shape(&self) -> Result<RootShape<'r, 'a>> {
        let (mut children, mut non_text, mut elements) = (0usize, false, std::vec::Vec::new());
        let mut depth = 0usize;
        for segment in self.segments {
            match (segment.kind, segment.source) {
                (Kind::OpenElement, Source::Element(element)) => {
                    if depth == 0 {
                        children += 1;
                        non_text = true;
                        elements.push(element);
                    }
                    depth += 1;
                }
                (Kind::CloseElement, Source::Element(_)) => {
                    depth = depth.checked_sub(1).ok_or(AdmissionFailure::Invalid(
                        "string plan closes an element it never opened",
                    ))?;
                }
                (Kind::Text, Source::Text(_)) if depth == 0 => children += 1,
                (Kind::DynamicText, Source::Interpolation(interpolation)) if depth == 0 => {
                    children += text::legacy_child_count(self.texts, segment, interpolation)?;
                    non_text = true;
                }
                (Kind::Text | Kind::DynamicText, _) if depth == 0 => {
                    return Err(LegacyReason::Operation.into());
                }
                _ => {}
            }
        }
        let fragment = children > 1 && non_text;
        let fallthrough = match elements.as_slice() {
            [element] if !fragment => Some(*element),
            _ => None,
        };
        Ok(RootShape {
            fragment,
            fallthrough,
        })
    }

    /// Emit children until the enclosing element's close segment (or the end
    /// of the plan at the root).
    fn region(&mut self, fallthrough: Option<&'r s2::ElementOp<'a>>) -> Result<()> {
        while let Some(segment) = self.segments.get(self.pos).copied() {
            match (segment.kind, segment.source) {
                (Kind::CloseElement, _) => return Ok(()),
                (Kind::OpenElement, Source::Element(element)) => {
                    let inherit = fallthrough.is_some_and(|root| core::ptr::eq(root, element));
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
                (Kind::Comment, _) => return Err(LegacyReason::Operation.into()),
                (Kind::Component | Kind::If | Kind::For | Kind::SlotOutlet, _) => {
                    return Err(LegacyReason::Operation.into());
                }
                _ => {
                    return Err(AdmissionFailure::Invalid(
                        "string plan places an attached segment in a child position",
                    ));
                }
            }
        }
        Ok(())
    }

    fn element(
        &mut self,
        open: SsrStringSegment<'r, 'a>,
        element: &'r s2::ElementOp<'a>,
        inherit: bool,
    ) -> Result<()> {
        let tag = plan_source(&open, SsrStringPayloadKind::TagName)?;
        admit_element(tag)?;
        self.pos += 1;
        let attached_len = element.attributes.len() + element.bindings.len();
        let end = self.pos + attached_len;
        let attached = self
            .segments
            .get(self.pos..end)
            .ok_or(AdmissionFailure::Invalid(
                "string plan lost attached element segments",
            ))?;
        self.pos = end;
        attrs::admit(attached, open.fact)?;

        self.ctx.push_string_part_static("<");
        self.ctx.push_string_part_static(tag);
        if inherit {
            attrs::emit_fallthrough(self, attached)?;
        } else {
            attrs::emit_inline(self, attached)?;
        }
        let options = self.ctx.options;
        if let Some(scope_id) = &options.scope_id {
            self.ctx.push_string_part_static(" ");
            self.ctx.push_string_part_static(scope_id);
        }
        self.ctx.push_string_part_static(">");

        let void = vize_s0::is_void_tag(tag);
        if void && !element.children.ops.is_empty() {
            return Err(LegacyReason::Structure.into());
        }
        if !void {
            self.region(None)?;
        }
        match self.segments.get(self.pos) {
            Some(close)
                if close.kind == Kind::CloseElement
                    && matches!(close.source, Source::Element(closed) if core::ptr::eq(closed, element)) =>
            {
                self.pos += 1;
            }
            _ => {
                return Err(AdmissionFailure::Invalid(
                    "string plan element is not closed",
                ));
            }
        }
        if !void {
            self.ctx.push_string_part_static("</");
            self.ctx.push_string_part_static(tag);
            self.ctx.push_string_part_static(">");
        }
        Ok(())
    }
}

fn admit_element(tag: &str) -> Result<()> {
    if REFUSED_TAGS.contains(&tag) || !vize_s0::is_native_tag(tag) {
        return Err(LegacyReason::Element.into());
    }
    Ok(())
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
