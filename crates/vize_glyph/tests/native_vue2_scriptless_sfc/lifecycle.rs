//! Real original owners drop and unwind; surviving arena views grant no receipt.

use super::*;
use oxc_span::GetSpan;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn complete_whole_owner_drop_keeps_only_original_arena_ast_and_authored_mapping() {
    assert!(core::mem::needs_drop::<NativeVue2SfcObservation<'_>>());
    let source = "<!--前--><template><div>{{a&#43;1}}</div></template><!--尾-->";
    let arena = Allocator::default();
    let (root, source_view, span, printed) = {
        let owner = observe_native_vue2_sfc_in(&arena, source, options(200, 2, LineEnding::Lf));
        let syntax = owner.descriptor().component().unwrap().bindings()[0]
            .admitted()
            .unwrap()
            .base();
        let root = syntax.expression().unwrap();
        (
            root,
            syntax.source(),
            syntax.decoded_span(root.span()).unwrap(),
            owner.format().unwrap().code,
        )
    };
    assert_eq!(source_view.text(), "a+1");
    assert_eq!(
        source_view.authored_span(span).unwrap().slice(source),
        "a&#43;1"
    );
    assert!(core::ptr::eq(source_view.authored_root(), source));
    assert_eq!(root.span().size(), 3);
    assert_eq!(
        printed,
        "<!--前--><template><div>{{a &#43; 1}}</div></template><!--尾-->"
    );
    let next = observe_native_vue2_sfc_in(&arena, source, options(200, 2, LineEnding::Lf));
    assert_eq!(next.format().unwrap().code, printed);
}

#[test]
fn complete_and_refused_whole_owner_unwind_preserve_real_source_and_next_observation() {
    for body in ["<div>{{a&#43;1}}</div>", "<div>{{a&#43;1}}{{b+}}</div>"] {
        let source = format!("<!--前--><template>{body}</template><!--尾-->");
        let arena = Allocator::default();
        let owner = observe_native_vue2_sfc_in(&arena, &source, NativeVue2SfcOptions::default());
        let refusal = owner.refusal();
        assert_eq!(refusal.is_some(), body.contains("b+"));
        let component = owner.descriptor().component().unwrap();
        let syntax = component.bindings()[0].chain().unwrap().base();
        let root = syntax.expression().unwrap();
        let span = root.span();
        let view = syntax.source();
        let maps = view.decode_map().unwrap().segments();
        let values = maps.to_vec();
        let caught: Result<(), _> = catch_unwind(AssertUnwindSafe(move || {
            let _owner = core::hint::black_box(owner);
            panic!("unwind real full Vue2 owner");
        }));
        assert!(caught.is_err());
        assert_eq!(root.span(), span);
        assert_eq!(view.text(), "a+1");
        assert_eq!(
            view.decode_map().unwrap().segments().as_ptr(),
            maps.as_ptr()
        );
        assert_eq!(maps, values.as_slice());
        assert!(core::ptr::eq(view.authored_root(), source.as_str()));
        let next = observe_native_vue2_sfc_in(&arena, &source, NativeVue2SfcOptions::default());
        assert_eq!(next.refusal(), refusal);
        assert_eq!(
            next.descriptor().component().unwrap().bindings().len(),
            if refusal.is_some() { 2 } else { 1 }
        );
        if let Some(refusal) = refusal {
            assert_eq!(next.document().unwrap_err(), refusal);
        } else {
            assert_eq!(
                next.format().unwrap().code,
                "<!--前--><template><div>{{a &#43; 1}}</div></template><!--尾-->"
            );
        }
    }
}
