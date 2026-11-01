//! Actual original Model, Element, setup declaration and read/write custody.

extern crate std;

use crate::file::{DeclarationKind, InitializerKind, Namespace, NativeFileModelState};
use crate::lang::js::NativeTemplateIssueKind as Kind;
use crate::op::{BindingOp, Op};
use crate::resolution::Usage;
use vize_l0::Allocator;

mod refusal;
mod support;
use support::{completed, owner};

std::thread_local! {
    static FAULT: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
}
fn inject(point: u8) {
    FAULT.with(|fault| {
        if fault.get() == point {
            fault.set(0);
            panic!("interrupted original Model custody at {point}");
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

#[test]
fn original_five_primitive_let_var_models_keep_one_real_readwrite_and_no_alias_scope() {
    for (source, kind) in [
        (
            "<script setup>let draft='snow'</script><template><input v-model='draft'/></template>",
            DeclarationKind::Let,
        ),
        (
            "<script setup>var draft=7</script><template><input v-model='draft'/></template>",
            DeclarationKind::Var,
        ),
        (
            "<script setup>let draft=false</script><template><input v-model='draft'/></template>",
            DeclarationKind::Let,
        ),
        (
            "<script setup>var draft=null</script><template><input v-model='draft'/></template>",
            DeclarationKind::Var,
        ),
        (
            "\r\n<script setup lang='ts'>let café:bigint=9n</script>\r\n<template><input v-model=\"café\" /></template>\r\n",
            DeclarationKind::Let,
        ),
    ] {
        let arena = Allocator::default();
        let output = completed(&arena, source);
        let setup = output.setup().unwrap();
        let file = output.file().unwrap();
        assert!(file.is_complete() && output.view().is_ok());
        assert!(core::ptr::eq(setup.file(), file));
        let [unit] = file.units() else {
            panic!("one actual Program")
        };
        assert_eq!(file.scopes().len(), 2);
        assert_eq!(file.artifact().node_count(), 2);
        let [record] = file.native_models() else {
            panic!("one original Model")
        };
        let NativeFileModelState::Admitted(node) = record.state() else {
            panic!("closed Model")
        };
        let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
            panic!("real input")
        };
        let [BindingOp::Model(model)] = element.bindings.as_slice() else {
            panic!("real attached Model")
        };
        let element_node = record.element_node().unwrap();
        assert_ne!(node, element_node);
        assert!(record.accepts(model) && record.accepts_element(element_node, element));
        assert!(core::ptr::eq(record, file.native_model(node).unwrap()));
        assert!(file.artifact().scopes().get(node).is_none());
        assert_eq!(record.scope(), unit.scope);
        assert_eq!(record.scope(), setup.scope());
        assert_eq!(element.tag, "input");
        assert!(element.attributes.is_empty() && element.children.ops.is_empty());
        assert!(model.argument.is_none() && model.attributes.is_empty());
        assert_eq!(record.input().attribute_span(), Some(model.span));
        assert_eq!(
            record.input().operand().name_span().slice(source),
            "v-model"
        );
        let (crate::expr::ExprRef::Js(read), crate::expr::ExprRef::Js(write)) =
            (model.contract.read, model.contract.write)
        else {
            panic!("same original expression")
        };
        assert!(core::ptr::eq(read, write));
        assert!(record.input().matches_expression(read));
        let resolution = file.expression(node).unwrap();
        assert_eq!(resolution.scope(), Some(unit.scope));
        let table = resolution.table().unwrap();
        let resolved = table.expression();
        assert!(!record.input().matches_expression(resolved));
        assert!(core::ptr::eq(resolved.ast, read.ast));
        assert!(core::ptr::eq(resolved.source, read.source));
        assert_eq!(resolved.span, read.span);
        assert!(core::ptr::eq(
            resolved.coordinates.unwrap(),
            read.coordinates.unwrap()
        ));
        let [occurrence] = table.occurrences() else {
            panic!("one genuine ReadWrite occurrence")
        };
        assert_eq!(occurrence.usage, Usage::ReadWrite);
        assert!(!occurrence.shorthand && !occurrence.constructor);
        assert_eq!(record.binding(), Some(occurrence.binding));
        let binding = resolution.binding(occurrence.binding).unwrap();
        assert!(resolution.accepts(binding));
        assert!(setup.binding(binding).is_ok());
        let declaration = binding.declaration().unwrap();
        assert_eq!(declaration.id, occurrence.binding);
        assert_eq!(declaration.kind, kind);
        assert_eq!(declaration.namespace, Namespace::Value);
        assert_eq!(declaration.initializer, InitializerKind::PrimitiveLiteral);
        assert!(declaration.is_direct_program());
        assert_eq!(declaration.scope, unit.scope);
        assert_eq!(declaration.unit, unit.id);
        assert_eq!(declaration.name.as_str(), occurrence.name);
        assert_eq!(
            record.input().operand().value_span().slice(source),
            occurrence.name
        );
        let identifier = read.ast.get_identifier_reference().unwrap();
        assert_eq!(
            read.ast_span_to_source(identifier.span),
            Some(occurrence.span)
        );
        assert_eq!(
            read.authored_span(occurrence.span).unwrap().slice(source),
            occurrence.name
        );
        assert!(file.native_interpolations().is_empty());
    }
}

#[test]
fn equal_foreign_model_nodes_elements_and_bindings_never_satisfy_same_file_custody() {
    let arena = Allocator::default();
    let source = "<script setup>let draft=''</script><template><input v-model='draft'/></template>";
    let own = completed(&arena, source);
    let foreign = completed(&arena, source);
    let file = own.file().unwrap();
    let other = foreign.file().unwrap();
    let [record] = file.native_models() else {
        panic!("own original")
    };
    let [other_record] = other.native_models() else {
        panic!("foreign original")
    };
    assert_eq!(record.state(), other_record.state());
    assert_eq!(record.binding(), other_record.binding());
    assert_eq!(record.element_node(), other_record.element_node());
    let NativeFileModelState::Admitted(node) = record.state() else {
        panic!("actual node")
    };
    let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
        panic!("own input")
    };
    let [Op::Element(other_element)] = other.artifact().root().ops.as_slice() else {
        panic!("foreign input")
    };
    let [BindingOp::Model(model)] = element.bindings.as_slice() else {
        panic!("own Model")
    };
    let [BindingOp::Model(other_model)] = other_element.bindings.as_slice() else {
        panic!("foreign Model")
    };
    assert!(record.accepts(model));
    assert!(!record.accepts(other_model));
    assert!(record.accepts_element(record.element_node().unwrap(), element));
    assert!(!record.accepts_element(record.element_node().unwrap(), other_element));
    assert!(
        !file
            .expression(node)
            .unwrap()
            .accepts(other.binding(record.binding().unwrap()).unwrap())
    );
    assert!(
        !record
            .input()
            .matches_expression(other_record.input().expression().unwrap())
    );
    let copied = *record.input().expression().unwrap();
    assert!(!record.input().matches_expression(&copied));
    let mut interrupted = owner(&arena, source);
    {
        let mut walk = interrupted.begin().unwrap();
        let foreign_child = foreign.selected().children().next().unwrap();
        assert_eq!(
            walk.child(foreign_child).unwrap_err().kind,
            Kind::InvalidEvent
        );
        assert!(walk.root.facts.native_models().is_empty());
        assert_eq!(walk.complete().unwrap_err().kind, Kind::InvalidEvent);
    }
    let interrupted = interrupted.finish();
    assert!(interrupted.view().is_err());
    assert!(interrupted.file().unwrap().native_models().is_empty());
}

#[test]
fn caught_model_transition_unwinds_keep_actual_prefix_but_never_publish_complete_file_access() {
    for point in 1..=5 {
        let arena = Allocator::default();
        let source =
            "<script setup>let draft='snow'</script><template><input v-model='draft'/></template>";
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
                    panic!("parked actual Model")
                };
                assert_eq!(record.scope(), scope);
                assert_eq!(record.input().operand().raw_value(), "draft");
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

#[test]
fn interrupted_model_preparation_failure_keeps_the_original_failure_without_a_model_or_node() {
    let arena = Allocator::default();
    let source = "<script setup>let draft=''</script><template><input v-model/></template>";
    let mut owner = owner(&arena, source);
    {
        let mut walk = owner.begin().unwrap();
        let child = walk.selected().children().next().unwrap();
        FAULT.with(|fault| fault.set(6));
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| walk.child(child))).is_err()
        );
        assert!(walk.root.facts.native_models().is_empty());
        let [failure] = walk.root.facts.native_model_failures() else {
            panic!("parked whole L1 failure")
        };
        assert_eq!(
            failure.kind(),
            vize_l1::markup::NativeAttributeOperandError::IncompleteValue
        );
        assert_eq!(walk.complete().unwrap_err().kind, Kind::Interrupted);
    }
    let output = owner.finish();
    assert!(output.view().is_err() && output.setup().is_err());
    let file = output.file().unwrap();
    assert!(!file.is_complete());
    assert!(file.template_interruption().is_some());
    assert_eq!(file.artifact().node_count(), 0);
    assert!(file.native_models().is_empty());
    let [failure] = file.native_model_failures() else {
        panic!("retained complete failure")
    };
    assert_eq!(
        failure.kind(),
        vize_l1::markup::NativeAttributeOperandError::IncompleteValue
    );
}
