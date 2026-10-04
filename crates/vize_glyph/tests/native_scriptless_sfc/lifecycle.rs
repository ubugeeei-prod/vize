//! Ordinary full-owner drop, unwind and forget retain only genuine arena/source views.

use oxc_span::GetSpan;
use std::panic::{AssertUnwindSafe, catch_unwind};
use vize_glyph::native_doc::{
    ExpressionRefusal, LineEnding, NativeSfcObservation, NativeSfcOptions, NativeSfcRefusal,
    NativeTemplateRefusal, ObservedNativeTemplateRefusal, observe_native_sfc_in,
};
use vize_l0::{Allocator, Span};

fn options() -> NativeSfcOptions {
    let mut options = NativeSfcOptions::default();
    options.print.width = 200;
    options.print.line_ending = LineEnding::CrLf;
    options
}

#[test]
fn normal_full_owner_drop_retains_only_authentic_arena_ast_and_authored_source_views() {
    assert!(core::mem::needs_drop::<NativeSfcObservation<'_>>());
    let arena = Allocator::default();
    let source = "<!--前-->\r\n<template>{{/*x*/a&#43;1n}}</template>\r\n";
    let expected = "<!--前-->\r\n<template>{{ /*x*/a &#43; 1n }}</template>\r\n";
    let (root, view, comment_text, decoded_span) = {
        let owner = observe_native_sfc_in(&arena, source, options());
        let result = owner.format().unwrap();
        assert_eq!(result.code, expected);
        assert!(result.changed);
        let syntax = owner.operands()[0].syntax();
        let root = syntax.expression().unwrap();
        let view = syntax.source();
        let comment_text = syntax.comments().next().unwrap().text().unwrap();
        let decoded_span = syntax.decoded_span(root.span()).unwrap();
        (root, view, comment_text, decoded_span)
    };
    assert_eq!(view.text(), "/*x*/a+1n");
    assert_eq!(view.decode_map().unwrap().segments().len(), 3);
    assert!(core::ptr::eq(view.authored_root(), source));
    assert_eq!(comment_text, "/*x*/");
    assert_eq!(decoded_span, Span::new(5, 9));
    assert_eq!(
        view.authored_span(decoded_span).unwrap().slice(source),
        "a&#43;1n"
    );
    assert_eq!(root.span().size(), 4);
    let next = observe_native_sfc_in(&arena, source, options());
    assert_eq!(next.options(), options());
    assert_eq!(next.format().unwrap().code, expected);
    assert!(next.format().unwrap().changed);
    // These surviving source/arena references carry no completed-owner admission.
}

#[test]
fn actual_complete_and_refused_owner_unwind_keeps_source_maps_and_genuine_next_observation() {
    let refused = NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document {
        index: 1,
        refusal: NativeTemplateRefusal::Expression {
            offset: 9,
            refusal: ExpressionRefusal::UnsupportedNode {
                span: Span::new(2, 9),
            },
        },
    });
    for (body, decoded, refusal) in [
        ("{{1n}}<p>{{a&#43;2n}}</p>", "a+2n", None),
        (
            "{{1n}}<p>{{a&#43;f(...b)}}</p>{{later}}",
            "a+f(...b)",
            Some(refused),
        ),
    ] {
        let arena = Allocator::default();
        let source = vize_l0::cstr!("<!--前-->\r\n<template>{body}</template>\r\n");
        let owner = observe_native_sfc_in(&arena, &source, options());
        assert_eq!(owner.refusal(), refusal);
        assert_eq!(owner.operands().len(), 2);
        let root = owner.operands()[1].syntax().expression().unwrap();
        let root_span = root.span();
        let view = owner.operands()[1].syntax().source();
        let map = view.decode_map().unwrap().segments();
        let map_values = map.to_vec();
        let prefix = owner
            .operands()
            .iter()
            .map(|operand| (operand.full_span(), operand.content_span()))
            .collect::<Vec<_>>();
        let result: Result<(), _> = catch_unwind(AssertUnwindSafe(move || {
            let _owner = core::hint::black_box(owner);
            panic!("unwind the actual original full SFC owner");
        }));
        assert!(result.is_err());
        assert_eq!(root.span(), root_span);
        assert_eq!(view.text(), decoded);
        assert_eq!(view.decode_map().unwrap().segments().as_ptr(), map.as_ptr());
        assert_eq!(view.decode_map().unwrap().segments(), map_values.as_slice());
        assert!(core::ptr::eq(view.authored_root(), source.as_str()));
        let next = observe_native_sfc_in(&arena, &source, options());
        assert_eq!(next.refusal(), refusal);
        assert_eq!(next.options(), options());
        assert!(core::ptr::eq(next.source(), source.as_str()));
        assert_eq!(next.selected().unwrap().component().block().source(), body);
        assert_eq!(
            next.operands()
                .iter()
                .map(|operand| (operand.full_span(), operand.content_span()))
                .collect::<Vec<_>>(),
            prefix
        );
        assert_eq!(next.operands()[1].syntax().source().text(), decoded);
        assert_eq!(
            next.operands()[1]
                .syntax()
                .source()
                .decode_map()
                .unwrap()
                .segments(),
            map_values.as_slice()
        );
        assert!(next.interpolation_failure().is_none());
        if let Some(refusal) = refusal {
            assert_eq!(next.refusal(), Some(refusal));
            assert!(matches!(next.document(), Err(actual) if actual == refusal));
            assert!(matches!(next.format(), Err(actual) if actual == refusal));
        } else {
            let expected = "<!--前-->\r\n<template>{{ 1n }}<p>{{ a &#43; 2n }}</p></template>\r\n";
            for _ in 0..3 {
                let result = next.format().unwrap();
                assert_eq!(result.code, expected);
                assert!(result.changed);
            }
        }
    }
    // Ordinary unwind is exercised, without instrumented destructor or parse-count claims.
}

#[test]
fn deliberately_forgetting_full_owner_preserves_only_real_arena_source_and_result_data() {
    let arena = Allocator::default();
    let source = "<!--前--><template>{{a&#43;1n}}</template>\n";
    let expected = "<!--前--><template>{{ a &#43; 1n }}</template>\n";
    let owner = observe_native_sfc_in(&arena, source, options());
    let result = owner.format().unwrap();
    let root = owner.operands()[0].syntax().expression().unwrap();
    let root_span = root.span();
    let view = owner.operands()[0].syntax().source();
    let map = view.decode_map().unwrap().segments();
    let map_values = map.to_vec();
    core::mem::forget(owner);
    // This intentionally leaks the real normal owner; no receipt or owner is fabricated.
    assert_eq!(root.span(), root_span);
    assert_eq!(view.text(), "a+1n");
    assert_eq!(view.decode_map().unwrap().segments().as_ptr(), map.as_ptr());
    assert_eq!(view.decode_map().unwrap().segments(), map_values.as_slice());
    assert!(core::ptr::eq(view.authored_root(), source));
    assert_eq!(result.code, expected);
    assert!(result.changed);
    let next = observe_native_sfc_in(&arena, source, options());
    let selected = next.selected().unwrap();
    assert!(
        next.operands()[0]
            .admitted_for(selected, selected.children().next().unwrap())
            .is_some()
    );
    assert_eq!(next.options(), options());
    assert_eq!(next.format().unwrap().code, expected);
    assert!(next.format().unwrap().changed);
    // The old owned result string is data; a new full owner comes only from genuine observation.
}
