//! Single original number is a new head; malformed syntax keeps its old hole.

use super::*;

#[test]
fn number_retains_original_modifier_and_refuses_equal_byte_foreign_owners() {
    for source in [
        "<template><input v-model.number='draft'/></template>",
        "<!--雪🌸-->\r\n<script setup lang='ts'>let café:string='2'</script>\r\n<template><input v-model.number=café /></template>\r\n",
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let foreign = selected(&arena, source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let attribute = element.attributes().next().unwrap();
        let before = arena.allocated_bytes();
        let head = owner.observe_attribute_head(attribute.reborrow()).unwrap();
        let model = head.model().unwrap().unwrap();
        assert!(core::ptr::eq(model.head(), &head));
        assert_eq!(arena.allocated_bytes(), before);
        let operand = model.observe_expression().unwrap();
        let span = operand.number_span().unwrap();
        assert_eq!(operand.name_span().slice(source), "v-model.number");
        assert_eq!(operand.trim_span(), None);
        assert_eq!(operand.lazy_span(), None);
        assert_eq!(span, source_span(source, "number"));
        assert_eq!(span.slice(source), "number");
        assert!(core::ptr::eq(
            span.slice(source),
            operand
                .name_span()
                .slice(source)
                .strip_prefix("v-model.")
                .unwrap()
        ));
        assert!(operand.admitted_for(&owner, attribute).is_some());
        let other = foreign.children().next().unwrap().into_element().unwrap();
        assert!(
            operand
                .admitted_for(&foreign, other.attributes().next().unwrap())
                .is_none()
        );
        assert!(operand.syntax().hole().is_none());
        assert!(operand.syntax().diagnostics().next().is_none());
        assert!(core::ptr::eq(
            operand.syntax().source().authored_root(),
            source
        ));
        assert_eq!(operand.value_span().slice(source), operand.raw_value());
    }
}

#[test]
fn former_number_head_refusal_preserves_the_exact_malformed_value_as_a_hole() {
    // This exact old inline input gains head selection, never valid syntax.
    let source = "<template><input v-model.number='broken &amp;'/></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let head = owner
        .observe_attribute_head(element.attributes().next().unwrap())
        .unwrap();
    let before = arena.allocated_bytes();
    let model = head.model().unwrap().unwrap();
    assert_eq!(arena.allocated_bytes(), before);
    let operand = model.observe_expression().unwrap();
    assert_eq!(operand.number_span(), Some(source_span(source, "number")));
    assert_eq!(operand.trim_span(), None);
    assert_eq!(operand.lazy_span(), None);
    assert_eq!(operand.raw_value(), "broken &amp;");
    assert!(operand.syntax().hole().is_some());
    assert!(operand.syntax().admitted_expression().is_none());
    assert!(operand.syntax().source().decode_map().is_some());
}

#[test]
fn number_getter_cannot_borrow_a_value_or_adjacent_attribute_from_old_heads() {
    for (head, trim, lazy) in [
        ("v-model", false, false),
        ("v-model.trim", true, false),
        ("v-model.lazy", false, true),
    ] {
        let arena = Allocator::default();
        let source = alloc::format!("<template><input title='number' {head}='number'/></template>");
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let operand = owner
            .observe_attribute_head(element.attributes().nth(1).unwrap())
            .unwrap()
            .model()
            .unwrap()
            .unwrap()
            .observe_expression()
            .unwrap();
        assert_eq!(operand.number_span(), None);
        assert_eq!(operand.trim_span().is_some(), trim);
        assert_eq!(operand.lazy_span().is_some(), lazy);
        assert_eq!(operand.raw_value(), "number");
    }
}
