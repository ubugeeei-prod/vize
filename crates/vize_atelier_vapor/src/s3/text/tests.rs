use super::capture;
use crate::s3::{AdmissionFailure, VaporS3BridgeStatus, admit, retained::Retained};
use vize_carton::Allocator;
use vize_s3::operand::{OperandRole, ValueKind};

const SOURCE: &str = "<div>Hello {{ name }}!</div>";

#[test]
fn compound_parts_outlive_their_source_and_drive_generation() {
    let allocator = Allocator::new();
    let mut s3 = {
        let scratch = Allocator::new();
        let (tree, errors) = vize_s1::parse(&scratch, SOURCE);
        let s2 = vize_s1_to_s2::lower(&scratch, &tree, &errors);
        let mut s3 = vize_s2_to_s3::lower(&allocator, &s2.root);
        capture(&allocator, &s2, &mut s3, &mut Retained::new(&allocator)).unwrap();
        s3
    };
    assert!(vize_s3::verify::verify(&s3.program).is_empty());
    for operand in &mut s3.program.operands {
        if operand.role == OperandRole::Text && operand.value.kind == ValueKind::Js {
            operand.value.text = "changed";
        }
    }
    let VaporS3BridgeStatus::Accepted(artifact) = admit(s3, &Retained::new(&allocator)) else {
        panic!("expected native compound text artifact")
    };
    let ir = artifact.into_ir(&allocator, "<b>decoy</b>", None);
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
        let (tree, errors) = vize_s1::parse(&scratch, SOURCE);
        let mut s2 = vize_s1_to_s2::lower(&scratch, &tree, &errors);
        let mut s3 = vize_s2_to_s3::lower(&allocator, &s2.root);
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
                crate::s3::lower_source_for_vapor(&allocator, &source, crate::s3::tests::options()),
                VaporS3BridgeStatus::Legacy(crate::s3::LegacyReason::ExpressionOrEncoding)
            ),
            "{expression}"
        );
    }
    for expression in ["name + other", "call()", "state[key]", "a ? `${b}` : c"] {
        let allocator = Allocator::new();
        let source = vize_carton::cstr!("<div>Hello {{{{ {expression} }}}}!</div>");
        let status =
            crate::s3::lower_source_for_vapor(&allocator, &source, crate::s3::tests::options());
        assert!(
            matches!(status, VaporS3BridgeStatus::Accepted(_)),
            "{expression}: {status:?}"
        );
    }
}
