//! Authentic retained prefix boundaries do not alter any native admission budget.

use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use vize_glyph::native_doc::{
    ExpressionRefusal, LineEnding, NativeTemplateRefusal, ObservedNativeTemplateRefusal,
    observed_native_template_document,
};
use vize_l0::{Allocator, Span};
use vize_l1::embed::syntax::EmbedHole;

use super::{SCRIPTS, WIDTHS, assert_fixed, format, newline, options, selected};

#[test]
fn genuine_depth_sixteen_succeeds_and_depth_seventeen_retains_the_current_original_leaf() {
    for script in SCRIPTS {
        let supported = vize_l0::cstr!("{}1n", "!".repeat(16));
        let source = vize_l0::cstr!("<template>{{{{{supported}}}}}</template>{script}");
        assert_eq!(
            format(&source, options(200, LineEnding::Lf)),
            vize_l0::cstr!("{{{{ {}1n }}}}", "! ".repeat(16))
        );
        assert_fixed(&source, script);
        let refused = vize_l0::cstr!("{}1n", "!".repeat(17));
        let source = vize_l0::cstr!(
            "<template>{{{{1n}}}}<p>{{{{{refused}}}}}</p>{{{{later}}}}</template>{script}"
        );
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let failure = observed_native_template_document(&owner, &arena).unwrap_err();
        assert_eq!(failure.operands().len(), 2);
        let current = &failure.operands()[1];
        assert_eq!(current.syntax().hole(), None);
        let mut leaf = current.syntax().expression().unwrap();
        for _ in 0..17 {
            let Expression::UnaryExpression(unary) = leaf else {
                panic!("genuine original prefix")
            };
            leaf = &unary.argument;
        }
        assert!(
            matches!(leaf, Expression::BigIntLiteral(literal) if literal.value.as_str() == "1")
        );
        let span = current.syntax().decoded_span(leaf.span()).unwrap();
        assert_eq!(span, Span::new(17, 19));
        assert_eq!(
            failure.refusal(),
            ObservedNativeTemplateRefusal::Document {
                index: 1,
                refusal: NativeTemplateRefusal::Expression {
                    offset: 9,
                    refusal: ExpressionRefusal::DepthLimit { span }
                },
            }
        );
        assert_eq!(current.raw_content(), refused);
        assert!(core::ptr::eq(
            current.syntax().source().authored_root(),
            source.as_str()
        ));
        assert!(failure.interpolation_failure().is_none());
    }
}

#[test]
fn real_thirty_one_unit_input_succeeds_and_thirty_two_keeps_the_rejected_current_owner() {
    let at_limit = std::iter::repeat_n("1n", 15)
        .collect::<std::vec::Vec<_>>()
        .join(",");
    for script in SCRIPTS {
        let source = vize_l0::cstr!("<template>{{{{{at_limit}}}}}</template>{script}");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let document = observed_native_template_document(&owner, &arena).unwrap();
        assert_eq!(document.operands().len(), 1);
        assert_eq!(document.operands()[0].syntax().hole(), None);
        let Expression::SequenceExpression(sequence) =
            document.operands()[0].syntax().expression().unwrap()
        else {
            panic!("original bounded Sequence")
        };
        assert_eq!(sequence.expressions.len(), 15);
        assert_fixed(&source, script);
        let over = vize_l0::cstr!("!{at_limit}");
        let source = vize_l0::cstr!(
            "<template>{{{{1n}}}}<p>{{{{{over}}}}}</p>{{{{later}}}}</template>{script}"
        );
        let owner = selected(&arena, &source);
        let failure = observed_native_template_document(&owner, &arena).unwrap_err();
        assert_eq!(failure.operands().len(), 2);
        let current = &failure.operands()[1];
        assert_eq!(current.syntax().hole(), Some(EmbedHole::TokenBudget));
        assert!(current.syntax().expression().is_none());
        assert_eq!(current.syntax().comments().count(), 0);
        assert_eq!(current.syntax().diagnostics().count(), 0);
        assert_eq!(current.raw_content(), over);
        assert_eq!(
            failure.refusal(),
            ObservedNativeTemplateRefusal::Document {
                index: 1,
                refusal: NativeTemplateRefusal::OperandRejected {
                    offset: 9,
                    index: 1,
                    hole: Some(EmbedHole::TokenBudget)
                },
            }
        );
        assert!(failure.interpolation_failure().is_none());
        assert_eq!(current.content_span().slice(&source), current.raw_content());
    }
}

#[test]
fn numeric_four_thousand_ninety_six_boundary_keeps_the_original_suffix_and_failed_prefix() {
    let literal = vize_l0::cstr!("{}n", "9".repeat(4095));
    for script in SCRIPTS {
        let source = vize_l0::cstr!("<template>{{{{{literal}}}}}</template>{script}");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let carrier = observed_native_template_document(&owner, &arena).unwrap();
        let syntax = carrier.operands()[0].syntax();
        assert_eq!(syntax.hole(), None);
        let Expression::BigIntLiteral(node) = syntax.expression().unwrap() else {
            panic!("actual scalar atom")
        };
        assert_eq!(node.raw.unwrap().as_str().len(), 4096);
        assert_eq!(node.raw.unwrap().as_str(), literal);
        for width in WIDTHS {
            for ending in [LineEnding::Lf, LineEnding::CrLf] {
                let generated = newline(ending);
                assert_eq!(
                    format(&source, options(width, ending)),
                    vize_l0::cstr!("{{{{{generated}  {literal}{generated}}}}}")
                );
            }
        }
        assert_fixed(&source, script);
        let over = vize_l0::cstr!("{}n", "9".repeat(4096));
        let source = vize_l0::cstr!(
            "<template>{{{{1n}}}}<p>{{{{{over}}}}}</p>{{{{later}}}}</template>{script}"
        );
        let owner = selected(&arena, &source);
        let failure = observed_native_template_document(&owner, &arena).unwrap_err();
        assert_eq!(failure.operands().len(), 2);
        let current = &failure.operands()[1];
        assert_eq!(current.syntax().hole(), Some(EmbedHole::SafetyAdmission));
        assert!(current.syntax().expression().is_none());
        assert_eq!(current.raw_content(), over);
        assert_eq!(
            failure.refusal(),
            ObservedNativeTemplateRefusal::Document {
                index: 1,
                refusal: NativeTemplateRefusal::OperandRejected {
                    offset: 9,
                    index: 1,
                    hole: Some(EmbedHole::SafetyAdmission)
                },
            }
        );
        assert_eq!(current.content_span().slice(&source), current.raw_content());
        assert!(core::ptr::eq(
            current.syntax().source().authored_root(),
            source.as_str()
        ));
        assert!(failure.interpolation_failure().is_none());
    }
}
