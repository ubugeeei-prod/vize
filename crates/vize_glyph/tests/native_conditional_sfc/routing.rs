//! Existing strict/opaque contracts retain their original negative visit order.

use super::{assert_output, options};
use vize_glyph::native_doc::{
    LineEnding, NativeSfcDirectivePolicy, NativeSfcOptions, NativeSfcRefusal,
    NativeTemplateRefusal, NativeTemplateValuePolicy, ObservedNativeTemplateRefusal,
    TemplateRefusal, UnsupportedSyntax, observe_native_sfc_in,
    observed_native_template_document_with_policy, print,
};
use vize_l0::{Allocator, Span};
use vize_l1::markup::NativeInterpolationError;

#[test]
fn original_v_pre_negative_route_preserves_opaque_bytes_and_strict_value_refusal() {
    let source = "<template><p v-pre v-if='a+b'>{{1n}}</p></template>";
    let arena = Allocator::default();
    let policy = options(200, 2, LineEnding::Lf);
    let owner = observe_native_sfc_in(&arena, source, policy);
    let refusal = NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document {
        index: 0,
        refusal: NativeTemplateRefusal::DirectiveValue {
            span: Span::new(25, 28),
        },
    });
    assert_eq!(owner.refusal(), Some(refusal));
    assert_eq!(owner.format().unwrap_err(), refusal);
    assert!(owner.operands().is_empty());
    assert!(owner.attribute_operands().is_empty());
    assert!(owner.attribute_failure().is_none());
    let selected = owner.selected().unwrap();
    assert_eq!(
        selected.component().block().source(),
        "<p v-pre v-if='a+b'>{{1n}}</p>"
    );
    let opaque = observed_native_template_document_with_policy(
        selected,
        &arena,
        NativeTemplateValuePolicy::PreserveOpaque,
    )
    .unwrap();
    assert_eq!(
        print(opaque.document(), &policy.print),
        "<p v-pre v-if='a+b'>{{1n}}</p>"
    );
    assert!(opaque.operands().is_empty());
    assert!(opaque.attribute_operands().is_empty());
    let strict = observed_native_template_document_with_policy(
        selected,
        &arena,
        NativeTemplateValuePolicy::RefuseValuedDirectives,
    )
    .unwrap_err();
    assert_eq!(
        strict.refusal(),
        match refusal {
            NativeSfcRefusal::Template(value) => value,
            _ => panic!("template"),
        }
    );
    assert!(strict.attribute_operands().is_empty());
    let source = "<template><p v-pre>{{a+b}}</p></template>";
    let owner = assert_output(&arena, source, source, policy);
    assert!(owner.attribute_operands().is_empty());
    assert!(owner.operands().is_empty());
}

#[test]
fn recovered_header_static_and_valued_refusal_precedence_matches_unchanged_strict_route() {
    let sources = [
        "<template><p id='x'>{{1n}}</template>",
        "<template><p v-show='x'></template>",
    ];
    let expected = [
        NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Interpolation {
            offset: 10,
            index: 0,
            kind: NativeInterpolationError::RecoveredComponent,
        }),
        NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document {
            index: 0,
            refusal: NativeTemplateRefusal::DirectiveValue {
                span: Span::new(21, 22),
            },
        }),
    ];
    for (source, expected) in sources.into_iter().zip(expected) {
        for directives in [
            NativeSfcDirectivePolicy::Refuse,
            NativeSfcDirectivePolicy::FormatConditionals,
        ] {
            let arena = Allocator::default();
            let mut policy = NativeSfcOptions::default();
            policy.directives = directives;
            let owner = observe_native_sfc_in(&arena, source, policy);
            assert_eq!(owner.source(), source);
            assert_eq!(owner.options(), policy);
            assert_eq!(owner.refusal(), Some(expected));
            assert_eq!(owner.document().unwrap_err(), expected);
            assert_eq!(owner.format().unwrap_err(), expected);
            assert!(owner.attribute_operands().is_empty());
            assert!(owner.operands().is_empty());
            assert!(owner.attribute_failure().is_none());
            if matches!(
                expected,
                NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Interpolation { .. })
            ) {
                let failure = owner.interpolation_failure().unwrap();
                assert_eq!(failure.kind(), NativeInterpolationError::RecoveredComponent);
                assert!(failure.syntax().is_none());
            } else {
                assert!(owner.interpolation_failure().is_none());
            }
        }
    }
}

#[test]
fn original_malformed_name_keeps_selected_local_offset_under_unicode_outer_shift() {
    for prefix in ["", "<!--é-->"] {
        let source = format!("{prefix}<template>{{{{1n}}}}<p v->{{{{later}}}}</p></template>");
        let arena = Allocator::default();
        let owner = observe_native_sfc_in(&arena, &source, options(200, 2, LineEnding::Lf));
        let refusal = NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document {
            index: 1,
            refusal: NativeTemplateRefusal::Template(TemplateRefusal::Unsupported {
                offset: 9,
                syntax: UnsupportedSyntax::Directive,
            }),
        });
        assert_eq!(owner.refusal(), Some(refusal));
        assert_eq!(owner.format().unwrap_err(), refusal);
        assert_eq!(owner.source(), source);
        assert_eq!(owner.operands().len(), 1);
        assert_eq!(owner.operands()[0].raw_content(), "1n");
        assert!(owner.attribute_operands().is_empty());
        assert!(owner.attribute_failure().is_none());
        let selected = owner.selected().unwrap();
        assert_eq!(
            selected.component().block().source(),
            "{{1n}}<p v->{{later}}</p>"
        );
        let element = selected.children().nth(1).unwrap().into_element().unwrap();
        let head = selected
            .observe_attribute_head(element.attributes().next().unwrap())
            .unwrap();
        assert_eq!(head.name_block().source(), "v-");
        assert_eq!(
            head.name_block().span(),
            Span::new(prefix.len() as u32 + 19, prefix.len() as u32 + 21)
        );
    }
}

#[test]
fn later_unimplemented_valued_heads_keep_current_conditional_before_refusing_body() {
    for name in [
        "v-if.foo",
        "v-if:arg",
        "v-if:[key]",
        "@click",
        "v-for",
        "#slot",
    ] {
        let source =
            format!("<template>{{{{1n}}}}<p v-if='a+b' {name}='a'>{{{{later}}}}</p></template>");
        let arena = Allocator::default();
        let owner = observe_native_sfc_in(&arena, &source, options(200, 2, LineEnding::Lf));
        // Authored from the complete fixture concatenation, not observed output.
        let start = 25 + 3 + 2 + name.len() as u32 + 2;
        let span = Span::new(start, start + 1);
        assert_eq!(span.slice(&source), "a");
        let refusal = NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document {
            index: 1,
            refusal: NativeTemplateRefusal::DirectiveValue { span },
        });
        assert_eq!(owner.refusal(), Some(refusal));
        assert_eq!(owner.format().unwrap_err(), refusal);
        assert_eq!(owner.source(), source);
        assert_eq!(owner.operands().len(), 1);
        assert_eq!(owner.attribute_operands().len(), 1);
        assert!(owner.attribute_failure().is_none());
        let current = &owner.attribute_operands()[0];
        assert_eq!(current.raw_value(), "a+b");
        let selected = owner.selected().unwrap();
        let element = selected.children().nth(1).unwrap().into_element().unwrap();
        let first = element.attributes().next().unwrap();
        let second = element.attributes().nth(1).unwrap();
        assert!(current.admitted_for(selected, first).is_some());
        assert!(current.admitted_for(selected, second).is_none());
        assert_eq!(element.children().len(), 1);
    }
}
