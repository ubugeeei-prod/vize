//! `.trim` is new original-head support, not credit for its former refusal.

use super::*;

#[test]
fn trim_keeps_the_actual_modifier_token_and_original_value_without_trimming_source() {
    for source in [
        "<template><input v-model.trim='draft'/></template>",
        "<!--雪🌸-->\r\n<script setup lang='ts'>let café:string=' x '</script>\r\n<template><input v-model.trim=café /></template>\r\n",
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
        assert_eq!(operand.name_span().slice(source), "v-model.trim");
        let trim = operand.trim_span().unwrap();
        assert_eq!(trim, source_span(source, "trim"));
        assert_eq!(trim.slice(source), "trim");
        assert!(core::ptr::eq(
            trim.slice(source),
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
        assert_eq!(operand.syntax().source().text(), operand.raw_value());
        assert_eq!(operand.value_span().slice(source), operand.raw_value());
    }
}

#[test]
fn trim_selects_before_value_preparation_and_preserves_malformed_original_syntax() {
    let arena = Allocator::default();
    // This exact modifier was part of the old inline unsupported-head set.
    // It now selects normally; the malformed value still has its real hole.
    let source = "<template><input v-model.trim='broken &amp;'/></template>";
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let head = owner
        .observe_attribute_head(element.attributes().next().unwrap())
        .unwrap();
    let before = arena.allocated_bytes();
    let model = head.model().unwrap().unwrap();
    assert_eq!(arena.allocated_bytes(), before);
    let operand = model.observe_expression().unwrap();
    assert_eq!(operand.trim_span(), Some(source_span(source, "trim")));
    assert_eq!(operand.raw_value(), "broken &amp;");
    assert!(operand.syntax().hole().is_some());
    assert!(operand.syntax().admitted_expression().is_none());
    assert!(operand.syntax().source().decode_map().is_some());
}

#[test]
fn bare_model_keeps_no_modifier_even_when_the_value_or_adjacent_attribute_contains_trim() {
    let arena = Allocator::default();
    let source = "<template><input title='trim' v-model='trim'/></template>";
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let operand = owner
        .observe_attribute_head(element.attributes().nth(1).unwrap())
        .unwrap()
        .model()
        .unwrap()
        .unwrap()
        .observe_expression()
        .unwrap();
    assert_eq!(operand.trim_span(), None);
    assert_eq!(operand.name_span().slice(source), "v-model");
    assert_eq!(operand.raw_value(), "trim");
}
