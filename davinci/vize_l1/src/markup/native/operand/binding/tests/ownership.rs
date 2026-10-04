use super::{assert_original, observe, selected};
use crate::embed::{DecodeSegmentKind, syntax::EmbedHole};
use crate::markup::{NativeAttributeBindingExpression, NativeAttributeOperandError};
use alloc::vec::Vec;
use oxc_ast::ast::CommentKind;
use vize_l0::{Allocator, Span};

#[test]
fn sibling_nested_same_buffer_and_equal_byte_copy_attributes_cannot_join_the_original_binding() {
    let arena = Allocator::default();
    let source = "<template><p :id='same' :title='same'><i :id='same'/></p></template>";
    let owner = selected(&arena, source);
    let operand = observe(&owner, 0);
    assert_original(&operand, &owner, 0);
    let element = owner.children().next().unwrap().into_element().unwrap();
    assert!(
        operand
            .admitted_for(&owner, element.attributes().nth(1).unwrap())
            .is_none()
    );
    let nested = element.children().next().unwrap().into_element().unwrap();
    assert!(
        operand
            .admitted_for(&owner, nested.attributes().next().unwrap())
            .is_none()
    );
    let other = selected(&arena, source);
    let copy = vize_l0::String::from(source);
    let copied = selected(&arena, copy.as_str());
    assert!(!core::ptr::eq(copy.as_str(), source));
    for foreign in [&other, &copied] {
        let element = foreign.children().next().unwrap().into_element().unwrap();
        let attribute = element.attributes().next().unwrap();
        assert!(
            operand
                .admitted_for(foreign, attribute.reborrow())
                .is_none()
        );
        assert!(operand.admitted_for(&owner, attribute.reborrow()).is_none());
        let before = arena.allocated_bytes();
        assert_eq!(
            owner.observe_attribute_head(attribute).err(),
            Some(NativeAttributeOperandError::ForeignComponent)
        );
        assert_eq!(arena.allocated_bytes(), before);
        assert_eq!(foreign.component().block().root_source(), source);
    }
    assert_eq!(operand.name_span(), Span::new(13, 16));
    assert_eq!(operand.argument_span(), Span::new(14, 16));
    assert_eq!(operand.value_span(), Span::new(18, 22));
    assert_eq!(operand.raw_value(), "same");
    assert_eq!(operand.syntax().source().text(), "same");
    assert_original(&operand, &owner, 0);
}

#[test]
fn parked_move_and_selection_scope_end_preserve_original_stock_root_map_and_raw_transfer() {
    let arena = Allocator::default();
    let source = "<template><p id='kept' :id='/*原*/ a &amp;&amp; b'/></template>";
    let (syntax, root) = {
        let owner = selected(&arena, source);
        let operand = observe(&owner, 1);
        let root = operand.syntax().expression().unwrap();
        let mut parked = alloc::vec![operand];
        parked.reserve(32);
        let moved = core::hint::black_box(owner);
        assert_original(&parked[0], &moved, 1);
        assert!(core::ptr::eq(
            parked[0].syntax().expression().unwrap(),
            root
        ));
        (parked.pop().unwrap().into_syntax(), root)
    };
    assert!(core::ptr::eq(syntax.expression().unwrap(), root));
    assert_eq!(syntax.source().span(), Span::new(28, 50));
    assert_eq!(syntax.source().text(), "/*原*/ a && b");
    assert!(core::ptr::eq(syntax.source().authored_root(), source));
    assert_eq!(syntax.hole(), None);
    assert_eq!(syntax.diagnostics().count(), 0);
    let comments: Vec<_> = syntax
        .comments()
        .map(|comment| {
            (
                comment.kind(),
                comment.decoded_span().unwrap(),
                comment.authored_span().unwrap(),
                comment.text().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        comments,
        [(
            CommentKind::SingleLineBlock,
            Span::new(0, 7),
            Span::new(28, 35),
            "/*原*/"
        )]
    );
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
                Span::new(0, 10),
                Span::new(28, 38),
                DecodeSegmentKind::Identity
            ),
            (
                Span::new(10, 11),
                Span::new(38, 43),
                DecodeSegmentKind::Entity
            ),
            (
                Span::new(11, 12),
                Span::new(43, 48),
                DecodeSegmentKind::Entity
            ),
            (
                Span::new(12, 14),
                Span::new(48, 50),
                DecodeSegmentKind::Identity
            ),
        ]
    );
    assert!(syntax.admitted_expression().is_some());
    assert!(core::mem::needs_drop::<
        NativeAttributeBindingExpression<'static>,
    >());
}

#[test]
fn pending_original_hole_survives_callback_unwind_and_forget_grants_no_new_receipt() {
    extern crate std;
    let arena = Allocator::default();
    let source = "<template><p :id='/*kept*/ ready +'/></template>";
    let mut pending = None;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let owner = selected(&arena, source);
        pending = Some(observe(&owner, 0));
        panic!("fixture callback unwind");
    }));
    assert!(result.is_err());
    let retained = pending.take().unwrap();
    assert_eq!(retained.name_span(), Span::new(13, 16));
    assert_eq!(retained.argument_span(), Span::new(14, 16));
    assert_eq!(retained.value_span(), Span::new(18, 34));
    assert_eq!(retained.raw_value(), "/*kept*/ ready +");
    assert_eq!(retained.syntax().source().text(), "/*kept*/ ready +");
    assert_eq!(retained.syntax().hole(), Some(EmbedHole::Syntax));
    assert!(retained.syntax().expression().is_none());
    assert!(retained.syntax().admitted_expression().is_none());
    assert_eq!(retained.syntax().comments().count(), 1);
    let diagnostics: Vec<_> = retained
        .syntax()
        .diagnostics()
        .map(|row| row.message())
        .collect();
    assert_eq!(diagnostics, ["Unexpected token"]);
    let syntax = retained.into_syntax();
    assert!(core::ptr::eq(syntax.source().authored_root(), source));
    drop(syntax);

    let intact_source = "<template><p :id='ready'/></template>";
    let intact = selected(&arena, intact_source);
    let forgotten = observe(&intact, 0);
    assert_original(&forgotten, &intact, 0);
    core::mem::forget(forgotten);
    assert_eq!(intact.component().block().source(), "<p :id='ready'/>");
    assert_eq!(intact.component().block().root_source(), intact_source);
    crate::check_fidelity(&intact.component().carrier().tree).unwrap();
    // This tests source survival at the deliberate forget boundary, without
    // fabricating an admission or destructor-count claim after that boundary.
}
