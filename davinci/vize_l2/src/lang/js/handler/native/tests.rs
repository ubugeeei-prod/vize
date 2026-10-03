use oxc_ast::ast::{Expression, Statement};
use oxc_span::GetSpan;
use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::{Vue, vue::DescriptorOptions};
use vize_l1::markup::NativeTemplateComponent;

use super::{HandlerInputErrorKind, NativeHandlerInput};

fn selected<'a>(allocator: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(
        allocator,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: vize_l1::SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(allocator, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap()
}

#[test]
fn a_single_original_header_iteration_preserves_the_event_and_whole_body() {
    let allocator = Allocator::default();
    let source = "前<template><button @click='/*kept*/ α++; return α'/></template>";
    let selected = selected(&allocator, source);
    let element = selected.children().next().unwrap().into_element().unwrap();
    let mut visits = 0;
    for attribute in element.attributes() {
        visits += 1;
        let operand = selected
            .observe_attribute_handler(attribute.reborrow())
            .unwrap();
        let body = operand.syntax().body().unwrap();
        let comments = operand
            .syntax()
            .comments()
            .next()
            .unwrap()
            .text()
            .unwrap()
            .as_ptr();
        let input = NativeHandlerInput::new(operand).unwrap();
        let mut parked = alloc::vec::Vec::new();
        parked.push(input);
        parked.reserve(32);
        let input = parked.pop().unwrap();
        assert!(core::ptr::eq(input.body(), body));
        assert_eq!(
            input
                .operand()
                .syntax()
                .comments()
                .next()
                .unwrap()
                .text()
                .unwrap()
                .as_ptr(),
            comments
        );
        assert_eq!(input.operand().raw_value(), "/*kept*/ α++; return α");
        let joined = input.admitted_for(&selected, attribute).unwrap();
        assert!(core::ptr::eq(joined.handler_body().unwrap().body(), body));
        assert!(core::ptr::eq(joined.selected(), &selected));
        assert_eq!(input.body().statements.len(), 2);
        let Statement::ReturnStatement(returned) = &input.body().statements[1] else {
            panic!("original return")
        };
        let Expression::Identifier(identifier) = returned.argument.as_ref().unwrap() else {
            panic!("original reference")
        };
        let authored = input.authored_span(identifier.span()).unwrap();
        assert_eq!(authored, Span::new(51, 53));
        assert_eq!(authored.slice(source), "α");
        let operand = input.into_operand();
        assert!(core::ptr::eq(operand.syntax().body().unwrap(), body));
    }
    assert_eq!(visits, 1);
}

#[test]
fn equal_bytes_and_sibling_headers_cannot_substitute_for_the_original_event() {
    let allocator = Allocator::default();
    let source = "<template><button @click='value' @click='value'/></template>";
    let first = selected(&allocator, source);
    let independent = selected(&allocator, source);
    let element = first.children().next().unwrap().into_element().unwrap();
    let operand = first
        .observe_attribute_handler(element.attributes().next().unwrap())
        .unwrap();
    let input = NativeHandlerInput::new(operand).unwrap();
    assert!(
        input
            .admitted_for(&first, element.attributes().next().unwrap())
            .is_some()
    );
    assert!(
        input
            .admitted_for(&first, element.attributes().nth(1).unwrap())
            .is_none()
    );
    let foreign = independent
        .children()
        .next()
        .unwrap()
        .into_element()
        .unwrap();
    assert!(
        input
            .admitted_for(&independent, foreign.attributes().next().unwrap())
            .is_none()
    );
}

#[test]
fn local_holes_return_the_original_event_association_with_all_diagnostics() {
    let allocator = Allocator::default();
    let source = "<template><button @click='return /x/uv; //kept'/></template>";
    let selected = selected(&allocator, source);
    let element = selected.children().next().unwrap().into_element().unwrap();
    let attribute = element.attributes().next().unwrap();
    let operand = selected
        .observe_attribute_handler(attribute.reborrow())
        .unwrap();
    let diagnostic = operand
        .syntax()
        .diagnostics()
        .next()
        .unwrap()
        .message()
        .as_ptr();
    let count = operand.syntax().diagnostics().count();
    let value_span = operand.value_span();
    let rejected = NativeHandlerInput::new(operand).unwrap_err();
    assert_eq!(rejected.kind, HandlerInputErrorKind::IncompleteSyntax);
    assert_eq!(rejected.span, value_span);
    assert_eq!(rejected.operand().syntax().diagnostics().count(), count);
    assert_eq!(
        rejected
            .operand()
            .syntax()
            .diagnostics()
            .next()
            .unwrap()
            .message()
            .as_ptr(),
        diagnostic
    );
    assert_eq!(
        rejected
            .operand()
            .syntax()
            .comments()
            .next()
            .unwrap()
            .text()
            .unwrap(),
        "//kept"
    );
    let operand = rejected.into_operand();
    assert_eq!(operand.argument(), "click");
    assert_eq!(operand.raw_value(), "return /x/uv; //kept");
    assert!(operand.admitted_for(&selected, attribute).is_none());
}
