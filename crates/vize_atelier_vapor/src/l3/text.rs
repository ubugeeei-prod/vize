//! Transfer L2's validated compound-text side facts into owned L3 operands.
//! Opaque display strings are never parsed or treated as executable JavaScript.

use oxc_allocator::HashMap;
use vize_carton::Allocator;
use vize_l3::operand::{OperandRole, OperandValue, ValueKind};

use super::{AdmissionFailure, LegacyReason, native::validate::reference, retained::Retained};

pub(super) fn capture<'a>(
    allocator: &'a Allocator,
    s2: &vize_l1_to_l2::Lowered<'_>,
    s3: &mut vize_l2_to_l3::Lowered<'a>,
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
                "compound text lacks its L2 parts",
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
                    "compound text lacks its L2 parts",
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
                .any(|pair| matches!(pair, [left, right] if left.span.end != right.span.start))
            || !rebuilds(&parts.parts, value.text)
        {
            return Err(AdmissionFailure::Invalid("compound text parts are stale"));
        }
        for part in &parts.parts {
            let text = allocator.alloc_str(&part.text);
            // L2 never parsed compound parts; a non-reference part takes its
            // single parse here, with L2's own admission rule.
            if part.dynamic && !reference(text) && !retained.parse(text, part.span) {
                return Err(LegacyReason::ExpressionOrEncoding.into());
            }
            operands.push(vize_l3::operand::Operand {
                value: OperandValue {
                    native_handler: None,
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

/// `vize_l1_to_l2::lower::rebuild_source(parts) == text`, without building
/// the rebuilt string.
fn rebuilds(parts: &[vize_l1_to_l2::lower::TextPart], mut text: &str) -> bool {
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
mod tests {
    use super::capture;
    use crate::l3::{AdmissionFailure, VaporL3BridgeStatus, admit, retained::Retained};
    use vize_carton::Allocator;
    use vize_l3::operand::{OperandRole, ValueKind};

    const SOURCE: &str = "<div>Hello {{ name }}!</div>";

    #[test]
    fn compound_parts_outlive_their_source_and_drive_generation() {
        let allocator = Allocator::new();
        let mut s3 = {
            let scratch = Allocator::new();
            let (tree, errors) = vize_l1::parse(&scratch, SOURCE);
            let s2 = vize_l1_to_l2::lower(&scratch, &tree, &errors);
            let mut s3 = vize_l2_to_l3::lower(&allocator, &s2.root);
            capture(&allocator, &s2, &mut s3, &mut Retained::new(&allocator)).unwrap();
            s3
        };
        assert!(vize_l3::verify::verify(&s3.program).is_empty());
        for operand in &mut s3.program.operands {
            if operand.role == OperandRole::Text && operand.value.kind == ValueKind::Js {
                operand.value.text = "changed";
            }
        }
        let VaporL3BridgeStatus::Accepted(artifact) = admit(s3, &Retained::new(&allocator)) else {
            panic!("expected native compound text artifact")
        };
        let ir = artifact
            .into_ir(&allocator, "<b>decoy</b>", None)
            .expect("an admitted artifact emits");
        let code = crate::generate::generate_vapor(&ir, None).code;
        assert!(
            code.contains("\"Hello \" + _toDisplayString(_ctx.changed) + \"!\""),
            "{code}"
        );
        assert!(!code.contains("decoy"));
        assert!(!code.contains("_ctx.name"));
    }

    #[test]
    fn stale_compound_facts_cannot_become_executable() {
        for mutation in 0..5 {
            let scratch = Allocator::new();
            let allocator = Allocator::new();
            let (tree, errors) = vize_l1::parse(&scratch, SOURCE);
            let mut s2 = vize_l1_to_l2::lower(&scratch, &tree, &errors);
            let mut s3 = vize_l2_to_l3::lower(&allocator, &s2.root);
            let (_, parts) = s2.texts.iter_mut().next().unwrap();
            match mutation {
                0 => parts.parts[0].text = "stale".into(),
                1 => parts.parts[0].span.start += 1,
                2 => parts.parts[1].span.start += 1,
                3 => parts.parts.last_mut().unwrap().span.end -= 1,
                _ => s2
                    .provenance
                    .retain(|record| record.rule != "lower.compound"),
            }
            assert!(
                matches!(
                    capture(&allocator, &s2, &mut s3, &mut Retained::new(&allocator)),
                    Err(AdmissionFailure::Invalid(_))
                ),
                "mutation {mutation}"
            );
        }
    }

    #[test]
    fn compound_parts_parse_once_or_select_the_legacy_lane() {
        // Unparseable text, dialect-divergent parses, and context-reserved roots
        // cannot be consumed from a retained AST byte-equivalently.
        for expression in [
            "for",
            "class",
            "await",
            "value as number",
            "$attrs.id",
            "a ? b",
        ] {
            let allocator = Allocator::new();
            let source = vize_carton::cstr!("<div>Hello {{{{ {expression} }}}}!</div>");
            assert!(
                matches!(
                    crate::l3::lower_source_for_vapor(
                        &allocator,
                        &source,
                        crate::l3::tests::options()
                    ),
                    VaporL3BridgeStatus::Legacy(crate::l3::LegacyReason::ExpressionOrEncoding)
                ),
                "{expression}"
            );
        }
        for expression in ["name + other", "call()", "state[key]", "a ? `${b}` : c"] {
            let allocator = Allocator::new();
            let source = vize_carton::cstr!("<div>Hello {{{{ {expression} }}}}!</div>");
            let status =
                crate::l3::lower_source_for_vapor(&allocator, &source, crate::l3::tests::options());
            assert!(
                matches!(status, VaporL3BridgeStatus::Accepted(_)),
                "{expression}: {status:?}"
            );
        }
    }
}
