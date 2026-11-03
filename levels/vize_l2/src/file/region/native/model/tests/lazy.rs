//! Lazy is an original modifier fact, never additional binding authority.

use super::*;

#[test]
fn lazy_uses_the_original_attribute_and_same_file_mutable_readwrite_binding() {
    for source in [
        "<script setup>let draft='x'</script><template><input v-model.lazy='draft'/></template>",
        "<script setup>var count=7</script><template><input v-model.lazy=count /></template>",
        "\r\n<script setup lang='ts'>let café:string=' 雪🌸 '</script>\r\n<template><input v-model.lazy='café'/></template>\r\n",
    ] {
        let arena = Allocator::default();
        let output = completed(&arena, source);
        let setup = output.setup().unwrap();
        let file = output.file().unwrap();
        assert!(file.is_complete() && output.view().is_ok());
        let [record] = file.native_models() else {
            panic!("one original")
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
            panic!("one original modifier")
        };
        let span = record.input().operand().lazy_span().unwrap();
        assert_eq!(record.input().operand().trim_span(), None);
        assert_eq!(attribute.name, "lazy");
        assert_eq!(attribute.span, span);
        assert_eq!(attribute.value, None);
        assert!(core::ptr::eq(attribute.name, span.slice(source)));
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
        assert!(declaration.is_direct_program());
        assert_eq!(declaration.scope, setup.scope());
        assert_eq!(declaration.unit, setup.unit());
        assert_eq!(declaration.namespace, Namespace::Value);
        assert!(file.native_interpolations().is_empty());
    }
}

#[test]
fn lazy_with_a_late_header_failure_keeps_its_fact_without_a_published_model() {
    let arena = Allocator::default();
    let source = "<script setup>let draft='x'</script><template><input v-model.lazy='draft' :class='draft'/></template>";
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
        panic!("original retained")
    };
    assert_eq!(record.state(), NativeFileModelState::Pending);
    assert_eq!(
        record.input().operand().lazy_span().unwrap().slice(source),
        "lazy"
    );
    assert!(
        record.binding().is_none() && record.original.is_none() && record.element_node().is_none()
    );
}

#[test]
fn lazy_cannot_make_an_original_const_declaration_writable() {
    let arena = Allocator::default();
    let source =
        "<script setup>const draft='x'</script><template><input v-model.lazy='draft'/></template>";
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
        panic!("original retained")
    };
    assert_eq!(
        record.state(),
        NativeFileModelState::Refused(Kind::UnsupportedChild)
    );
    assert_eq!(
        record.input().operand().lazy_span().unwrap().slice(source),
        "lazy"
    );
    assert!(
        record.binding().is_none() && record.original.is_none() && record.element_node().is_none()
    );
}
