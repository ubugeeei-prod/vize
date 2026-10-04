//! Independent immutable expression, wrapper and safety bounds in complete SFCs.

use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use vize_glyph::native_doc::{
    ExpressionRefusal, LineEnding, NativeSfcObservation, NativeSfcOptions, NativeSfcRefusal,
    NativeTemplateRefusal, ObservedNativeTemplateRefusal, observe_native_sfc_in,
};
use vize_l0::{Allocator, Span};
use vize_l1::container::ContainerErrorCode;
use vize_l1::container::vue::{DescriptorIssue, DescriptorIssueCode as Code};
use vize_l1::embed::syntax::EmbedHole;

fn span(source: &str, spelling: &str) -> Span {
    let start = source.rfind(spelling).unwrap() as u32;
    Span::new(start, start + spelling.len() as u32)
}

fn fingerprint(expression: &Expression<'_>) -> std::string::String {
    match expression {
        Expression::BigIntLiteral(node) => format!(
            "{:?}:{:?}:{:?}",
            node.base,
            node.value.as_str(),
            node.raw.map(|raw| raw.as_str().to_owned())
        ),
        Expression::UnaryExpression(node) => format!(
            "{}({})",
            node.operator.as_str(),
            fingerprint(&node.argument)
        ),
        Expression::SequenceExpression(node) => format!(
            "{:?}",
            node.expressions
                .iter()
                .map(fingerprint)
                .collect::<std::vec::Vec<_>>()
        ),
        _ => panic!("outside these actual budget fixture families"),
    }
}

fn refused(owner: &NativeSfcObservation<'_>, source: &str, expected: NativeSfcRefusal) {
    assert!(core::ptr::eq(owner.source(), source));
    assert_eq!(owner.refusal(), Some(expected));
    assert_eq!(owner.document().unwrap_err(), expected);
    assert_eq!(owner.format().unwrap_err(), expected);
    assert_eq!(owner.options().descriptor, owner.descriptor().options());
    for operand in owner.operands() {
        assert!(core::ptr::eq(
            operand.syntax().source().authored_root(),
            source
        ));
        assert_eq!(operand.content_span().slice(source), operand.raw_content());
    }
}

fn late(refusal: NativeTemplateRefusal) -> NativeSfcRefusal {
    NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document { index: 1, refusal })
}

fn hole(owner: &NativeSfcObservation<'_>, source: &str, content: &str, hole: EmbedHole) {
    assert_eq!(owner.operands().len(), 2);
    let current = &owner.operands()[1];
    assert_eq!(current.raw_content(), content);
    assert_eq!(current.syntax().hole(), Some(hole));
    assert!(current.syntax().expression().is_none());
    assert_eq!(current.syntax().comments().count(), 0);
    assert_eq!(current.syntax().diagnostics().count(), 0);
    refused(
        owner,
        source,
        late(NativeTemplateRefusal::OperandRejected {
            offset: 9,
            index: 1,
            hole: Some(hole),
        }),
    );
    assert!(owner.interpolation_failure().is_none());
    assert_eq!(
        owner.selected().unwrap().component().block().source(),
        vize_l0::cstr!("{{{{1n}}}}<p>{{{{{content}}}}}</p>{{{{later}}}}")
    );
}

fn output(source: &str, expression: &str, flat_at: usize) {
    for width in [0, 1, 7, 60, 61, 80, 84, 85, 200] {
        for (ending, newline) in [(LineEnding::Lf, "\n"), (LineEnding::CrLf, "\r\n")] {
            let mut options = NativeSfcOptions::default();
            options.print.width = width;
            options.print.line_ending = ending;
            let expected = if width >= flat_at {
                vize_l0::cstr!("<template>{{{{ {expression} }}}}</template>")
            } else {
                vize_l0::cstr!("<template>{{{{{newline}  {expression}{newline}}}}}</template>")
            };
            let arena = Allocator::default();
            let original = observe_native_sfc_in(&arena, source, options);
            let before = fingerprint(original.operands()[0].syntax().expression().unwrap());
            let result = original.format().unwrap();
            assert_eq!(result.code, expected, "width={width} ending={ending:?}");
            assert_eq!(result.changed, result.code != source);
            let reparsed = observe_native_sfc_in(&arena, &result.code, options);
            assert!(reparsed.descriptor().admitted().is_ok());
            assert_eq!(reparsed.operands().len(), 1);
            assert_eq!(reparsed.operands()[0].syntax().hole(), None);
            assert_eq!(
                fingerprint(reparsed.operands()[0].syntax().expression().unwrap()),
                before
            );
            let fixed = reparsed.format().unwrap();
            assert_eq!(fixed.code, result.code);
            assert!(!fixed.changed);
        }
    }
}

