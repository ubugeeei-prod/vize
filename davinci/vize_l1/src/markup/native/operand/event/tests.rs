use super::{NativeAttributeHandler, NativeAttributeOperandError};
use crate::container::Vue;
use crate::container::vue::DescriptorOptions;
use crate::embed::{Lang, Shape, syntax::EmbedHole};
use crate::markup::{NativeTemplateComponent, NativeTemplateGrammar};
use oxc_ast::ast::{Expression, Statement};
use oxc_span::GetSpan;
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};

fn selected<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: crate::SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap()
}

fn observe<'a>(owner: &NativeTemplateComponent<'a>, ordinal: usize) -> NativeAttributeHandler<'a> {
    let element = owner.children().next().unwrap().into_element().unwrap();
    owner
        .observe_attribute_handler(element.attributes().nth(ordinal).unwrap())
        .unwrap()
}

#[test]
fn original_static_heads_values_and_real_multistatement_body_join_the_same_event() {
    let arena = Allocator::default();
    let source = "<!--前--><template><button @更新='/*keep*/ count++; return count' v-on:change='done'/></template>";
    let owner = selected(&arena, source);
    for (ordinal, name, raw_name, raw_value) in [
        (0, "更新", "@更新", "/*keep*/ count++; return count"),
        (1, "change", "v-on:change", "done"),
    ] {
        let operand = observe(&owner, ordinal);
        assert_eq!(operand.argument(), name);
        assert_eq!(operand.argument_span().slice(source), name);
        assert_eq!(operand.name_span().slice(source), raw_name);
        assert_eq!(operand.raw_value(), raw_value);
        assert_eq!(operand.value_span().slice(source), raw_value);
        assert_eq!(operand.syntax().grammar().shape, Shape::HandlerBody);
        assert_eq!(operand.syntax().grammar().lang, Lang::Js);
        assert!(core::ptr::eq(
            operand.syntax().source().authored_root(),
            source
        ));
        let original = operand.syntax().body().unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let admitted = operand
            .admitted_for(&owner, element.attributes().nth(ordinal).unwrap())
            .unwrap();
        assert!(core::ptr::eq(admitted.selected(), &owner));
        assert!(core::ptr::eq(
            admitted.attribute().element(),
            element.surface()
        ));
        assert!(core::ptr::eq(
            admitted.handler_body().unwrap().body(),
            original
        ));
        if ordinal == 0 {
            assert_eq!(original.statements.len(), 2);
            assert!(matches!(
                original.statements[0],
                Statement::ExpressionStatement(_)
            ));
            assert!(matches!(
                original.statements[1],
                Statement::ReturnStatement(_)
            ));
            assert_eq!(
                operand.syntax().comments().next().unwrap().text().unwrap(),
                "/*keep*/"
            );
        }
        assert!(operand.syntax().authored_span(original.span).is_err());
    }
}

#[test]
fn once_decoded_handler_keeps_original_unicode_crlf_and_entity_coordinates() {
    let arena = Allocator::default();
    let source = "前\r\n<template><button @click='/*kept*/\r\nreturn α &amp;&amp; β'/></template>";
    let owner = selected(&arena, source);
    let operand = observe(&owner, 0);
    assert_eq!(
        operand.syntax().source().text(),
        "/*kept*/\r\nreturn α && β"
    );
    assert!(operand.syntax().source().decode_map().is_some());
    let body = operand.syntax().body().unwrap();
    let Statement::ReturnStatement(returned) = &body.statements[0] else {
        panic!("actual return")
    };
    let Expression::LogicalExpression(logical) = returned.argument.as_ref().unwrap() else {
        panic!("actual logical")
    };
    assert_eq!(
        operand
            .syntax()
            .authored_span(logical.left.span())
            .unwrap()
            .slice(source),
        "α"
    );
    assert_eq!(
        operand
            .syntax()
            .authored_span(logical.right.span())
            .unwrap()
            .slice(source),
        "β"
    );
    let comment = operand.syntax().comments().next().unwrap();
    assert_eq!(comment.authored_span().unwrap().slice(source), "/*kept*/");
    assert_eq!(comment.text().unwrap(), "/*kept*/");
    assert_eq!(operand.syntax().diagnostics().count(), 0);
}

#[test]
fn selected_setup_only_ts_is_intrinsic_and_js_syntax_failure_keeps_complete_owner() {
    let arena = Allocator::default();
    let value = "const value: number = 1; return value;";
    let ts = "<template><button @click='const value: number = 1; return value;'/></template><script setup lang=ts>const n=1</script>";
    let owner = selected(&arena, ts);
    assert_eq!(owner.grammar(), NativeTemplateGrammar::TypeScriptModule);
    let operand = observe(&owner, 0);
    assert_eq!(operand.raw_value(), value);
    assert_eq!(operand.syntax().grammar().lang, Lang::Ts);
    assert_eq!(operand.syntax().body().unwrap().statements.len(), 2);
    let js = "<template><button @click='const value: number = 1; return value;'/></template>";
    let owner = selected(&arena, js);
    let operand = observe(&owner, 0);
    assert_eq!(operand.syntax().grammar().lang, Lang::Js);
    assert_eq!(operand.syntax().hole(), Some(EmbedHole::Syntax));
    assert!(operand.syntax().diagnostics().count() > 0);
    assert_eq!(operand.raw_value(), value);
    let element = owner.children().next().unwrap().into_element().unwrap();
    assert!(
        operand
            .admitted_for(&owner, element.attributes().next().unwrap())
            .is_none()
    );
}

