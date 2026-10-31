extern crate std;

use super::*;
use crate::file::{InitializerKind, NativeFileConditionalState};
use crate::lang::js::{NativeTemplateFile, NativeTemplateOwner};
use crate::op::Op;
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    container::{Vue, vue::DescriptorOptions},
    markup::SurfaceParseOptions,
};

std::thread_local! {
    static FAULT: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
}

fn inject(point: u8) {
    FAULT.with(|fault| {
        if fault.get() == point {
            fault.set(0);
            panic!("interrupted original conditional custody at {point}");
        }
    });
}
pub(super) fn before_observe() {
    inject(1);
}
pub(super) fn after_park() {
    inject(2);
}
pub(in crate::file::region::native) fn after_mint() {
    inject(3);
}
pub(in crate::file::region::native) fn after_body() {
    inject(4);
}
pub(in crate::file::region::native) fn after_close() {
    inject(5);
}
pub(super) fn after_failure_park() {
    inject(6);
}

fn owner<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateOwner<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let selected = NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap();
    let mut owner = NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("owner"));
    if owner.selected().setup().is_some() {
        owner.parse_setup_program().unwrap();
    }
    owner
}

fn completed<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateFile<'a> {
    let mut owner = owner(arena, source);
    {
        let mut walk = owner.begin().unwrap();
        for child in walk.selected().children() {
            walk.child(child).unwrap();
        }
        walk.complete().unwrap();
    }
    owner.finish()
}

#[test]
fn native_boolean_if_keeps_actual_input_read_scope_and_if_allocation_without_alias_scope() {
    for (source, expected) in [
        (
            "<script setup>let ready=true</script><template><div v-if='ready'>kept</div></template>",
            true,
        ),
        (
            "<script setup>const ready=false</script><template><p v-if='ready'/></template>",
            false,
        ),
        (
            "<script setup lang='ts'>var ready:boolean=true</script><template><i v-if='ready'>yes</i></template>",
            true,
        ),
        (
            "\r\n<script setup>const prêt=false</script>\r\n<template><div v-if='prêt'>雪🌸</div></template>\r\n",
            false,
        ),
    ] {
        let arena = Allocator::default();
        let output = completed(&arena, source);
        assert!(output.view().is_ok());
        let file = output.file().unwrap();
        assert!(file.is_complete());
        let [unit] = file.units() else {
            panic!("one original setup");
        };
        assert_eq!(file.scopes().len(), 2);
        let [record] = file.native_conditionals() else {
            panic!("one actual If input");
        };
        let NativeFileConditionalState::Admitted(node) = record.state() else {
            panic!("closed If");
        };
        let [Op::If(original)] = file.artifact().root().ops.as_slice() else {
            panic!("real If root");
        };
        assert!(record.accepts(original));
        assert!(core::ptr::eq(
            record,
            file.native_conditional(node).unwrap()
        ));
        assert!(file.artifact().scopes().get(node).is_none());
        assert_eq!(record.scope(), unit.scope);
        let resolution = file.expression(node).unwrap();
        assert_eq!(resolution.scope(), Some(unit.scope));
        let table = resolution.table().unwrap();
        assert!(record.input().matches_expression(table.expression()));
        let [occurrence] = table.occurrences() else {
            panic!("one real read");
        };
        assert_eq!(occurrence.usage, crate::resolution::Usage::Read);
        assert_eq!(record.binding(), Some(occurrence.binding));
        let binding = resolution.binding(occurrence.binding).unwrap();
        let declaration = binding.declaration().unwrap();
        assert_eq!(declaration.boolean_initializer(), Some(expected));
        assert_eq!(declaration.initializer, InitializerKind::PrimitiveLiteral);
        assert!(declaration.is_direct_program());
        assert_eq!(declaration.scope, unit.scope);
        assert_eq!(declaration.unit, unit.id);
        assert_eq!(declaration.name.as_str(), occurrence.name);
        let [branch] = original.branches.as_slice() else {
            panic!("one original branch");
        };
        let Some(crate::expr::ExprRef::Js(condition)) = branch.condition else {
            panic!("actual condition");
        };
        assert!(record.input().matches_expression(condition));
        let [Op::Element(element)] = branch.region.ops.as_slice() else {
            panic!("one original element");
        };
        assert!(element.attributes.is_empty() && element.bindings.is_empty());
        assert_eq!(branch.span, original.span);
        assert_eq!(element.span, original.span);
        let original_element = output
            .selected()
            .children()
            .next()
            .unwrap()
            .into_element()
            .unwrap();
        let original_block = original_element.component().block();
        let opening = original_block
            .span_of(original_element.surface().open.lt_name.text)
            .unwrap();
        let closing = match &original_element.surface().close {
            vize_l1::markup::ElementClose::Present(close) => {
                original_block.span_of(close.gt.text).unwrap()
            }
            vize_l1::markup::ElementClose::NotExpected => original_block
                .span_of(original_element.surface().open.gt.text)
                .unwrap(),
            _ => panic!("actual normal original close"),
        };
        assert_eq!(
            original.span,
            vize_l0::Span::new(opening.start, closing.end)
        );
        assert_eq!(record.input().operand().name_span().slice(source), "v-if");
        assert_eq!(
            record.input().operand().value_span().slice(source),
            occurrence.name
        );
        assert!(file.native_interpolations().is_empty());
    }
}

