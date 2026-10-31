use crate::container::Vue;
use crate::container::vue::DescriptorOptions;
use crate::markup::{NativeAttributeOperandError as Error, NativeTemplateComponent};
use vize_l0::Allocator;
use vize_l0::config::{VueDialect, VueVersion};

fn selected<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: crate::markup::SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap()
}

#[test]
fn valueless_else_retains_original_unicode_extent_without_expression_or_allocation() {
    let arena = Allocator::default();
    let source = "<script setup lang='ts'>let 可視:boolean=true</script>\r\n<template><p v-if='可視'>雪🌸</p><b v-else>否</b></template>";
    let selected = selected(&arena, source);
    let element = selected.children().nth(1).unwrap().into_element().unwrap();
    let attribute = element.attributes().next().unwrap();
    let before = arena.allocated_bytes();
    let head = selected
        .observe_attribute_head(attribute.reborrow())
        .unwrap();
    assert_eq!(head.condition_kind(), None);
    let owner = head.observe_else().unwrap();
    assert_eq!(owner.name_span().slice(source), "v-else");
    assert_eq!(owner.element_span().slice(source), "<b v-else>否</b>");
    assert!(core::ptr::eq(owner.source_block().root_source(), source));
    let view = owner.admitted_for(&selected, attribute.reborrow()).unwrap();
    assert!(core::ptr::eq(view.owner(), &owner));
    assert!(core::ptr::eq(view.selected(), &selected));
    assert!(core::ptr::eq(view.attribute().element(), element.surface()));
    assert_eq!(arena.allocated_bytes(), before);
    let failure = head.observe_expression().err().unwrap();
    assert_eq!(failure.kind(), Error::UnsupportedDirective);
    assert!(failure.syntax().is_none());
    assert_eq!(arena.allocated_bytes(), before);
}

#[test]
fn identical_sibling_and_fresh_parse_cannot_borrow_original_else_authority() {
    let arena = Allocator::default();
    let source = "<template><p v-else/><p v-else/></template>";
    let first = selected(&arena, source);
    let reparsed = selected(&arena, source);
    let mut children = first.children();
    let one = children.next().unwrap().into_element().unwrap();
    let two = children.next().unwrap().into_element().unwrap();
    let one_attr = one.attributes().next().unwrap();
    let owner = first
        .observe_attribute_head(one_attr.reborrow())
        .unwrap()
        .observe_else()
        .unwrap();
    assert!(owner.admitted_for(&first, one_attr.reborrow()).is_some());
    assert!(
        owner
            .admitted_for(&first, two.attributes().next().unwrap())
            .is_none()
    );
    let foreign = reparsed.children().next().unwrap().into_element().unwrap();
    assert!(
        owner
            .admitted_for(&reparsed, foreign.attributes().next().unwrap())
            .is_none()
    );
    assert!(owner.admitted_for(&reparsed, one_attr.reborrow()).is_none());
    assert_eq!(owner.element_span().slice(source), "<p v-else/>");
}

#[test]
fn values_arguments_modifiers_and_other_directives_never_become_an_else_owner() {
    for source in [
        "<template><p v-else='ready'/></template>",
        "<template><p v-else=''/></template>",
        "<template><p v-else:arg/></template>",
        "<template><p v-else.foo/></template>",
        "<template><p v-if='ready'/></template>",
        "<template><p id/></template>",
    ] {
        let arena = Allocator::default();
        let selected = selected(&arena, source);
        let element = selected.children().next().unwrap().into_element().unwrap();
        let attribute = element.attributes().next().unwrap();
        let head = selected.observe_attribute_head(attribute).unwrap();
        let before = arena.allocated_bytes();
        assert_eq!(head.observe_else().err(), Some(Error::UnsupportedDirective));
        assert_eq!(arena.allocated_bytes(), before);
        assert_eq!(selected.component().block().root_source(), source);
    }
}

#[test]
fn recovery_and_verbatim_precede_even_the_valueless_else_head() {
    for (source, ordinal, expected) in [
        (
            "<template><p v-else></template>",
            0,
            Error::RecoveredComponent,
        ),
        ("<template><p v-pre v-else/></template>", 1, Error::Verbatim),
    ] {
        let arena = Allocator::default();
        let selected = selected(&arena, source);
        let element = selected.children().next().unwrap().into_element().unwrap();
        let attribute = element.attributes().nth(ordinal).unwrap();
        assert_eq!(
            selected.observe_attribute_head(attribute).err(),
            Some(expected)
        );
        assert_eq!(selected.component().block().root_source(), source);
    }
}