#[test]
fn depth_sixteen_formats_whole_sfc_and_depth_seventeen_retains_the_original_leaf() {
    let at_limit = vize_l0::cstr!("{}1n", "!".repeat(16));
    let source = vize_l0::cstr!("<template>{{{{{at_limit}}}}}</template>");
    let flat = vize_l0::cstr!("{}1n", "! ".repeat(16));
    output(&source, &flat, 61);
    for operator in ["!", "&#33;"] {
        let arena = Allocator::default();
        let over = vize_l0::cstr!("{}1n", operator.repeat(17));
        let source =
            vize_l0::cstr!("<template>{{{{1n}}}}<p>{{{{{over}}}}}</p>{{{{later}}}}</template>tail");
        let owner = observe_native_sfc_in(&arena, &source, NativeSfcOptions::default());
        assert_eq!(owner.operands().len(), 2);
        let current = &owner.operands()[1];
        assert_eq!(current.syntax().hole(), None);
        let root = core::ptr::from_ref(current.syntax().expression().unwrap());
        let mut leaf = current.syntax().expression().unwrap();
        for _ in 0..17 {
            let Expression::UnaryExpression(unary) = leaf else {
                panic!("actual original depth")
            };
            leaf = &unary.argument;
        }
        assert!(
            matches!(leaf, Expression::BigIntLiteral(node) if node.value.as_str() == "1" && node.raw.unwrap().as_str() == "1n")
        );
        let span = current.syntax().decoded_span(leaf.span()).unwrap();
        assert_eq!(span, Span::new(17, 19));
        let map = current
            .syntax()
            .source()
            .decode_map()
            .map(|map| map.segments());
        let values = map.map(|segments| segments.to_vec());
        assert_eq!(map.is_some(), operator != "!");
        refused(
            &owner,
            &source,
            late(NativeTemplateRefusal::Expression {
                offset: 9,
                refusal: ExpressionRefusal::DepthLimit { span },
            }),
        );
        assert_eq!(current.raw_content(), over);
        assert_eq!(
            core::ptr::from_ref(current.syntax().expression().unwrap()),
            root
        );
        if let Some(map) = map {
            assert!(core::ptr::eq(
                current.syntax().source().decode_map().unwrap().segments(),
                map
            ));
            assert_eq!(map, values.unwrap().as_slice());
        }
        assert_eq!(current.syntax().diagnostics().count(), 0);
        assert!(owner.interpolation_failure().is_none());
    }
}

#[test]
fn whole_parser_wrapper_thirty_one_units_and_thirty_two_units_have_distinct_outcomes() {
    let at_limit = std::iter::repeat_n("1n", 15)
        .collect::<std::vec::Vec<_>>()
        .join(",");
    let source = vize_l0::cstr!("<template>{{{{{at_limit}}}}}</template>");
    let flat = std::iter::repeat_n("1n", 15)
        .collect::<std::vec::Vec<_>>()
        .join(", ");
    output(&source, &flat, 85);
    let arena = Allocator::default();
    let accepted = observe_native_sfc_in(&arena, &source, NativeSfcOptions::default());
    let Expression::SequenceExpression(sequence) =
        accepted.operands()[0].syntax().expression().unwrap()
    else {
        panic!("original Sequence")
    };
    assert_eq!(sequence.expressions.len(), 15);
    assert!(
        sequence
            .expressions
            .iter()
            .all(|item| matches!(item, Expression::BigIntLiteral(_)))
    );
    assert_eq!(accepted.operands()[0].syntax().hole(), None);
    // The actual Expr parser wrapper contributes two units to the original 29.
    let over = vize_l0::cstr!("!{at_limit}");
    let source =
        vize_l0::cstr!("<template>{{{{1n}}}}<p>{{{{{over}}}}}</p>{{{{later}}}}</template>tail");
    let owner = observe_native_sfc_in(&arena, &source, NativeSfcOptions::default());
    hole(&owner, &source, &over, EmbedHole::TokenBudget);
}

