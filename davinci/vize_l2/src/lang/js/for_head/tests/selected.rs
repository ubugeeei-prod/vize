use super::{Allocator, BindingPattern, GetSpan, Lang, NativeForInput, NativeForRefusal, Span};
use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::container::{Vue, vue::DescriptorOptions};
use vize_l1::markup::NativeTemplateComponent;

fn selected<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: vize_l1::SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap()
}

#[test]
fn one_original_header_visit_parks_both_observations_and_joins_the_same_token() {
    let arena = Allocator::default();
    let source = "頭<script lang='ts'></script><template><b v-for='(α, key) in items'/></template>";
    let selected = selected(&arena, source);
    let element = selected.children().next().unwrap().into_element().unwrap();
    let mut visits = 0;
    for attribute in element.attributes() {
        visits += 1;
        let operand = selected
            .observe_attribute_for_head(attribute.reborrow())
            .unwrap();
        let aliases = operand
            .syntax()
            .aliases()
            .unwrap()
            .unwrap()
            .parameters()
            .unwrap()
            .as_ptr();
        let collection = operand
            .syntax()
            .collection()
            .unwrap()
            .unwrap()
            .expression()
            .unwrap() as *const _;
        let input = NativeForInput::new(operand).unwrap();
        let mut pending = alloc::vec::Vec::new();
        pending.push(input);
        pending.reserve(32);
        let input = pending.pop().unwrap();
        assert_eq!(input.aliases().as_ptr(), aliases);
        assert_eq!(input.collection() as *const _, collection);
        assert_eq!(input.operand().syntax().grammar().lang, Lang::Ts);
        let joined = input.admitted_for(&selected, attribute).unwrap();
        let admitted = joined.for_head().unwrap();
        assert_eq!(admitted.aliases().parameters().items.as_ptr(), aliases);
        assert_eq!(admitted.collection().expression() as *const _, collection);
        assert!(core::ptr::eq(joined.selected(), &selected));
        let BindingPattern::BindingIdentifier(alias) = &input.aliases()[0].pattern else {
            panic!("original alias")
        };
        let authored = input.alias_authored_span(alias.span).unwrap();
        assert_eq!(authored.slice(source), "α");
        let authored = input
            .collection_authored_span(input.collection().span())
            .unwrap();
        assert_eq!(authored.slice(source), "items");
        let operand = input.into_operand();
        assert_eq!(operand.raw_value(), "(α, key) in items");
        assert_eq!(
            operand
                .syntax()
                .aliases()
                .unwrap()
                .unwrap()
                .parameters()
                .unwrap()
                .as_ptr(),
            aliases
        );
    }
    assert_eq!(visits, 1);
}

#[test]
fn sibling_equal_parse_and_foreign_equal_file_cannot_replace_the_selected_header() {
    let arena = Allocator::default();
    let source = "<template><b v-for='item in items' v-for='item in items'/></template>";
    let selected = selected(&arena, source);
    let foreign = self::selected(&arena, source);
    let element = selected.children().next().unwrap().into_element().unwrap();
    let operand = selected
        .observe_attribute_for_head(element.attributes().next().unwrap())
        .unwrap();
    let input = NativeForInput::new(operand).unwrap();
    assert!(
        input
            .admitted_for(&selected, element.attributes().next().unwrap())
            .is_some()
    );
    assert!(
        input
            .admitted_for(&selected, element.attributes().nth(1).unwrap())
            .is_none()
    );
    let other = foreign.children().next().unwrap().into_element().unwrap();
    assert!(
        input
            .admitted_for(&foreign, other.attributes().next().unwrap())
            .is_none()
    );
}

#[test]
fn selected_refusal_retains_the_complete_original_attribute_and_syntax() {
    let arena = Allocator::default();
    let source = "<template><b v-for='item in /*kept*/items'/></template>";
    let selected = selected(&arena, source);
    let element = selected.children().next().unwrap().into_element().unwrap();
    let attribute = element.attributes().next().unwrap();
    let operand = selected
        .observe_attribute_for_head(attribute.reborrow())
        .unwrap();
    let span = operand.value_span();
    let comment = operand
        .syntax()
        .comments()
        .next()
        .unwrap()
        .1
        .text()
        .unwrap()
        .as_ptr();
    let rejected = NativeForInput::new(operand).unwrap_err();
    assert_eq!(rejected.kind, NativeForRefusal::Comment);
    assert_eq!(rejected.span, span);
    assert_eq!(
        rejected
            .operand()
            .syntax()
            .comments()
            .next()
            .unwrap()
            .1
            .text()
            .unwrap()
            .as_ptr(),
        comment
    );
    let operand = rejected.into_operand();
    assert_eq!(operand.raw_value(), "item in /*kept*/items");
    assert!(operand.admitted_for(&selected, attribute).is_none());
    assert_eq!(span.slice(source), operand.raw_value());
    assert_ne!(span, Span::new(0, source.len() as u32));
}
