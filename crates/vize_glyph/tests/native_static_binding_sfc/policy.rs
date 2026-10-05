use super::{assert_output, options};
use vize_glyph::native_doc::{
    LineEnding, NativeSfcDirectivePolicy, NativeSfcObservation, NativeSfcOptions, NativeSfcRefusal,
    NativeTemplateRefusal, ObservedNativeTemplateRefusal, observe_native_sfc_in,
};
use vize_l0::{Allocator, Span};
use vize_l1::markup::NativeAttributeOperandError;

pub(super) fn empty_refusal(
    owner: &NativeSfcObservation<'_>,
    source: &str,
    policy: NativeSfcOptions,
    refusal: NativeSfcRefusal,
) {
    assert_eq!(owner.source(), source);
    assert!(core::ptr::eq(owner.source(), source));
    assert!(core::ptr::eq(owner.descriptor().source(), source));
    assert_eq!(owner.options(), policy);
    assert_eq!(owner.descriptor().options(), policy.descriptor);
    assert!(owner.binding_operands().is_empty());
    assert!(owner.attribute_operands().is_empty());
    assert!(owner.operands().is_empty());
    assert!(owner.binding_failure().is_none());
    assert!(owner.attribute_failure().is_none());
    assert!(owner.interpolation_failure().is_none());
    for _ in 0..2 {
        assert_eq!(owner.refusal(), Some(refusal));
        assert_eq!(owner.document().unwrap_err(), refusal);
        assert_eq!(owner.format().unwrap_err(), refusal);
    }
}

#[test]
fn default_and_existing_conditional_policy_keep_exact_strict_binding_refusal_and_empty_custody() {
    let source = "<template><p :id='a+b'/></template>";
    assert_eq!(
        NativeSfcOptions::default().directives,
        NativeSfcDirectivePolicy::Refuse
    );
    for directives in [
        NativeSfcDirectivePolicy::Refuse,
        NativeSfcDirectivePolicy::FormatConditionals,
    ] {
        let arena = Allocator::default();
        let mut policy = NativeSfcOptions::default();
        policy.directives = directives;
        let owner = observe_native_sfc_in(&arena, source, policy);
        empty_refusal(
            &owner,
            source,
            policy,
            NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document {
                index: 0,
                refusal: NativeTemplateRefusal::DirectiveValue {
                    span: Span::new(18, 21),
                },
            }),
        );
        assert_eq!(owner.descriptor().issues(), []);
        assert_eq!(owner.descriptor().container().errors.as_slice(), []);
        assert!(owner.selected().is_some());
    }
}

#[test]
fn recognized_unsupported_heads_refuse_before_value_observation_and_other_families_stay_strict() {
    for (source, refusal) in [
        (
            "<template><p :[key]='a+b'/></template>",
            NativeTemplateRefusal::BindingHead {
                span: Span::new(13, 19),
                kind: NativeAttributeOperandError::UnsupportedDirective,
            },
        ),
        (
            "<template><p v-bind='a+b'/></template>",
            NativeTemplateRefusal::BindingHead {
                span: Span::new(13, 19),
                kind: NativeAttributeOperandError::UnsupportedDirective,
            },
        ),
        (
            "<template><p :id.prop='a+b'/></template>",
            NativeTemplateRefusal::BindingHead {
                span: Span::new(13, 21),
                kind: NativeAttributeOperandError::UnsupportedDirective,
            },
        ),
        (
            "<template><p .id='a+b'/></template>",
            NativeTemplateRefusal::BindingHead {
                span: Span::new(13, 16),
                kind: NativeAttributeOperandError::UnsupportedDirective,
            },
        ),
        (
            "<template><p v-show='a+b'/></template>",
            NativeTemplateRefusal::DirectiveValue {
                span: Span::new(21, 24),
            },
        ),
        (
            "<template><p @click='a+b'/></template>",
            NativeTemplateRefusal::DirectiveValue {
                span: Span::new(21, 24),
            },
        ),
    ] {
        let arena = Allocator::default();
        let policy = options(200, 2, LineEnding::Lf);
        let owner = observe_native_sfc_in(&arena, source, policy);
        empty_refusal(
            &owner,
            source,
            policy,
            NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document {
                index: 0,
                refusal,
            }),
        );
        assert_eq!(owner.descriptor().issues(), []);
        assert_eq!(owner.descriptor().container().errors.as_slice(), []);
    }
}

#[test]
fn missing_static_value_preserves_old_bare_header_output_and_new_actual_failure_boundary() {
    let source = "<template><p :id/></template>";
    let expected = "<template><p :id /></template>";
    for directives in [
        NativeSfcDirectivePolicy::Refuse,
        NativeSfcDirectivePolicy::FormatConditionals,
    ] {
        let arena = Allocator::default();
        let mut policy = options(200, 2, LineEnding::Lf);
        policy.directives = directives;
        let owner = assert_output(&arena, source, expected, policy);
        assert!(owner.binding_operands().is_empty());
    }
    let arena = Allocator::default();
    let policy = options(200, 2, LineEnding::Lf);
    let owner = observe_native_sfc_in(&arena, source, policy);
    let expected = NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Binding {
        span: Span::new(13, 16),
        index: 0,
        kind: NativeAttributeOperandError::IncompleteValue,
    });
    assert_eq!(owner.source(), source);
    assert_eq!(owner.options(), policy);
    assert_eq!(owner.refusal(), Some(expected));
    assert_eq!(owner.document().unwrap_err(), expected);
    assert_eq!(owner.format().unwrap_err(), expected);
    assert!(owner.binding_operands().is_empty());
    assert!(owner.attribute_operands().is_empty());
    assert!(owner.operands().is_empty());
    let failure = owner.binding_failure().unwrap();
    assert_eq!(failure.kind(), NativeAttributeOperandError::IncompleteValue);
    assert!(failure.syntax().is_none());
    assert!(owner.attribute_failure().is_none());
    assert!(owner.interpolation_failure().is_none());
}
