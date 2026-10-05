use super::{assert_original, options};
use vize_glyph::native_doc::{
    LineEnding, NativeSfcDirectivePolicy, NativeSfcOptions, NativeSfcRefusal,
    NativeTemplateRefusal, ObservedNativeTemplateRefusal, observe_native_sfc_in,
};
use vize_l0::{Allocator, Span};

#[test]
fn default_strict_policy_keeps_the_original_complete_directive_value_refusal() {
    let source = "<template><p v-if='a+b'>{{1n}}</p></template>";
    let policy = NativeSfcOptions::default();
    assert_eq!(policy.directives, NativeSfcDirectivePolicy::Refuse);
    let arena = Allocator::default();
    let owner = observe_native_sfc_in(&arena, source, policy);
    assert_original(&owner, source, policy);
    let refusal = NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document {
        index: 0,
        refusal: NativeTemplateRefusal::DirectiveValue {
            span: Span::new(19, 22),
        },
    });
    for _ in 0..2 {
        assert_eq!(owner.refusal(), Some(refusal));
        assert_eq!(owner.document().unwrap_err(), refusal);
        assert_eq!(owner.format().unwrap_err(), refusal);
        assert!(owner.attribute_operands().is_empty());
        assert!(owner.operands().is_empty());
        assert!(owner.attribute_failure().is_none());
        assert!(owner.interpolation_failure().is_none());
    }
    let element = owner
        .selected()
        .unwrap()
        .children()
        .next()
        .unwrap()
        .into_element()
        .unwrap();
    assert_eq!(
        element
            .attributes()
            .next()
            .unwrap()
            .surface()
            .value
            .as_ref()
            .unwrap()
            .content
            .text,
        "a+b"
    );
    assert_eq!(element.children().len(), 1);
}

#[test]
fn explicit_conditional_policy_still_refuses_unknown_v_show_and_bound_values_whole() {
    for (source, span, name) in [
        (
            "<template><p v-show='a+b'>{{1n}}</p></template>",
            Span::new(21, 24),
            "v-show",
        ),
        (
            "<template><p :id='a+b'>{{1n}}</p></template>",
            Span::new(18, 21),
            ":id",
        ),
    ] {
        let arena = Allocator::default();
        let policy = options(200, 2, LineEnding::CrLf);
        let owner = observe_native_sfc_in(&arena, source, policy);
        assert_original(&owner, source, policy);
        assert_eq!(span.slice(source), "a+b");
        let refusal = NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document {
            index: 0,
            refusal: NativeTemplateRefusal::DirectiveValue { span },
        });
        for _ in 0..2 {
            assert_eq!(owner.refusal(), Some(refusal));
            assert_eq!(owner.document().unwrap_err(), refusal);
            assert_eq!(owner.format().unwrap_err(), refusal);
            assert!(owner.attribute_operands().is_empty());
            assert!(owner.operands().is_empty());
            assert!(owner.attribute_failure().is_none());
            assert!(owner.interpolation_failure().is_none());
        }
        let element = owner
            .selected()
            .unwrap()
            .children()
            .next()
            .unwrap()
            .into_element()
            .unwrap();
        let attribute = element.attributes().next().unwrap();
        assert_eq!(attribute.surface().name.text, name);
        assert_eq!(
            attribute.surface().value.as_ref().unwrap().content.text,
            "a+b"
        );
        assert_eq!(element.children().len(), 1);
    }
}
