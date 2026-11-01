//! A normally closed original type/Model owns the input kind, never the write.

use super::*;

const ORIGINAL_INLINE: &str = "<script setup>let draft=''</script><template><input v-model='draft' type='checkbox'/></template>";

#[test]
fn checkbox_original_inline_successor_and_both_header_orders_keep_real_type_and_write() {
    for source in [
        ORIGINAL_INLINE,
        "<script setup>let enabled=false</script><template><input type=\"checkbox\" v-model=\"enabled\" /></template>",
        "\r\n<script setup lang='ts'>/* 雪🌸 import { fake } from 'vue'; */var café:boolean=true</script>\r\n<template><input v-model='café' type='checkbox'/></template>\r\n",
    ] {
        let arena = Allocator::default();
        let output = completed(&arena, source);
        let file = output.file().unwrap();
        let setup = output.setup().unwrap();
        assert!(file.is_complete() && output.view().is_ok());
        let [record] = file.native_models() else {
            panic!("one Model")
        };
        let NativeFileModelState::Admitted(node) = record.state() else {
            panic!("closed Model")
        };
        assert_eq!(record.kind(), Some(NativeFileModelKind::Checkbox));
        let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
            panic!("input")
        };
        let [BindingOp::Model(model)] = element.bindings.as_slice() else {
            panic!("actual Model")
        };
        assert!(
            record.accepts(model)
                && record.accepts_element(record.element_node().unwrap(), element)
        );
        assert!(model.argument.is_none() && model.attributes.is_empty());
        let [attribute] = element.attributes.as_slice() else {
            panic!("one original type")
        };
        assert_eq!(attribute.name, "type");
        assert_eq!(attribute.value, Some("checkbox"));
        let value = file.native_attribute_value_for(0, element, 0).unwrap();
        let observation = value.observation().unwrap();
        assert!(core::ptr::eq(value.attribute().unwrap(), attribute));
        assert_eq!(observation.raw_value(), "checkbox");
        assert!(observation.source().decode_map().is_none());
        assert!(core::ptr::eq(observation.source().authored_root(), source));
        assert!(core::ptr::eq(
            observation.source().text(),
            attribute.value.unwrap()
        ));
        assert_eq!(observation.source().span().slice(source), "checkbox");
        assert!(file.native_attribute_value_for(0, element, 1).is_none());
        let resolution = file.expression(node).unwrap();
        let [occurrence] = resolution.table().unwrap().occurrences() else {
            panic!("one write")
        };
        assert_eq!(occurrence.usage, Usage::ReadWrite);
        assert_eq!(record.binding(), Some(occurrence.binding));
        let target = resolution.binding(occurrence.binding).unwrap();
        assert!(resolution.accepts(target) && setup.binding(target).is_ok());
        let declaration = target.declaration().unwrap();
        assert!(matches!(
            declaration.kind,
            DeclarationKind::Let | DeclarationKind::Var
        ));
        assert!(declaration.is_direct_program());
        assert_eq!(declaration.scope, record.scope());
        assert_eq!(declaration.unit, setup.unit());
        assert_eq!(declaration.initializer, InitializerKind::PrimitiveLiteral);
    }
}

#[test]
fn type_owner_and_model_owner_cannot_be_borrowed_from_equal_byte_files() {
    let arena = Allocator::default();
    let left = std::string::String::from(ORIGINAL_INLINE);
    let right = std::string::String::from(ORIGINAL_INLINE);
    let actual = completed(&arena, &left);
    let foreign = completed(&arena, &right);
    let file = actual.file().unwrap();
    let other = foreign.file().unwrap();
    let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
        panic!("actual")
    };
    let [Op::Element(foreign_element)] = other.artifact().root().ops.as_slice() else {
        panic!("foreign")
    };
    let [record] = file.native_models() else {
        panic!("actual Model")
    };
    let [foreign_record] = other.native_models() else {
        panic!("foreign Model")
    };
    assert_eq!(record.kind(), foreign_record.kind());
    assert!(file.native_attribute_value_for(0, element, 0).is_some());
    assert!(
        file.native_attribute_value_for(0, foreign_element, 0)
            .is_none()
    );
    assert!(!record.accepts_element(foreign_record.element_node().unwrap(), foreign_element));
}

#[test]
fn checkbox_kind_does_not_admit_extra_or_encoded_type_surface_or_const_writes() {
    for (script, template) in [
        (
            "let draft=false",
            "<input type='checkbox' v-model.trim='draft'/>",
        ),
        (
            "let draft=false",
            "<input type='checkbox' v-model.lazy='draft'/>",
        ),
        (
            "let draft=false",
            "<input type='checkbox' v-model.number='draft'/>",
        ),
        (
            "let draft=false",
            "<input type='&#99;heckbox' v-model='draft'/>",
        ),
        (
            "let draft=false",
            "<input type='CHECKBOX' v-model='draft'/>",
        ),
        ("let draft=false", "<input type='text' v-model='draft'/>"),
        ("let draft=false", "<input type='radio' v-model='draft'/>"),
        (
            "let draft=false",
            "<input type='checkbox' v-model='draft' disabled/>",
        ),
        (
            "let draft=false",
            "<input type='checkbox' v-model='draft' value='x'/>",
        ),
        (
            "let draft=false",
            "<input type='checkbox' v-model='draft' :class='draft'/>",
        ),
        (
            "let draft=false",
            "<input type='checkbox' type='checkbox' v-model='draft'/>",
        ),
        (
            "const draft=false",
            "<input type='checkbox' v-model='draft'/>",
        ),
    ] {
        let source = std::format!("<script setup>{script}</script><template>{template}</template>");
        let arena = Allocator::default();
        let mut original = owner(&arena, &source);
        {
            let mut walk = original.begin_setup().unwrap();
            let child = walk.selected().children().next().unwrap();
            assert!(walk.child(child).is_err(), "{source}");
            assert!(walk.complete().is_err());
        }
        let output = original.finish();
        assert!(output.view().is_err() && output.setup().is_err());
        let file = output.file().unwrap();
        assert!(!file.is_complete());
        for record in file.native_models() {
            assert_eq!(record.kind(), None);
            assert!(record.element_node().is_none());
        }
    }
}

#[test]
fn checkbox_unwinds_keep_the_actual_prefix_and_never_publish_an_incomplete_file() {
    for point in 1..=5 {
        let arena = Allocator::default();
        let mut original = owner(&arena, ORIGINAL_INLINE);
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut walk = original.begin_setup().unwrap();
            let child = walk.selected().children().next().unwrap();
            FAULT.with(|fault| fault.set(point));
            walk.child(child).unwrap();
        }));
        FAULT.with(|fault| fault.set(0));
        assert!(caught.is_err(), "point {point}");
        let output = original.finish();
        assert!(output.view().is_err() && output.setup().is_err());
        let file = output.file().unwrap();
        assert!(!file.is_complete());
        if point == 1 {
            assert!(file.native_models().is_empty());
        } else {
            let [record] = file.native_models() else {
                panic!("retained Model prefix")
            };
            if point < 5 {
                assert_eq!(record.kind(), None);
                assert_eq!(record.state(), NativeFileModelState::Pending);
                assert!(record.element_node().is_none());
            } else {
                let NativeFileModelState::Admitted(node) = record.state() else {
                    panic!("real closed prefix")
                };
                assert_eq!(record.kind(), Some(NativeFileModelKind::Checkbox));
                assert!(record.element_node().is_some());
                assert!(file.native_model(node).is_none());
            }
        }
    }
}
