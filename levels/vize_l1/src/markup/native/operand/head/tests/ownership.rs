use super::selected;
use crate::markup::{NativeAttributeOperandError, NativeConditionKind};
use vize_l0::Allocator;

#[test]
fn short_head_retains_the_actual_slot_and_can_end_before_owner_move() {
    let arena = Allocator::default();
    let source = "<template><p id='kept' v-if='/*原*/ a &amp;&amp; b'/></template>";
    let selected = selected(&arena, source);
    let observed = {
        let element = selected.children().next().unwrap().into_element().unwrap();
        let mut attributes = element.attributes();
        let plain = attributes.next().unwrap();
        let before = arena.allocated_bytes();
        {
            let head = selected.observe_attribute_head(plain).unwrap();
            assert_eq!(arena.allocated_bytes(), before);
            assert!(core::ptr::eq(head.selected(), &selected));
            assert!(core::ptr::eq(head.attribute().element(), element.surface()));
            assert_eq!(head.attribute().ordinal(), 0);
            assert_eq!(head.name_block().source(), "id");
            assert_eq!(head.directive(), None);
            assert_eq!(head.condition_kind(), None);
        }
        let current = attributes.next().unwrap();
        let head = selected.observe_attribute_head(current.reborrow()).unwrap();
        assert_eq!(head.attribute().ordinal(), 1);
        assert_eq!(head.condition_kind(), Some(NativeConditionKind::If));
        let observed = head.observe_expression().unwrap();
        assert!(observed.admitted_for(&selected, current).is_some());
        assert!(attributes.next().is_none());
        observed
    };
    let original = observed.syntax().expression().unwrap();
    let mut parked = alloc::vec![observed];
    parked.reserve(32);
    let moved = core::hint::black_box(selected);
    let element = moved.children().next().unwrap().into_element().unwrap();
    let admitted = parked[0]
        .admitted_for(&moved, element.attributes().nth(1).unwrap())
        .unwrap();
    assert!(core::ptr::eq(
        admitted.expression().unwrap().expression(),
        original
    ));
    assert_eq!(parked[0].raw_value(), "/*原*/ a &amp;&amp; b");
    assert_eq!(parked[0].syntax().source().text(), "/*原*/ a && b");
    assert_eq!(moved.component().block().root_source(), source);
}

#[test]
fn independent_same_buffer_and_equal_byte_copy_headers_cannot_select_a_head() {
    let arena = Allocator::default();
    let source = "<template><p v-if='a' v-else-if='a'/><p v-if='a'/></template>";
    let selected = selected(&arena, source);
    let original_element = selected.children().next().unwrap().into_element().unwrap();
    let original = selected
        .observe_attribute_head(original_element.attributes().next().unwrap())
        .unwrap()
        .observe_expression()
        .unwrap();
    let other = super::selected(&arena, source);
    let copy = vize_l0::String::from(source);
    let copied = super::selected(&arena, copy.as_str());
    for foreign in [&other, &copied] {
        let element = foreign.children().next().unwrap().into_element().unwrap();
        let failure = selected
            .observe_attribute_head(element.attributes().next().unwrap())
            .err()
            .unwrap();
        assert_eq!(failure, NativeAttributeOperandError::ForeignComponent);
        assert!(
            original
                .admitted_for(foreign, element.attributes().next().unwrap())
                .is_none()
        );
        assert_eq!(foreign.component().block().root_source(), source);
    }
    assert!(
        original
            .admitted_for(&selected, original_element.attributes().nth(1).unwrap())
            .is_none()
    );
    let sibling = selected.children().nth(1).unwrap().into_element().unwrap();
    assert!(
        original
            .admitted_for(&selected, sibling.attributes().next().unwrap())
            .is_none()
    );
}

#[test]
fn normally_owned_observation_survives_short_head_and_selected_owner_drop() {
    let arena = Allocator::default();
    let source = "<template><p v-if='/*原*/ a &amp;&amp; b'/></template>";
    let observed = {
        let selected = selected(&arena, source);
        let element = selected.children().next().unwrap().into_element().unwrap();
        let head = selected
            .observe_attribute_head(element.attributes().next().unwrap())
            .unwrap();
        let observed = head.observe_expression().unwrap();
        assert!(
            observed
                .admitted_for(&selected, head.attribute().reborrow())
                .is_some()
        );
        observed
    };
    assert!(core::mem::needs_drop::<
        crate::markup::NativeAttributeExpression<'static>,
    >());
    let root = observed.syntax().expression().unwrap();
    let syntax = observed.into_syntax();
    assert!(core::ptr::eq(syntax.expression().unwrap(), root));
    assert_eq!(syntax.source().text(), "/*原*/ a && b");
    assert!(core::ptr::eq(syntax.source().authored_root(), source));
    assert_eq!(syntax.comments().count(), 1);
    assert_eq!(syntax.diagnostics().count(), 0);
    assert!(syntax.admitted_expression().is_some());
}
