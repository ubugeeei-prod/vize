//! The former lazy head refusal now retains one actual modifier and its value.

use super::*;

#[test]
fn lazy_retains_the_original_token_header_and_value_with_foreign_owner_refusal() {
    for source in [
        "<template><input v-model.lazy='draft'/></template>",
        "<!--雪🌸-->\r\n<script setup lang='ts'>let café:string=' x '</script>\r\n<template><input v-model.lazy=café /></template>\r\n",
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
        let span = operand.lazy_span().unwrap();
        assert_eq!(operand.name_span().slice(source), "v-model.lazy");
        assert_eq!(operand.trim_span(), None);
        assert_eq!(span, source_span(source, "lazy"));
        assert_eq!(span.slice(source), "lazy");
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
fn lazy_selection_does_not_turn_the_old_malformed_value_into_admitted_syntax() {
    let arena = Allocator::default();
    let source = "<template><input v-model.lazy='broken &amp;'/></template>";
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let head = owner
        .observe_attribute_head(element.attributes().next().unwrap())
        .unwrap();
    let before = arena.allocated_bytes();
    let model = head.model().unwrap().unwrap();
    assert_eq!(arena.allocated_bytes(), before);
    let operand = model.observe_expression().unwrap();
    assert_eq!(operand.lazy_span(), Some(source_span(source, "lazy")));
    assert_eq!(operand.trim_span(), None);
    assert_eq!(operand.raw_value(), "broken &amp;");
    assert!(operand.syntax().hole().is_some());
    assert!(operand.syntax().admitted_expression().is_none());
    assert!(operand.syntax().source().decode_map().is_some());
}

#[test]
fn old_bare_and_trim_getters_cannot_borrow_a_lazy_value_or_adjacent_attribute() {
    for (head, expected_trim) in [("v-model", false), ("v-model.trim", true)] {
        let arena = Allocator::default();
        let source = alloc::format!("<template><input title='lazy' {head}='lazy'/></template>");
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
        assert_eq!(operand.lazy_span(), None);
        assert_eq!(operand.trim_span().is_some(), expected_trim);
        assert_eq!(operand.raw_value(), "lazy");
    }
}
