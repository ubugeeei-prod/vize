use super::{options, selected};
use vize_glyph::native_doc::{
    Doc, LineEnding, NativeTemplateRefusal, NativeTemplateValuePolicy,
    ObservedNativeTemplateFailureParts, ObservedNativeTemplateRefusal,
    observed_native_template_document_with_policy, print,
};
use vize_l0::{Allocator, Span};
use vize_l1::markup::{
    NativeAttributeExpression, NativeAttributeOperandError, NativeInterpolationOperand,
    NativeTemplateComponent,
};

#[test]
fn complete_named_document_transfer_keeps_all_three_original_families_and_old_four_part_schema() {
    let arena = Allocator::default();
    let source = "<template><i :title='first'>{{1n}}</i><p v-if='ok' :id='a+b'></p></template>";
    let expected_body = "<i :title='first'>{{ 1n }}</i><p v-if='ok' :id='a + b'></p>";
    let policy = options(200, 2, LineEnding::Lf);
    let selected = selected(&arena, source, policy);
    let document = observed_native_template_document_with_policy(
        &selected,
        &arena,
        NativeTemplateValuePolicy::FormatConditionalsAndStaticBindings,
    )
    .unwrap();
    let roots: Vec<_> = document
        .binding_operands()
        .iter()
        .map(|operand| core::ptr::from_ref(operand.syntax().expression().unwrap()).cast::<()>())
        .collect();
    let parts = document.into_binding_parts();
    assert!(core::ptr::eq(parts.original, &selected));
    assert_eq!(parts.bindings.len(), 2);
    assert_eq!(parts.attributes.len(), 1);
    assert_eq!(parts.operands.len(), 1);
    assert_eq!(print(&parts.document, &policy.print), expected_body);
    assert_eq!(
        roots,
        parts
            .bindings
            .iter()
            .map(|operand| core::ptr::from_ref(operand.syntax().expression().unwrap()).cast::<()>())
            .collect::<Vec<_>>()
    );
    assert_eq!(parts.bindings[0].raw_value(), "first");
    assert_eq!(parts.bindings[1].raw_value(), "a+b");
    let first = selected.children().next().unwrap().into_element().unwrap();
    let second = selected.children().nth(1).unwrap().into_element().unwrap();
    assert!(
        parts.bindings[0]
            .admitted_for(&selected, first.attributes().next().unwrap())
            .is_some()
    );
    assert!(
        parts.bindings[1]
            .admitted_for(&selected, second.attributes().nth(1).unwrap())
            .is_some()
    );
    assert!(
        parts.attributes[0]
            .admitted_for(&selected, second.attributes().next().unwrap())
            .is_some()
    );
    assert!(
        parts.operands[0]
            .admitted_for(&selected, first.children().next().unwrap())
            .is_some()
    );
    let document = observed_native_template_document_with_policy(
        &selected,
        &arena,
        NativeTemplateValuePolicy::FormatConditionalsAndStaticBindings,
    )
    .unwrap();
    let (original, interpolations, attributes, document): (
        &NativeTemplateComponent<'_>,
        Vec<NativeInterpolationOperand<'_>>,
        Vec<NativeAttributeExpression<'_>>,
        Doc<'_>,
    ) = document.into_full_parts();
    assert!(core::ptr::eq(original, &selected));
    assert_eq!(interpolations.len(), 1);
    assert_eq!(attributes.len(), 1);
    assert_eq!(print(&document, &policy.print), expected_body);
}

#[test]
fn complete_named_failure_transfer_keeps_current_binding_or_actual_failure_and_old_six_part_schema()
{
    for (source, expected, count, actual_failure) in [
        (
            "<template><i :title='first'>{{1n}}</i><p v-if='ok' :id=a+b>{{later}}</p></template>",
            ObservedNativeTemplateRefusal::Document {
                index: 1,
                refusal: NativeTemplateRefusal::UnquotedBindingValue {
                    span: Span::new(55, 58),
                },
            },
            2,
            false,
        ),
        (
            "<template><i :title='first'>{{1n}}</i><p v-if='ok' :id>{{later}}</p></template>",
            ObservedNativeTemplateRefusal::Binding {
                span: Span::new(51, 54),
                index: 1,
                kind: NativeAttributeOperandError::IncompleteValue,
            },
            1,
            true,
        ),
    ] {
        let arena = Allocator::default();
        let policy = options(200, 2, LineEnding::Lf);
        let selected = selected(&arena, source, policy);
        let failure = observed_native_template_document_with_policy(
            &selected,
            &arena,
            NativeTemplateValuePolicy::FormatConditionalsAndStaticBindings,
        )
        .unwrap_err();
        assert_eq!(failure.refusal(), expected);
        let roots: Vec<_> = failure
            .binding_operands()
            .iter()
            .map(|operand| core::ptr::from_ref(operand.syntax().expression().unwrap()).cast::<()>())
            .collect();
        let parts = failure.into_binding_parts();
        assert!(core::ptr::eq(parts.original, &selected));
        assert_eq!(parts.refusal, expected);
        assert_eq!(parts.bindings.len(), count);
        assert_eq!(parts.attributes.len(), 1);
        assert_eq!(parts.operands.len(), 1);
        assert!(parts.attribute_failure.is_none());
        assert!(parts.interpolation_failure.is_none());
        assert_eq!(parts.binding_failure.is_some(), actual_failure);
        if let Some(failure) = &parts.binding_failure {
            assert_eq!(failure.kind(), NativeAttributeOperandError::IncompleteValue);
            assert!(failure.syntax().is_none());
        }
        assert_eq!(
            roots,
            parts
                .bindings
                .iter()
                .map(
                    |operand| core::ptr::from_ref(operand.syntax().expression().unwrap())
                        .cast::<()>()
                )
                .collect::<Vec<_>>()
        );
        assert_eq!(parts.bindings[0].raw_value(), "first");
        let first = selected.children().next().unwrap().into_element().unwrap();
        assert!(
            parts.bindings[0]
                .admitted_for(&selected, first.attributes().next().unwrap())
                .is_some()
        );
        let failure = observed_native_template_document_with_policy(
            &selected,
            &arena,
            NativeTemplateValuePolicy::FormatConditionalsAndStaticBindings,
        )
        .unwrap_err();
        let (
            original,
            interpolations,
            attributes,
            refusal,
            interpolation_failure,
            attribute_failure,
        ): ObservedNativeTemplateFailureParts<'_, '_> = failure.into_full_parts();
        assert!(core::ptr::eq(original, &selected));
        assert_eq!(refusal, expected);
        assert_eq!(interpolations.len(), 1);
        assert_eq!(attributes.len(), 1);
        assert!(interpolation_failure.is_none());
        assert!(attribute_failure.is_none());
    }
}
