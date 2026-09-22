//! Transfer S2's validated compound-text side facts into owned S3 operands.
//! Opaque display strings are never parsed or treated as executable JavaScript.

use oxc_allocator::HashMap;
use vize_carton::Allocator;
use vize_s3::operand::{OperandRole, OperandValue, ValueKind};

use super::{AdmissionFailure, LegacyReason, native::validate::reference, retained::Retained};

pub(super) fn capture<'a>(
    allocator: &'a Allocator,
    s2: &vize_s1_to_s2::Lowered<'_>,
    s3: &mut vize_s2_to_s3::Lowered<'a>,
    retained: &mut Retained<'_, 'a>,
) -> Result<(), AdmissionFailure> {
    let mut facts: HashMap<'_, (u32, u32), _> = HashMap::new_in(allocator.as_oxc());
    facts.extend(
        (s2.provenance.iter())
            .filter(|record| record.rule == "lower.compound")
            .filter_map(|record| {
                s2.texts
                    .get(record.node?)
                    .map(|parts| ((record.span.start, record.span.end), parts))
            }),
    );
    if facts.is_empty() {
        return if s3.program.operands.iter().any(|operand| {
            operand.role == OperandRole::Text
                && operand.value.kind == ValueKind::Opaque
                && operand.value.qualifier == "compound"
        }) {
            Err(AdmissionFailure::Invalid(
                "compound text lacks its S2 parts",
            ))
        } else {
            Ok(())
        };
    }
    let mut operands = vize_carton::Vec::new_in(&allocator);
    for operand in &s3.program.operands {
        let value = operand.value;
        if operand.role != OperandRole::Text
            || value.kind != ValueKind::Opaque
            || value.qualifier != "compound"
        {
            operands.push(*operand);
            continue;
        }
        let parts =
            facts
                .get(&(value.span.start, value.span.end))
                .ok_or(AdmissionFailure::Invalid(
                    "compound text lacks its S2 parts",
                ))?;
        if parts.parts.len() < 2
            || !parts.parts.iter().any(|part| part.dynamic)
            || parts
                .parts
                .first()
                .is_none_or(|part| part.span.start != value.span.start)
            || parts
                .parts
                .last()
                .is_none_or(|part| part.span.end != value.span.end)
            || parts
                .parts
                .windows(2)
                .any(|pair| pair[0].span.end != pair[1].span.start)
            || !rebuilds(&parts.parts, value.text)
        {
            return Err(AdmissionFailure::Invalid("compound text parts are stale"));
        }
        for part in &parts.parts {
            let text = allocator.alloc_str(&part.text);
            // S2 never parsed compound parts; a non-reference part takes its
            // single parse here, with S2's own admission rule.
            if part.dynamic && !reference(text) && !retained.parse(text, part.span) {
                return Err(LegacyReason::ExpressionOrEncoding.into());
            }
            operands.push(vize_s3::operand::Operand {
                value: OperandValue {
                    kind: if part.dynamic {
                        ValueKind::Js
                    } else {
                        ValueKind::Literal
                    },
                    text,
                    qualifier: "",
                    span: part.span,
                },
                ..*operand
            });
        }
    }
    s3.program.operands = operands;
    Ok(())
}

/// `vize_s1_to_s2::lower::rebuild_source(parts) == text`, without building
/// the rebuilt string.
fn rebuilds(parts: &[vize_s1_to_s2::lower::TextPart], mut text: &str) -> bool {
    for part in parts {
        let rest = if part.dynamic {
            text.strip_prefix("{{ ")
                .and_then(|rest| rest.strip_prefix(part.text.as_str()))
                .and_then(|rest| rest.strip_prefix(" }}"))
        } else {
            text.strip_prefix(part.text.as_str())
        };
        let Some(rest) = rest else {
            return false;
        };
        text = rest;
    }
    text.is_empty()
}

#[cfg(test)]
mod tests;
