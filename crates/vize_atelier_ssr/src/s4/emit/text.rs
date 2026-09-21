//! Static text and `{{ }}` output. S2 merges adjacent text/interpolation runs
//! into one compound op; the SSR lane renders each recorded part in place.

use vize_atelier_core::RuntimeHelper;
use vize_davinci::id::NodeId;
use vize_davinci::side_table::SideTable;
use vize_s0::cstr;
use vize_s1_to_s2::lower::{TextPart, TextParts, rebuild_source};
use vize_s1_to_s2::{TransformContent, decode_template_entities};
use vize_s2::expr::{ExprRef, OpaqueReason};
use vize_s2::op as s2;

use super::{Emitter, Result};
use crate::codegen::SsrCodegenContext;
use crate::codegen::helpers::escape_html;
use crate::s4::string_plan::SsrStringSegment;
use crate::s4::{AdmissionFailure, LegacyReason};

/// Static text: the shipped parser decodes entities, the SSR lane re-escapes.
///
/// The shipped parser condenses whitespace *after* decoding, while S2
/// condenses the raw text; an entity that decodes to whitespace would make
/// the two disagree, so that text keeps the legacy lane.
pub(super) fn emit_text(ctx: &mut SsrCodegenContext<'_>, content: &str) -> Result<()> {
    let decoded = decode_template_entities(content);
    if whitespace_count(&decoded) != whitespace_count(content) {
        return Err(LegacyReason::ExpressionOrEncoding.into());
    }
    ctx.push_string_part_static(&escape_html(&decoded));
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
                push_interpolate(em, rewritten);
            } else {
                emit_text(em.ctx, part.text.as_str())?;
            }
        }
        return Ok(());
    }
    if !matches!(interpolation.expression, ExprRef::Js(_)) {
        return Err(LegacyReason::ExpressionOrEncoding.into());
    }
    let rewritten = em.expr(&interpolation.expression, TransformContent::Padded)?;
    push_interpolate(em, rewritten);
    Ok(())
}

fn push_interpolate(em: &mut Emitter<'_, '_, '_, '_, '_, '_>, text: vize_s0::String) {
    em.ctx.use_ssr_helper(RuntimeHelper::SsrInterpolate);
    em.ctx
        .push_string_part_dynamic(&cstr!("_ssrInterpolate({text})"));
}

/// The validated parts of a compound interpolation, or `None` for a plain one.
fn compound_parts<'t>(
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
