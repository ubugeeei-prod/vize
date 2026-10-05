use super::{
    failure_support::{document, prefix},
    options,
};
use oxc_ast::ast::{Argument, Expression};
use oxc_span::GetSpan;
use vize_glyph::native_doc::{
    ExpressionRefusal, LineEnding, NativeSfcRefusal, NativeTemplateRefusal,
    ObservedNativeTemplateRefusal, observe_native_sfc_in,
};
use vize_l0::{Allocator, Span};
use vize_l1::embed::{DecodeSegmentKind, syntax::EmbedHole};
use vize_l1::markup::NativeAttributeOperandError;

#[test]
fn unquoted_current_binding_is_parked_after_all_three_prefix_families_before_whole_refusal() {
    let source =
        "<template><i :title='first'>{{1n}}</i><p v-if='ok' :id=a+b>{{later}}</p></template>";
    let arena = Allocator::default();
    let owner = observe_native_sfc_in(&arena, source, options(200, 2, LineEnding::Lf));
    prefix(
        &owner,
        source,
        document(NativeTemplateRefusal::UnquotedBindingValue {
            span: Span::new(55, 58),
        }),
        2,
        Span::new(59, 68),
    );
    let current = &owner.binding_operands()[1];
    assert_eq!(current.name_span(), Span::new(51, 54));
    assert_eq!(current.argument_span(), Span::new(52, 54));
    assert_eq!(current.value_span(), Span::new(55, 58));
    assert_eq!(current.raw_value(), "a+b");
    assert_eq!(current.syntax().source().text(), "a+b");
    assert_eq!(current.syntax().hole(), None);
    assert_eq!(current.syntax().comments().count(), 0);
    assert_eq!(current.syntax().diagnostics().count(), 0);
    assert!(matches!(
        current.syntax().expression(),
        Some(Expression::BinaryExpression(_))
    ));
    let selected = owner.selected().unwrap();
    let element = selected.children().nth(1).unwrap().into_element().unwrap();
    assert!(
        current
            .admitted_for(selected, element.attributes().nth(1).unwrap())
            .is_some()
    );
    assert!(owner.binding_failure().is_none());
}

#[test]
fn syntax_hole_current_binding_retains_the_complete_original_value_after_three_prefixes() {
    let source =
        "<template><i :title='first'>{{1n}}</i><p v-if='ok' :id='(ready'>{{later}}</p></template>";
    let arena = Allocator::default();
    let owner = observe_native_sfc_in(&arena, source, options(200, 2, LineEnding::Lf));
    prefix(
        &owner,
        source,
        document(NativeTemplateRefusal::BindingRejected {
            span: Span::new(56, 62),
            index: 1,
            hole: Some(EmbedHole::SafetyAdmission),
        }),
        2,
        Span::new(64, 73),
    );
    let current = &owner.binding_operands()[1];
    assert_eq!(current.name_span(), Span::new(51, 54));
    assert_eq!(current.argument_span(), Span::new(52, 54));
    assert_eq!(current.value_span(), Span::new(56, 62));
    assert_eq!(current.raw_value(), "(ready");
    let syntax = current.syntax();
    assert_eq!(syntax.source().text(), "(ready");
    assert_eq!(syntax.source().span(), Span::new(56, 62));
    assert_eq!(syntax.hole(), Some(EmbedHole::SafetyAdmission));
    assert!(syntax.expression().is_none());
    assert!(syntax.admitted_expression().is_none());
    assert_eq!(syntax.comments().count(), 0);
    assert_eq!(syntax.diagnostics().count(), 0);
    assert!(core::ptr::eq(syntax.source().authored_root(), source));
    assert!(owner.binding_failure().is_none());
}

#[test]
fn mapped_spread_doc_refusal_keeps_the_original_admitted_current_ast_and_complete_decode_map() {
    let source = "<template><i :title='first'>{{1n}}</i><p v-if='ok' :id='a&#43;f(...b)'>{{later}}</p></template>";
    let arena = Allocator::default();
    let owner = observe_native_sfc_in(&arena, source, options(200, 2, LineEnding::Lf));
    prefix(
        &owner,
        source,
        document(NativeTemplateRefusal::BindingExpression {
            span: Span::new(56, 69),
            refusal: ExpressionRefusal::UnsupportedNode {
                span: Span::new(2, 9),
            },
        }),
        2,
        Span::new(71, 80),
    );
    let current = &owner.binding_operands()[1];
    assert_eq!(current.name_span(), Span::new(51, 54));
    assert_eq!(current.argument_span(), Span::new(52, 54));
    assert_eq!(current.raw_value(), "a&#43;f(...b)");
    assert_eq!(current.value_span(), Span::new(56, 69));
    let syntax = current.syntax();
    assert_eq!(syntax.source().text(), "a+f(...b)");
    assert_eq!(syntax.hole(), None);
    let Expression::BinaryExpression(binary) = syntax.expression().unwrap() else {
        panic!("original binary")
    };
    let Expression::CallExpression(call) = &binary.right else {
        panic!("original call")
    };
    assert_eq!(syntax.decoded_span(call.span()), Ok(Span::new(2, 9)));
    assert_eq!(syntax.authored_span(call.span()), Ok(Span::new(62, 69)));
    assert_eq!(call.arguments.len(), 1);
    assert!(matches!(call.arguments[0], Argument::SpreadElement(_)));
    let map: Vec<_> = syntax
        .source()
        .decode_map()
        .unwrap()
        .segments()
        .iter()
        .map(|segment| (segment.decoded(), segment.authored(), segment.kind()))
        .collect();
    assert_eq!(
        map,
        [
            (
                Span::new(0, 1),
                Span::new(56, 57),
                DecodeSegmentKind::Identity
            ),
            (
                Span::new(1, 2),
                Span::new(57, 62),
                DecodeSegmentKind::Entity
            ),
            (
                Span::new(2, 9),
                Span::new(62, 69),
                DecodeSegmentKind::Identity
            ),
        ]
    );
    assert_eq!(syntax.comments().count(), 0);
    assert_eq!(syntax.diagnostics().count(), 0);
    let selected = owner.selected().unwrap();
    let element = selected.children().nth(1).unwrap().into_element().unwrap();
    assert!(
        current
            .admitted_for(selected, element.attributes().nth(1).unwrap())
            .is_some()
    );
    assert!(owner.binding_failure().is_none());
}

#[test]
fn missing_value_keeps_the_actual_failure_and_all_earlier_families_without_current_surrogate() {
    let source = "<template><i :title='first'>{{1n}}</i><p v-if='ok' :id>{{later}}</p></template>";
    let arena = Allocator::default();
    let owner = observe_native_sfc_in(&arena, source, options(200, 2, LineEnding::Lf));
    let expected = NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Binding {
        span: Span::new(51, 54),
        index: 1,
        kind: NativeAttributeOperandError::IncompleteValue,
    });
    prefix(&owner, source, expected, 1, Span::new(55, 64));
    let failure = owner.binding_failure().unwrap();
    assert_eq!(failure.kind(), NativeAttributeOperandError::IncompleteValue);
    assert!(failure.syntax().is_none());
}
