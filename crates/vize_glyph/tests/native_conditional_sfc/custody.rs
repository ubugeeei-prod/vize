//! Normal original ownership survives move, transfer, drop and caught unwind.

use super::{assert_output, options};
use oxc_span::GetSpan;
use std::panic::{AssertUnwindSafe, catch_unwind};
use vize_glyph::native_doc::{
    LineEnding, NativeSfcObservation, NativeSfcRefusal, NativeTemplateRefusal,
    NativeTemplateValuePolicy, ObservedNativeTemplateRefusal, observe_native_sfc_in,
    observed_native_template_document_with_policy, print,
};
use vize_l0::{Allocator, Span};
use vize_l1::markup::{NativeAttributeExpression, NativeTemplateComponent};

fn first_attribute<'p, 'a>(
    selected: &'p NativeTemplateComponent<'a>,
) -> vize_l1::markup::NativeAttribute<'p, 'a> {
    selected
        .children()
        .next()
        .unwrap()
        .into_element()
        .unwrap()
        .attributes()
        .next()
        .unwrap()
}

#[test]
fn actual_full_owner_moves_keep_both_original_prefix_vectors_and_reject_equal_foreign_source() {
    let arena = Allocator::default();
    let source = "<template><p v-if='a+b'>{{1n}}</p><p v-else-if='a+b'>{{2n}}</p></template>";
    let expected =
        "<template><p v-if='a + b'>{{ 1n }}</p><p v-else-if='a + b'>{{ 2n }}</p></template>";
    let policy = options(200, 2, LineEnding::Lf);
    let owner = assert_output(&arena, source, expected, policy);
    let identity = |owner: &NativeSfcObservation<'_>| {
        (
            owner.source().as_ptr(),
            owner.options(),
            owner.descriptor().container().blocks.as_ptr(),
            owner.operands().as_ptr(),
            owner.attribute_operands().as_ptr(),
            owner
                .attribute_operands()
                .iter()
                .map(|operand| {
                    (
                        operand.name_span(),
                        operand.value_span(),
                        operand.raw_value().as_ptr(),
                        core::ptr::from_ref(operand.syntax().expression().unwrap()),
                        operand.syntax().source().text().as_ptr(),
                    )
                })
                .collect::<Vec<_>>(),
        )
    };
    let before = identity(&owner);
    let mut parked = Vec::new();
    parked.push(owner);
    for _ in 0..32 {
        parked.push(observe_native_sfc_in(
            &arena,
            "<template></template>",
            policy,
        ));
    }
    let owner = Box::new(parked.remove(0));
    assert_eq!(identity(&owner), before);
    assert_eq!(owner.format().unwrap().code, expected);
    let selected = owner.selected().unwrap();
    let first = first_attribute(selected);
    let second = selected
        .children()
        .nth(1)
        .unwrap()
        .into_element()
        .unwrap()
        .attributes()
        .next()
        .unwrap();
    let first_owner = &owner.attribute_operands()[0];
    assert!(first_owner.admitted_for(selected, first).is_some());
    assert!(first_owner.admitted_for(selected, second).is_none());
    let other = observe_native_sfc_in(&arena, source, policy);
    assert!(
        first_owner
            .admitted_for(
                other.selected().unwrap(),
                first_attribute(other.selected().unwrap())
            )
            .is_none()
    );
    assert!(core::mem::needs_drop::<NativeSfcObservation<'_>>());
    assert!(core::mem::needs_drop::<NativeAttributeExpression<'_>>());
    assert_eq!(identity(&owner), before);
}

#[test]
fn public_full_transfers_preserve_original_conditional_owners_after_short_document_drop() {
    let arena = Allocator::default();
    let source = "<template><p v-if='a&#43;1n'>{{2n}}</p></template>";
    let expected = "<template><p v-if='a &#43; 1n'>{{ 2n }}</p></template>";
    let policy = options(200, 2, LineEnding::Lf);
    let owner = assert_output(&arena, source, expected, policy);
    let selected = owner.selected().unwrap();
    let document = observed_native_template_document_with_policy(
        selected,
        &arena,
        NativeTemplateValuePolicy::FormatConditionals,
    )
    .unwrap();
    let roots = document
        .attribute_operands()
        .iter()
        .map(|operand| core::ptr::from_ref(operand.syntax().expression().unwrap()))
        .collect::<Vec<_>>();
    let (original, interpolations, attributes, document) = document.into_full_parts();
    assert!(core::ptr::eq(original, selected));
    assert_eq!(interpolations.len(), 1);
    assert_eq!(attributes.len(), 1);
    assert_eq!(
        print(&document, &policy.print),
        "<p v-if='a &#43; 1n'>{{ 2n }}</p>"
    );
    assert_eq!(
        roots,
        attributes
            .iter()
            .map(|operand| core::ptr::from_ref(operand.syntax().expression().unwrap()))
            .collect::<Vec<_>>()
    );
    assert_eq!(attributes[0].raw_value(), "a&#43;1n");
    assert!(
        attributes[0]
            .admitted_for(selected, first_attribute(selected))
            .is_some()
    );

    let source = "<template><p v-if='a&#43;1n' :id='x'>{{later}}</p></template>";
    let owner = observe_native_sfc_in(&arena, source, policy);
    let selected = owner.selected().unwrap();
    let failure = observed_native_template_document_with_policy(
        selected,
        &arena,
        NativeTemplateValuePolicy::FormatConditionals,
    )
    .unwrap_err();
    let expected_refusal = ObservedNativeTemplateRefusal::Document {
        index: 0,
        refusal: NativeTemplateRefusal::DirectiveValue {
            span: Span::new(34, 35),
        },
    };
    assert_eq!(
        owner.refusal(),
        Some(NativeSfcRefusal::Template(expected_refusal))
    );
    assert_eq!(failure.refusal(), expected_refusal);
    let roots = failure
        .attribute_operands()
        .iter()
        .map(|operand| core::ptr::from_ref(operand.syntax().expression().unwrap()))
        .collect::<Vec<_>>();
    let (original, interpolations, attributes, refusal, interpolation_failure, attribute_failure) =
        failure.into_full_parts();
    assert!(core::ptr::eq(original, selected));
    assert_eq!(refusal, expected_refusal);
    assert!(interpolations.is_empty());
    assert_eq!(attributes.len(), 1);
    assert!(interpolation_failure.is_none());
    assert!(attribute_failure.is_none());
    assert_eq!(
        roots,
        attributes
            .iter()
            .map(|operand| core::ptr::from_ref(operand.syntax().expression().unwrap()))
            .collect::<Vec<_>>()
    );
    assert_eq!(attributes[0].raw_value(), "a&#43;1n");
    assert!(
        attributes[0]
            .admitted_for(selected, first_attribute(selected))
            .is_some()
    );
}

#[test]
fn normal_drop_and_unwind_keep_only_authentic_arena_ast_map_and_source_views() {
    let arena = Allocator::default();
    let source = "<template><p v-if='/*x*/a&#43;1n'>{{2n}}</p></template>";
    let expected = "<template><p v-if='/*x*/a &#43; 1n'>{{ 2n }}</p></template>";
    let policy = options(200, 2, LineEnding::Lf);
    let (root, view, comment) = {
        let owner = assert_output(&arena, source, expected, policy);
        let syntax = owner.attribute_operands()[0].syntax();
        (
            syntax.expression().unwrap(),
            syntax.source(),
            syntax.comments().next().unwrap().text().unwrap(),
        )
    };
    assert_eq!(view.text(), "/*x*/a+1n");
    assert_eq!(comment, "/*x*/");
    assert!(core::ptr::eq(view.authored_root(), source));
    assert_eq!(root.span().size(), 4);
    assert_eq!(
        view.authored_span(Span::new(5, 9)).unwrap().slice(source),
        "a&#43;1n"
    );
    let map = view.decode_map().unwrap().segments();
    let snapshot = map.to_vec();
    let owner = observe_native_sfc_in(&arena, source, policy);
    assert_eq!(owner.format().unwrap().code, expected);
    let result: Result<(), _> = catch_unwind(AssertUnwindSafe(move || {
        let _owner = core::hint::black_box(owner);
        panic!("drop the original conditional whole owner during unwind");
    }));
    assert!(result.is_err());
    assert_eq!(view.text(), "/*x*/a+1n");
    assert_eq!(view.decode_map().unwrap().segments(), snapshot.as_slice());
    assert_eq!(view.decode_map().unwrap().segments().as_ptr(), map.as_ptr());
    let next = assert_output(&arena, source, expected, policy);
    assert_eq!(next.attribute_operands().len(), 1);
    assert!(!core::ptr::eq(
        root,
        next.attribute_operands()[0].syntax().expression().unwrap()
    ));
}
