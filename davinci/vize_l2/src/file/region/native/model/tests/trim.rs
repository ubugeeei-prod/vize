//! One authored trim attribute cannot introduce or replace write authority.

use super::*;

#[test]
fn trim_model_retains_its_original_attribute_binding_and_same_file_readwrite_owner() {
    for source in [
        "<script setup>let draft=' x '</script><template><input v-model.trim='draft'/></template>",
        "<script setup>var draft=7</script><template><input v-model.trim=draft /></template>",
        "\r\n<script setup lang='ts'>let café:string=' 雪🌸 '</script>\r\n<template><input v-model.trim='café'/></template>\r\n",
    ] {
        let arena = Allocator::default();
        let output = completed(&arena, source);
        let setup = output.setup().unwrap();
        let file = output.file().unwrap();
        assert!(file.is_complete() && output.view().is_ok());
        let [record] = file.native_models() else {
            panic!("actual model")
        };
        let NativeFileModelState::Admitted(node) = record.state() else {
            panic!("closed")
        };
        let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
            panic!("input")
        };
        let [BindingOp::Model(model)] = element.bindings.as_slice() else {
            panic!("model")
        };
        let [attribute] = model.attributes.as_slice() else {
            panic!("one real modifier")
        };
        let span = record.input().operand().trim_span().unwrap();
        assert_eq!(span, attribute.span);
        assert_eq!(attribute.name, "trim");
        assert_eq!(attribute.value, None);
        assert!(core::ptr::eq(attribute.name, span.slice(source)));
        assert_eq!(
            record.input().operand().name_span().slice(source),
            "v-model.trim"
        );
        assert!(record.accepts(model));
        assert!(record.accepts_element(record.element_node().unwrap(), element));
        let resolution = file.expression(node).unwrap();
        let [occurrence] = resolution.table().unwrap().occurrences() else {
            panic!("one use")
        };
        assert_eq!(occurrence.usage, Usage::ReadWrite);
        assert!(!occurrence.shorthand && !occurrence.constructor);
        assert_eq!(record.binding(), Some(occurrence.binding));
        let target = resolution.binding(occurrence.binding).unwrap();
        assert!(resolution.accepts(target) && setup.binding(target).is_ok());
        let declaration = target.declaration().unwrap();
        assert!(matches!(
            declaration.kind,
            DeclarationKind::Let | DeclarationKind::Var
        ));
        assert_eq!(declaration.scope, setup.scope());
        assert_eq!(declaration.unit, setup.unit());
        assert_eq!(declaration.namespace, Namespace::Value);
        assert!(declaration.is_direct_program());
        assert!(element.attributes.is_empty() && element.children.ops.is_empty());
        assert!(file.native_interpolations().is_empty());
    }
}

#[test]
fn late_trim_model_header_refusal_keeps_the_modifier_but_publishes_no_model_allocation() {
    let arena = Allocator::default();
    let source = "<script setup>let draft=''</script><template><input v-model.trim='draft' :class='draft'/></template>";
    let mut owner = owner(&arena, source);
    {
        let mut walk = owner.begin_setup().unwrap();
        let child = walk.selected().children().next().unwrap();
        assert_eq!(walk.child(child).unwrap_err().kind, Kind::UnsupportedChild);
        assert_eq!(walk.complete().unwrap_err().kind, Kind::UnsupportedChild);
    }
    let output = owner.finish();
    assert!(output.view().is_err() && output.setup().is_err());
    let file = output.file().unwrap();
    assert!(!file.is_complete());
    let [record] = file.native_models() else {
        panic!("retained original")
    };
    assert_eq!(record.state(), NativeFileModelState::Pending);
    assert_eq!(
        record.input().operand().trim_span().unwrap().slice(source),
        "trim"
    );
    assert!(
        record.binding().is_none() && record.original.is_none() && record.element_node().is_none()
    );
}

#[test]
fn trim_without_a_mutable_binding_still_refuses_before_any_attached_model() {
    let arena = Allocator::default();
    let source = "<script setup>const draft=' x '</script><template><input v-model.trim='draft'/></template>";
    let mut owner = owner(&arena, source);
    {
        let mut walk = owner.begin_setup().unwrap();
        let child = walk.selected().children().next().unwrap();
        assert_eq!(walk.child(child).unwrap_err().kind, Kind::UnsupportedChild);
        assert_eq!(walk.complete().unwrap_err().kind, Kind::UnsupportedChild);
    }
    let output = owner.finish();
    assert!(output.view().is_err() && output.setup().is_err());
    let [record] = output.file().unwrap().native_models() else {
        panic!("retained model")
    };
    assert_eq!(
        record.state(),
        NativeFileModelState::Refused(Kind::UnsupportedChild)
    );
    assert_eq!(
        record.input().operand().trim_span().unwrap().slice(source),
        "trim"
    );
    assert!(
        record.binding().is_none() && record.original.is_none() && record.element_node().is_none()
    );
}
