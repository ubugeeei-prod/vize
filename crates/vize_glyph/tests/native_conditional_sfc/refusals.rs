//! Complete earliest-owner and later Doc refusals retain the actual prefix.

use super::options;
use oxc_ast::ast::{Argument, Expression};
use oxc_span::GetSpan;
use vize_glyph::native_doc::{
    ExpressionRefusal, LineEnding, NativeSfcObservation, NativeSfcRefusal, NativeTemplateRefusal,
    ObservedNativeTemplateRefusal, observe_native_sfc_in,
};
use vize_l0::{Allocator, Span};
use vize_l1::embed::syntax::EmbedHole;

fn refused(owner: &NativeSfcObservation<'_>, source: &str, expected: NativeTemplateRefusal) {
    let refusal = NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document {
        index: 1,
        refusal: expected,
    });
    assert_eq!(owner.source(), source);
    assert!(core::ptr::eq(owner.source(), source));
    assert_eq!(owner.options(), options(200, 2, LineEnding::Lf));
    assert_eq!(owner.descriptor().options(), owner.options().descriptor);
    assert_eq!(owner.descriptor().issues(), []);
    assert_eq!(owner.descriptor().container().errors.as_slice(), []);
    assert_eq!(
        owner.selected().unwrap().component().block().source(),
        &source[10..source.len() - 11]
    );
    for _ in 0..2 {
        assert_eq!(owner.refusal(), Some(refusal));
        assert_eq!(owner.document().unwrap_err(), refusal);
        assert_eq!(owner.format().unwrap_err(), refusal);
    }
    assert_eq!(owner.operands().len(), 1);
    assert_eq!(owner.operands()[0].raw_content(), "1n");
    assert_eq!(owner.attribute_operands().len(), 1);
    let current = &owner.attribute_operands()[0];
    assert_eq!(current.value_span().slice(source), current.raw_value());
    assert!(core::ptr::eq(
        current.syntax().source().authored_root(),
        source
    ));
    assert!(owner.attribute_failure().is_none());
    assert!(owner.interpolation_failure().is_none());
}

#[test]
fn unquoted_conditional_refuses_whole_after_parking_its_original_admitted_expression() {
    let source = "<template>{{1n}}<p v-if=a+b>{{later}}</p></template>";
    let arena = Allocator::default();
    let owner = observe_native_sfc_in(&arena, source, options(200, 2, LineEnding::Lf));
    refused(
        &owner,
        source,
        NativeTemplateRefusal::UnquotedConditionalValue {
            span: Span::new(24, 27),
        },
    );
    let current = &owner.attribute_operands()[0];
    assert_eq!(current.raw_value(), "a+b");
    assert_eq!(current.syntax().source().text(), "a+b");
    assert_eq!(current.syntax().hole(), None);
    assert_eq!(current.syntax().diagnostics().count(), 0);
    assert!(matches!(
        current.syntax().expression(),
        Some(Expression::BinaryExpression(_))
    ));
}

#[test]
fn mapped_spread_child_refuses_doc_without_replacing_the_original_admitted_call() {
    let source = "<template>{{1n}}<p v-if='a&#43;f(...b)'>{{later}}</p></template>";
    let arena = Allocator::default();
    let owner = observe_native_sfc_in(&arena, source, options(200, 2, LineEnding::Lf));
    refused(
        &owner,
        source,
        NativeTemplateRefusal::AttributeExpression {
            span: Span::new(25, 38),
            refusal: ExpressionRefusal::UnsupportedNode {
                span: Span::new(2, 9),
            },
        },
    );
    let current = &owner.attribute_operands()[0];
    assert_eq!(current.raw_value(), "a&#43;f(...b)");
    let syntax = current.syntax();
    assert_eq!(syntax.source().text(), "a+f(...b)");
    assert_eq!(syntax.hole(), None);
    let Expression::BinaryExpression(binary) = syntax.expression().unwrap() else {
        panic!("original binary")
    };
    let Expression::CallExpression(call) = &binary.right else {
        panic!("original call")
    };
    assert_eq!(syntax.decoded_span(call.span()).unwrap(), Span::new(2, 9));
    assert_eq!(
        syntax.authored_span(call.span()).unwrap(),
        Span::new(31, 38)
    );
    assert_eq!(Span::new(31, 38).slice(source), "f(...b)");
    assert!(matches!(call.arguments[0], Argument::SpreadElement(_)));
    assert_eq!(call.arguments.len(), 1);
    assert_eq!(syntax.diagnostics().count(), 0);
}

#[test]
fn safety_admission_keeps_the_complete_original_value_hole_and_unvisited_suffix() {
    let source = "<template>{{1n}}<p v-if='(ready'>{{later}}</p></template>";
    let arena = Allocator::default();
    let owner = observe_native_sfc_in(&arena, source, options(200, 2, LineEnding::Lf));
    refused(
        &owner,
        source,
        NativeTemplateRefusal::AttributeRejected {
            span: Span::new(25, 31),
            index: 0,
            hole: Some(EmbedHole::SafetyAdmission),
        },
    );
    let syntax = owner.attribute_operands()[0].syntax();
    assert_eq!(syntax.source().text(), "(ready");
    assert_eq!(syntax.hole(), Some(EmbedHole::SafetyAdmission));
    assert!(syntax.expression().is_none());
    assert_eq!(syntax.comments().count(), 0);
    assert_eq!(syntax.diagnostics().count(), 0);
}

#[test]
fn original_doc_depth_and_wrapped_unit_limits_remain_separate_whole_refusals() {
    let raw = format!("{}1n", "!".repeat(17));
    let source = format!("<template>{{{{1n}}}}<p v-if='{raw}'>{{{{later}}}}</p></template>");
    let arena = Allocator::default();
    let owner = observe_native_sfc_in(&arena, &source, options(200, 2, LineEnding::Lf));
    refused(
        &owner,
        &source,
        NativeTemplateRefusal::AttributeExpression {
            span: Span::new(25, 44),
            refusal: ExpressionRefusal::DepthLimit {
                span: Span::new(17, 19),
            },
        },
    );
    assert_eq!(owner.attribute_operands()[0].syntax().hole(), None);
    assert_eq!(owner.attribute_operands()[0].raw_value(), raw);
    let raw = format!("{}a{}", "(".repeat(32), ")".repeat(32));
    let source = format!("<template>{{{{1n}}}}<p v-if='{raw}'>{{{{later}}}}</p></template>");
    let arena = Allocator::default();
    let owner = observe_native_sfc_in(&arena, &source, options(200, 2, LineEnding::Lf));
    refused(
        &owner,
        &source,
        NativeTemplateRefusal::AttributeRejected {
            span: Span::new(25, 90),
            index: 0,
            hole: Some(EmbedHole::TokenBudget),
        },
    );
    assert_eq!(owner.attribute_operands()[0].raw_value(), raw);
    assert_eq!(
        owner.attribute_operands()[0].syntax().hole(),
        Some(EmbedHole::TokenBudget)
    );
    assert_eq!(owner.attribute_operands()[0].syntax().comments().count(), 0);
    assert_eq!(
        owner.attribute_operands()[0].syntax().diagnostics().count(),
        0
    );
}
