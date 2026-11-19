//! Number is an original modifier fact, never additional binding authority.

use super::*;

#[test]
fn number_uses_the_original_attribute_and_same_file_mutable_readwrite_binding() {
    for source in [
        "<script setup>let draft=''</script><template><input v-model.number='draft'/></template>",
        "<script setup>var count=7</script><template><input v-model.number=count /></template>",
        "\r\n<script setup lang='ts'>let café:string=' 雪🌸 '</script>\r\n<template><input v-model.number='café'/></template>\r\n",
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
        let span = record.input().operand().number_span().unwrap();
        assert_eq!(record.input().operand().trim_span(), None);
        assert_eq!(record.input().operand().lazy_span(), None);
        assert_eq!(attribute.name, "number");
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
fn number_with_a_late_header_failure_keeps_its_fact_without_a_published_model() {
    let arena = Allocator::default();
    let source = "<script setup>let draft='x'</script><template><input v-model.number='draft' :class='draft'/></template>";
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
        record
            .input()
            .operand()
            .number_span()
            .unwrap()
            .slice(source),
        "number"
    );
    assert!(
        record.binding().is_none() && record.original.is_none() && record.element_node().is_none()
    );
}

#[test]
fn number_cannot_make_an_original_const_declaration_writable() {
    let arena = Allocator::default();
    let source = "<script setup>const draft='x'</script><template><input v-model.number='draft'/></template>";
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
        record
            .input()
            .operand()
            .number_span()
            .unwrap()
            .slice(source),
        "number"
    );
    assert!(
        record.binding().is_none() && record.original.is_none() && record.element_node().is_none()
    );
}

#[test]
fn number_transition_unwinds_keep_actual_prefix_and_deny_complete_file_access() {
    for point in 1..=5 {
        let arena = Allocator::default();
        let source = "<script setup>let draft='snow'</script><template><input v-model.number='draft'/></template>";
        let mut owner = owner(&arena, source);
        {
            let mut walk = owner.begin().unwrap();
            let scope = walk.root.scope;
            let selected = walk.selected();
            let child = selected.children().next().unwrap();
            FAULT.with(|fault| fault.set(point));
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                    || walk.child(child.reborrow())
                ))
                .is_err()
            );
            assert_eq!(walk.root.scope, scope);
            assert_eq!(walk.root.facts.scopes.len(), 2);
            assert!(walk.root.facts.template_declarations.is_empty());
            assert_eq!(walk.cursor, 0);
            assert!(walk.root.facts.template_walk.interruption().is_some());
            if point == 1 {
                assert!(walk.root.facts.native_models().is_empty());
            } else {
                let [record] = walk.root.facts.native_models() else {
                    panic!("parked original Number Model")
                };
                assert_eq!(record.scope(), scope);
                assert_eq!(record.input().operand().raw_value(), "draft");
                assert_eq!(
                    record
                        .input()
                        .operand()
                        .number_span()
                        .unwrap()
                        .slice(source),
                    "number"
                );
                assert_eq!(record.input().operand().trim_span(), None);
                assert_eq!(record.input().operand().lazy_span(), None);
                assert!(core::ptr::eq(
                    record.input().operand().syntax().source().authored_root(),
                    source
                ));
                assert_eq!(record.input().expression().is_some(), point >= 3);
                assert_eq!(record.binding().is_some(), point >= 3);
                assert_eq!(record.original.is_some(), point >= 3);
                assert_eq!(record.element_node().is_some(), point == 5);
                if point < 5 {
                    assert_eq!(record.state(), NativeFileModelState::Pending);
                } else {
                    assert!(matches!(record.state(), NativeFileModelState::Admitted(_)));
                }
            }
            assert_eq!(walk.child(child).unwrap_err().kind, Kind::Interrupted);
            assert_eq!(walk.complete().unwrap_err().kind, Kind::Interrupted);
        }
        let output = owner.finish();
        assert!(output.view().is_err() && output.setup().is_err());
        assert!(output.retained_setup().is_some());
        assert!(core::ptr::eq(
            output.selected().component().block().root_source(),
            source
        ));
        if point == 3 || point == 4 {
            assert!(output.file().is_none());
            let rejected = output.rejected_file().unwrap();
            assert!(rejected.template_interruption().is_some());
            let [record] = rejected.native_models() else {
                panic!("whole pending owner")
            };
            assert_eq!(record.state(), NativeFileModelState::Pending);
            let [Op::Element(element)] = rejected.artifact().parts.root.ops.as_slice() else {
                panic!("retained pending input")
            };
            let [BindingOp::Model(model)] = element.bindings.as_slice() else {
                panic!("retained actual Model")
            };
            assert!(!record.accepts(model));
            let crate::expr::ExprRef::Js(read) = model.contract.read else {
                panic!("actual expression")
            };
            assert!(record.input().matches_expression(read));
            assert!(element.children.ops.is_empty());
        } else {
            let file = output.file().unwrap();
            assert!(!file.is_complete());
            assert!(file.template_interruption().is_some());
            if point == 5 {
                let [record] = file.native_models() else {
                    panic!("closed row, incomplete File")
                };
                let NativeFileModelState::Admitted(node) = record.state() else {
                    panic!("real closed allocation")
                };
                let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
                    panic!("closed input")
                };
                let [BindingOp::Model(model)] = element.bindings.as_slice() else {
                    panic!("closed Model")
                };
                assert!(record.accepts(model));
                assert!(record.accepts_element(record.element_node().unwrap(), element));
                assert!(file.expression(node).is_some());
                assert!(file.native_model(node).is_none());
            } else {
                assert!(file.artifact().root().ops.is_empty());
                assert_eq!(file.artifact().node_count(), 0);
            }
        }
    }
}
