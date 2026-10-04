use super::{assert_bindings, assert_output, options};
use oxc_span::GetSpan;
use std::panic::{AssertUnwindSafe, catch_unwind};
use vize_glyph::native_doc::{LineEnding, NativeSfcObservation, observe_native_sfc_in};
use vize_l0::{Allocator, Span};
use vize_l1::embed::DecodeSegmentKind;
use vize_l1::markup::NativeAttributeBindingExpression;

#[test]
fn whole_owner_moves_keep_all_three_vectors_actual_ast_roots_and_reject_foreign_equal_source() {
    let arena = Allocator::default();
    let source = "<template><p :id='a&#43;1n' v-if='ok'>{{2n}}</p><p :id='a&#43;1n'/></template>";
    let expected =
        "<template><p :id='a &#43; 1n' v-if='ok'>{{ 2n }}</p><p :id='a &#43; 1n' /></template>";
    let policy = options(200, 2, LineEnding::Lf);
    let owner = assert_output(&arena, source, expected, policy);
    let identity = |owner: &NativeSfcObservation<'_>| {
        (
            owner.source().as_ptr(),
            owner.options(),
            owner.descriptor().container().blocks.as_ptr().cast::<()>(),
            owner.binding_operands().as_ptr().cast::<()>(),
            owner.attribute_operands().as_ptr().cast::<()>(),
            owner.operands().as_ptr().cast::<()>(),
            owner
                .binding_operands()
                .iter()
                .map(|operand| {
                    (
                        operand.name_span(),
                        operand.argument_span(),
                        operand.value_span(),
                        operand.raw_value().as_ptr(),
                        core::ptr::from_ref(operand.syntax().expression().unwrap()).cast::<()>(),
                        operand.syntax().source().text().as_ptr(),
                    )
                })
                .collect::<Vec<_>>(),
        )
    };
    let before = identity(&owner);
    let mut parked = vec![owner];
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
    assert_bindings(&owner);
    assert_eq!(owner.binding_operands().len(), 2);
    assert_eq!(owner.attribute_operands().len(), 1);
    assert_eq!(owner.operands().len(), 1);
    let selected = owner.selected().unwrap();
    let sibling = selected.children().nth(1).unwrap().into_element().unwrap();
    let first = &owner.binding_operands()[0];
    assert!(
        first
            .admitted_for(selected, sibling.attributes().next().unwrap())
            .is_none()
    );
    let other = observe_native_sfc_in(&arena, source, policy);
    let copy = source.to_owned();
    let copied = observe_native_sfc_in(&arena, &copy, policy);
    assert!(!core::ptr::eq(copy.as_str(), source));
    for foreign in [&other, &copied] {
        let selected = foreign.selected().unwrap();
        let element = selected.children().next().unwrap().into_element().unwrap();
        assert!(
            first
                .admitted_for(selected, element.attributes().next().unwrap())
                .is_none()
        );
    }
    assert!(core::mem::needs_drop::<NativeSfcObservation<'_>>());
    assert!(core::mem::needs_drop::<NativeAttributeBindingExpression<'_>>());
    assert_eq!(identity(&owner), before);
}

#[test]
fn normal_drop_forget_and_callback_unwind_preserve_authentic_arena_source_map_and_comment_views() {
    let arena = Allocator::default();
    let source = "<template><p :id='/*x*/a&#43;1n' v-if='ok'>{{2n}}</p></template>";
    let expected = "<template><p :id='/*x*/a &#43; 1n' v-if='ok'>{{ 2n }}</p></template>";
    let policy = options(200, 2, LineEnding::Lf);
    let (root, source_view, comment) = {
        let owner = assert_output(&arena, source, expected, policy);
        let syntax = owner.binding_operands()[0].syntax();
        (
            syntax.expression().unwrap(),
            syntax.source(),
            syntax.comments().next().unwrap().text().unwrap(),
        )
    };
    assert_eq!(source_view.text(), "/*x*/a+1n");
    assert_eq!(source_view.span(), Span::new(18, 31));
    assert_eq!(comment, "/*x*/");
    assert!(core::ptr::eq(source_view.authored_root(), source));
    assert_eq!(root.span(), oxc_span::Span::new(7, 11));
    assert_eq!(
        source_view.authored_span(Span::new(5, 9)),
        Ok(Span::new(23, 31))
    );
    let map = source_view.decode_map().unwrap().segments();
    assert_eq!(
        map.iter()
            .map(|segment| (segment.decoded(), segment.authored(), segment.kind()))
            .collect::<Vec<_>>(),
        [
            (
                Span::new(0, 6),
                Span::new(18, 24),
                DecodeSegmentKind::Identity
            ),
            (
                Span::new(6, 7),
                Span::new(24, 29),
                DecodeSegmentKind::Entity
            ),
            (
                Span::new(7, 9),
                Span::new(29, 31),
                DecodeSegmentKind::Identity
            ),
        ]
    );
    let snapshot = map.to_vec();
    let owner = observe_native_sfc_in(&arena, source, policy);
    assert_eq!(owner.format().unwrap().code, expected);
    let result: Result<(), _> = catch_unwind(AssertUnwindSafe(move || {
        let _owner = core::hint::black_box(owner);
        panic!("fixture whole binding owner unwind");
    }));
    assert!(result.is_err());
    assert_eq!(source_view.text(), "/*x*/a+1n");
    assert_eq!(
        source_view.decode_map().unwrap().segments(),
        snapshot.as_slice()
    );
    assert_eq!(
        source_view.decode_map().unwrap().segments().as_ptr(),
        map.as_ptr()
    );
    let next = assert_output(&arena, source, expected, policy);
    assert_eq!(next.binding_operands().len(), 1);
    assert!(!core::ptr::eq(
        root,
        next.binding_operands()[0].syntax().expression().unwrap()
    ));
    let forgotten_source = next.binding_operands()[0].syntax().source();
    let forgotten_root = next.binding_operands()[0].syntax().expression().unwrap();
    let forgotten_map = forgotten_source.decode_map().unwrap().segments().as_ptr();
    core::mem::forget(next);
    assert_eq!(forgotten_source.text(), "/*x*/a+1n");
    assert_eq!(forgotten_source.span(), Span::new(18, 31));
    assert!(core::ptr::eq(forgotten_source.authored_root(), source));
    assert_eq!(forgotten_root.span(), oxc_span::Span::new(7, 11));
    assert_eq!(
        forgotten_source.decode_map().unwrap().segments(),
        snapshot.as_slice()
    );
    assert_eq!(
        forgotten_source.decode_map().unwrap().segments().as_ptr(),
        forgotten_map
    );
}
