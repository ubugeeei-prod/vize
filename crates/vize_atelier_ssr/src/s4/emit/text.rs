//! Static text and `{{ }}` output. S2 merges adjacent text/interpolation runs
//! into one compound op; the SSR lane renders each recorded part in place.

use vize_atelier_core::RuntimeHelper;
use vize_davinci::id::NodeId;
use vize_davinci::side_table::SideTable;
use vize_s0::Span;
use vize_s1_to_s2::lower::{TextPart, TextParts, rebuild_source};
use vize_s1_to_s2::{TransformContent, decode_ssr_static_text};
use vize_s2::expr::{ExprRef, OpaqueReason};
use vize_s2::op as s2;

use super::{Emitter, Result, spans};
use crate::codegen::SsrCodegenContext;
use crate::codegen::helpers::escape_html;
use crate::s4::string_plan::SsrStringSegment;
use crate::s4::{AdmissionFailure, LegacyReason};

/// Static text: the shipped parser decodes entities, the SSR lane re-escapes.
///
/// The shipped parser condenses whitespace *after* decoding, while S2
/// condenses the raw text; an entity that decodes to whitespace would make
/// the two disagree, so that text keeps the legacy lane.
pub(super) fn emit_text(ctx: &mut SsrCodegenContext<'_>, content: &str, start: u32) -> Result<()> {
    let decoded = decode_ssr_static_text(content);
    admit_decoded(content, &decoded)?;
    // Anchored at the authored text, as the AST walker anchors a text node.
    ctx.push_string_part_static_mapped(&escape_html(&decoded), start);
    Ok(())
}

/// Refuse text whose entities decode to whitespace (see [`emit_text`]).
pub(super) fn admit_decoded(content: &str, decoded: &str) -> Result<()> {
    if whitespace_count(decoded) != whitespace_count(content) {
        return Err(LegacyReason::ExpressionOrEncoding.into());
    }
    Ok(())
}

fn whitespace_count(text: &str) -> usize {
    text.bytes()
        .filter(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\x0c' | b'\r'))
        .count()
}

/// How many legacy AST children one interpolation segment stands for.
pub(super) fn legacy_child_count(
    texts: &SideTable<TextParts>,
    segment: &SsrStringSegment<'_, '_>,
    interpolation: &s2::InterpolationOp<'_>,
) -> Result<usize> {
    Ok(match compound_parts(texts, segment, interpolation)? {
        Some(parts) => parts.len(),
        None => 1,
    })
}

pub(super) fn emit_interpolation(
    em: &mut Emitter<'_, '_, '_, '_, '_, '_>,
    segment: &SsrStringSegment<'_, '_>,
    interpolation: &s2::InterpolationOp<'_>,
) -> Result<()> {
    if let Some(parts) = compound_parts(em.facts.texts, segment, interpolation)? {
        for part in parts {
            if part.dynamic {
                let rewritten = em.text_expr(part.text.as_str())?;
                let content = spans::interpolation_content(em.ctx.source, part.span);
                push_interpolate(em, rewritten, content.unwrap_or(part.span));
            } else {
                emit_text(em.ctx, part.text.as_str(), part.span.start)?;
            }
        }
        return Ok(());
    }
    let ExprRef::Js(js) = interpolation.expression else {
        return Err(LegacyReason::ExpressionOrEncoding.into());
    };
    let rewritten = em.expr(&interpolation.expression, TransformContent::Padded)?;
    let content = spans::trimmed(em.ctx.source, js.span);
    push_interpolate(em, rewritten, content);
    Ok(())
}

/// `_ssrInterpolate(exp)`, anchored at the authored content `span` as the AST
/// walker anchors an interpolation.
fn push_interpolate(em: &mut Emitter<'_, '_, '_, '_, '_, '_>, text: vize_s0::String, span: Span) {
    em.ctx.use_ssr_helper(RuntimeHelper::SsrInterpolate);
    em.ctx
        .push_wrapped_expression_part("_ssrInterpolate(", &text, ")", span);
}

/// The validated parts of a compound interpolation, or `None` for a plain one.
pub(super) fn compound_parts<'t>(
    texts: &'t SideTable<TextParts>,
    segment: &SsrStringSegment<'_, '_>,
    interpolation: &s2::InterpolationOp<'_>,
) -> Result<Option<&'t [TextPart]>> {
    let ExprRef::Opaque(opaque) = interpolation.expression else {
        return Ok(None);
    };
    if opaque.reason != OpaqueReason::Compound {
        return Ok(None);
    }
    let parts = NodeId::from_index(segment.fact)
        .and_then(|id| texts.get(id))
        .map(|parts| parts.parts.as_slice())
        .ok_or(AdmissionFailure::Invalid(
            "compound text lacks its S2 parts",
        ))?;
    let span = interpolation.span;
    let stale = parts.len() < 2
        || !parts.iter().any(|part| part.dynamic)
        || parts
            .first()
            .is_none_or(|part| part.span.start != span.start)
        || parts.last().is_none_or(|part| part.span.end != span.end)
        || parts
            .windows(2)
            .any(|pair| pair[0].span.end != pair[1].span.start)
        || rebuild_source(parts) != opaque.source;
    if stale {
        return Err(AdmissionFailure::Invalid("compound text parts are stale"));
    }
    Ok(Some(parts))
}