#[test]
fn equal_foreign_if_nodes_and_bindings_do_not_satisfy_the_original_file_receipt() {
    let arena = Allocator::default();
    let source =
        "<script setup>let ready=true</script><template><div v-if='ready'>kept</div></template>";
    let output = completed(&arena, source);
    let foreign = completed(&arena, source);
    let file = output.file().unwrap();
    let other = foreign.file().unwrap();
    let [record] = file.native_conditionals() else {
        panic!("own If");
    };
    let [other_record] = other.native_conditionals() else {
        panic!("foreign If");
    };
    let NativeFileConditionalState::Admitted(node) = record.state() else {
        panic!("own node");
    };
    assert_eq!(other_record.state(), record.state());
    let [Op::If(own_if)] = file.artifact().root().ops.as_slice() else {
        panic!("own allocation");
    };
    let [Op::If(other_if)] = other.artifact().root().ops.as_slice() else {
        panic!("other allocation");
    };
    assert!(record.accepts(own_if));
    assert!(!record.accepts(other_if));
    let resolution = file.expression(node).unwrap();
    let foreign_binding = other.binding(record.binding().unwrap()).unwrap();
    assert!(!resolution.accepts(foreign_binding));
    let expression = resolution.table().unwrap().expression();
    let copied = *expression;
    assert!(!record.input().matches_expression(&copied));
    let neutral = crate::expr::JsExpr {
        coordinates: None,
        ..*expression
    };
    assert!(!record.input().matches_expression(&neutral));
    assert!(
        !record
            .input()
            .matches_expression(other_record.input().expression().unwrap())
    );
}

#[test]
fn original_literal_numeric_and_dynamic_refusals_do_not_publish_an_if_query() {
    for source in [
        "<template><div v-if=\"true\">kept</div></template>",
        "<template><div v-if='true'/></template>",
        "<script setup>const value=1;</script><template><p v-if='value'>{{value}}</p></template>",
        "<script setup>const first=1;const second=2;</script><template><p v-if=first>{{first /* kept */}}{{second}}</p></template>",
    ] {
        let arena = Allocator::default();
        let mut owner = owner(&arena, source);
        {
            let mut walk = owner.begin().unwrap();
            let child = walk.selected().children().next().unwrap();
            assert_eq!(walk.child(child).unwrap_err().kind, Kind::UnsupportedChild);
            assert!(walk.complete().is_err());
        }
        let output = owner.finish();
        assert!(output.view().is_err());
        let file = output.file().unwrap();
        assert!(!file.is_complete());
        assert_eq!(file.artifact().node_count(), 0);
        let [record] = file.native_conditionals() else {
            panic!("retained original conditional");
        };
        assert_eq!(
            record.state(),
            NativeFileConditionalState::Refused(Kind::UnsupportedChild)
        );
        assert!(record.original.is_none());
        assert!(core::ptr::eq(
            record.input().operand().syntax().source().authored_root(),
            source
        ));
    }
}

mod interruption;
mod refusal;