#[test]
fn moved_and_parked_owners_rejoin_the_original_event_without_addressing_the_wrapper() {
    let arena = Allocator::default();
    let source = "<template><button @click='return count'/></template>";
    let owner = selected(&arena, source);
    let operand = observe(&owner, 0);
    let original = operand.syntax().body().unwrap();
    let mut parked = alloc::vec::Vec::new();
    parked.push(operand);
    parked.reserve(32);
    let owner = core::hint::black_box(owner);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let view = parked[0]
        .admitted_for(&owner, element.attributes().next().unwrap())
        .unwrap();
    assert!(core::ptr::eq(view.handler_body().unwrap().body(), original));
    assert!(core::ptr::eq(view.operand(), &parked[0]));
}

#[test]
fn siblings_nested_attributes_and_independent_equal_byte_parse_cannot_admit() {
    let arena = Allocator::default();
    let source =
        "<template><button @click='done' @change='done'><i @click='done'/></button></template>";
    let owner = selected(&arena, source);
    let operand = observe(&owner, 0);
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
    let copied = selected(&arena, source);
    let foreign = copied.children().next().unwrap().into_element().unwrap();
    assert!(
        operand
            .admitted_for(&copied, foreign.attributes().next().unwrap())
            .is_none()
    );
    let error = owner
        .observe_attribute_handler(foreign.attributes().next().unwrap())
        .err()
        .unwrap();
    assert_eq!(error.kind(), NativeAttributeOperandError::ForeignComponent);
}

#[test]
fn unsupported_heads_and_incomplete_values_refuse_before_decoding_or_parsing() {
    let arena = Allocator::default();
    for source in [
        "<template><button @[event]='broken &amp;'/></template>",
        "<template><button @click.stop='broken &amp;'/></template>",
        "<template><button v-on='broken &amp;'/></template>",
        "<template><button v-bind:click='broken &amp;'/></template>",
        "<template><button v-on:='broken &amp;'/></template>",
        "<template><button @clicktail.stop='broken &amp;'/></template>",
        "<template><button @click/></template>",
    ] {
        let owner = selected(&arena, source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let before = arena.allocated_bytes();
        let failure = owner
            .observe_attribute_handler(element.attributes().next().unwrap())
            .err()
            .unwrap();
        assert!(
            matches!(
                failure.kind(),
                NativeAttributeOperandError::UnsupportedDirective
                    | NativeAttributeOperandError::IncompleteValue
            ),
            "{source}: {:?}",
            failure.kind()
        );
        assert!(failure.syntax().is_none());
        assert_eq!(arena.allocated_bytes(), before);
    }
}

#[test]
fn verbatim_and_recovered_original_carriers_grant_no_handler_admission() {
    let arena = Allocator::default();
    for (source, expected) in [
        (
            "<template><button v-pre @click='return count'/></template>",
            NativeAttributeOperandError::Verbatim,
        ),
        (
            "<template><button @click='return count'></template>",
            NativeAttributeOperandError::RecoveredComponent,
        ),
    ] {
        let owner = selected(&arena, source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let event = element
            .attributes()
            .find(|attribute| attribute.surface().name.text == "@click")
            .unwrap();
        let error = owner.observe_attribute_handler(event).err().unwrap();
        assert_eq!(error.kind(), expected);
        assert!(error.syntax().is_none());
    }
}

#[test]
fn local_body_syntax_and_context_holes_retain_comments_and_cannot_join_event() {
    let arena = Allocator::default();
    for source in [
        "<template><button @click='return /x/uv; //kept'/></template>",
        "<template><button @click='import value from &quot;pkg&quot;; //kept'/></template>",
        "<template><button @click='return await value; //kept'/></template>",
        "<template><button @click='}; value; ()=>{ //kept'/></template>",
    ] {
        let owner = selected(&arena, source);
        let operand = observe(&owner, 0);
        assert!(operand.syntax().hole().is_some());
        assert!(operand.syntax().body().is_none());
        assert_eq!(
            operand.syntax().comments().next().unwrap().text().unwrap(),
            "//kept"
        );
        let element = owner.children().next().unwrap().into_element().unwrap();
        assert!(
            operand
                .admitted_for(&owner, element.attributes().next().unwrap())
                .is_none()
        );
    }
}

#[test]
fn single_header_visit_parks_then_joins_the_actual_original_event_token() {
    let arena = Allocator::default();
    let source = "<template><button @click='return count'/></template>";
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let mut visits = 0;
    let mut parked = alloc::vec::Vec::new();
    for attribute in element.attributes() {
        visits += 1;
        let operand = owner
            .observe_attribute_handler(attribute.reborrow())
            .unwrap();
        let original = operand.syntax().body().unwrap();
        parked.push(operand);
        parked.reserve(32);
        let admitted = parked[0].admitted_for(&owner, attribute).unwrap();
        assert_eq!(admitted.attribute().ordinal(), 0);
        assert!(core::ptr::eq(
            admitted.handler_body().unwrap().body(),
            original
        ));
    }
    assert_eq!(visits, 1);
}