#[test]
fn numeric_run_four_thousand_ninety_six_and_ninety_seven_keep_authored_scalar_bytes() {
    let at_limit = vize_l0::cstr!("{}n", "9".repeat(4095));
    let source = vize_l0::cstr!("<template>{{{{{at_limit}}}}}</template>");
    output(&source, &at_limit, usize::MAX);
    let arena = Allocator::default();
    let accepted = observe_native_sfc_in(&arena, &source, NativeSfcOptions::default());
    let syntax = accepted.operands()[0].syntax();
    let Expression::BigIntLiteral(node) = syntax.expression().unwrap() else {
        panic!("actual scalar")
    };
    assert_eq!(node.raw.unwrap().as_str(), at_limit);
    assert_eq!(node.raw.unwrap().as_str().len(), 4096);
    assert_eq!(syntax.hole(), None);
    let over = vize_l0::cstr!("{}n", "9".repeat(4096));
    let source =
        vize_l0::cstr!("<template>{{{{1n}}}}<p>{{{{{over}}}}}</p>{{{{later}}}}</template>tail");
    let owner = observe_native_sfc_in(&arena, &source, NativeSfcOptions::default());
    hole(&owner, &source, &over, EmbedHole::SafetyAdmission);
}

#[test]
fn missing_component_and_uncertain_outer_boundaries_keep_native_descriptor_errors() {
    for source in ["<!-- only -->", "<template>{{1n}}", "<template/>"] {
        let arena = Allocator::default();
        let owner = observe_native_sfc_in(&arena, source, NativeSfcOptions::default());
        refused(&owner, source, NativeSfcRefusal::Descriptor);
        let empty = source == "<!-- only -->";
        assert_eq!(
            owner.descriptor().issues().first(),
            Some(&DescriptorIssue {
                code: if empty {
                    Code::MissingComponentBlock
                } else {
                    Code::UnsupportedBoundary
                },
                container_index: if empty { None } else { Some(0) },
                span: if empty {
                    Span::new(0, 0)
                } else {
                    span(
                        source,
                        if source == "<template/>" {
                            "<template/>"
                        } else {
                            "<template>"
                        },
                    )
                },
            })
        );
        if source == "<template>{{1n}}" {
            assert_eq!(
                owner.descriptor().container().errors.as_slice(),
                &[vize_l1::container::ContainerError {
                    code: ContainerErrorCode::MissingCloseTag,
                    offset: 0
                }]
            );
        } else {
            assert!(owner.descriptor().container().errors.is_empty());
        }
        assert!(owner.selected().is_none());
        assert!(owner.operands().is_empty());
    }
    for terminator in ["", "\r", "&#10;"] {
        let arena = Allocator::default();
        let source = vize_l0::cstr!("<template>{{{{&#39;//x&#39;{terminator}}}}}</template>");
        let owner = observe_native_sfc_in(&arena, &source, NativeSfcOptions::default());
        refused(&owner, &source, NativeSfcRefusal::Descriptor);
        assert_eq!(
            owner.descriptor().issues()[0].code,
            Code::UnsupportedBoundary
        );
        assert!(
            owner
                .descriptor()
                .container()
                .errors
                .iter()
                .any(|error| error.code == ContainerErrorCode::UncertainInterpolation)
        );
        assert!(owner.selected().is_none());
        assert!(owner.operands().is_empty());
    }
}

#[test]
fn original_raw_double_closer_refuses_at_the_earliest_selected_safety_boundary() {
    let arena = Allocator::default();
    let source = "<template>{{1n}}<p>{{/*原*/ {a:1n,b(){}} }}</p>{{later}}</template>tail";
    let owner = observe_native_sfc_in(&arena, source, NativeSfcOptions::default());
    assert!(owner.descriptor().admitted().is_ok());
    assert_eq!(owner.operands().len(), 2);
    let current = &owner.operands()[1];
    assert_eq!(current.raw_content(), "/*原*/ {a:1n,b(){");
    assert_eq!(current.full_span().slice(source), "{{/*原*/ {a:1n,b(){}}");
    assert_eq!(current.syntax().hole(), Some(EmbedHole::SafetyAdmission));
    assert!(current.syntax().expression().is_none());
    assert_eq!(current.syntax().comments().count(), 0);
    assert_eq!(current.syntax().diagnostics().count(), 0);
    refused(
        &owner,
        source,
        NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document {
            index: 1,
            refusal: NativeTemplateRefusal::OperandRejected {
                offset: 9,
                index: 1,
                hole: Some(EmbedHole::SafetyAdmission),
            },
        }),
    );
    assert_eq!(
        vize_l1::check_fidelity(&owner.selected().unwrap().component().carrier().tree),
        Ok(())
    );
    assert!(owner.interpolation_failure().is_none());
}
