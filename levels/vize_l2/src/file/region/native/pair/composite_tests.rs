//! Four exact H874 whole-source refusals receive named composite successors.
//!
//! Original pair/text_tests/refusal.rs blob
//! b80941545aefc95bc639429d05d2dc6439102ea0, SHA256
//! b7ec9b8c8540ce926756aa7ad3976669785292fea940fa7511b3a8226d3aa2a6.
//! Its complete oracle retains both headers, refuses the conditional without
//! its operation pointer, leaves cursor zero and denies complete/view/setup.

extern crate std;

use super::Kind;
use super::tests::{completed, owner};
use crate::expr::ExprRef;
use crate::file::{
    FileArtifact, Namespace as BindingNamespace, NativeFileConditionalState,
    NativeFileInterpolationState as State,
};
use crate::lang::js::NativeTemplateFile;
use crate::op::Op;
use crate::resolution::Usage;
use crate::walk::NodeRef;
use vize_l0::{Allocator, id::NodeId};
use vize_l1::markup::SurfaceChild;

mod refusal;

fn node<'a>(file: &FileArtifact<'a>, target: &Op<'a>) -> NodeId {
    let mut found = None;
    file.artifact()
        .visit_nodes(&mut |id, actual| {
            if let NodeRef::Op(actual) = actual
                && core::ptr::eq(actual, target)
            {
                found = Some(id);
            }
        })
        .unwrap();
    found.unwrap()
}

fn assert_original(output: &NativeTemplateFile<'_>, shapes: [&str; 2]) {
    let setup = output.setup().unwrap();
    let file = output.file().unwrap();
    let source = file.artifact().source();
    assert!(file.is_complete() && output.view().is_ok());
    assert_eq!((file.units().len(), file.scopes().len()), (1, 2));
    assert!(file.template_declarations().next().is_none());
    let [conditional] = file.native_conditionals() else {
        panic!("original pair")
    };
    let NativeFileConditionalState::Admitted(if_node) = conditional.state() else {
        panic!("normally closed pair")
    };
    let [Op::If(original)] = file.artifact().root().ops.as_slice() else {
        panic!("actual If")
    };
    assert!(conditional.accepts(original));
    let [first, last] = original.branches.as_slice() else {
        panic!("two actual regions")
    };
    assert_eq!(first.span.end, last.span.start);
    let Some(ExprRef::Js(condition)) = first.condition else {
        panic!("original condition")
    };
    assert!(conditional.input().matches_expression(condition));
    assert_eq!(conditional.scope(), setup.scope());
    let mut root_children = output.selected().children();
    for (index, branch) in [first, last].into_iter().enumerate() {
        let [Op::Element(element)] = branch.region.ops.as_slice() else {
            panic!("one branch Element")
        };
        let shape = match element.children.ops.as_slice() {
            [] => "",
            [Op::Text(_)] => "T",
            [Op::Interpolation(_)] => "I",
            [Op::Text(_), Op::Interpolation(_)] => "TI",
            [Op::Interpolation(_), Op::Text(_)] => "IT",
            _ => panic!("bounded genuine body parts"),
        };
        assert_eq!(shape, *shapes.get(index).unwrap());
        assert!(element.attributes.is_empty() && element.bindings.is_empty());
        let original_element = root_children.next().unwrap().into_element().unwrap();
        let mut children = original_element.children();
        assert_eq!(children.len(), element.children.ops.len());
        let mut previous = None;
        for op in &element.children.ops {
            let original_child = children.next().unwrap();
            let actual_node = node(file, op);
            let span = NodeRef::Op(op).span();
            if let Some((previous_id, previous_span)) = previous {
                assert!(previous_id < actual_node.index());
                assert_eq!(previous_span, span.start);
            }
            previous = Some((actual_node.index(), span.end));
            match (op, original_child.surface()) {
                (Op::Text(text), SurfaceChild::Text(token)) => {
                    assert_eq!(
                        text.span,
                        original_element
                            .component()
                            .block()
                            .span_of(token.text)
                            .unwrap()
                    );
                    assert_eq!(text.span.slice(source), text.content);
                    assert!(core::ptr::eq(text.content, token.text));
                }
                (Op::Interpolation(interpolation), SurfaceChild::Interpolation(_)) => {
                    let record = file.native_interpolation(actual_node).unwrap();
                    assert_eq!(record.state(), State::Admitted(actual_node));
                    assert!(
                        record
                            .input()
                            .operand()
                            .admitted_for(output.selected(), original_child.reborrow())
                            .is_some()
                    );
                    let ExprRef::Js(expression) = interpolation.expression else {
                        panic!("actual AST")
                    };
                    assert_ne!(actual_node, if_node);
                    assert!(!core::ptr::eq(condition.ast, expression.ast));
                    let syntax = record.input().operand().syntax();
                    assert!(core::ptr::eq(syntax.expression().unwrap(), expression.ast));
                    assert!(core::ptr::eq(syntax.source().text(), expression.source));
                    assert!(core::ptr::eq(syntax.source().authored_root(), source));
                    assert_eq!(interpolation.span, record.input().operand().full_span());
                    let resolution = file.expression(actual_node).unwrap();
                    assert_eq!(resolution.scope(), Some(setup.scope()));
                    let table = resolution.table().unwrap();
                    assert!(core::ptr::eq(table.expression().ast, expression.ast));
                    assert!(core::ptr::eq(table.expression().source, expression.source));
                    assert!(core::ptr::eq(
                        table.expression().coordinates.unwrap(),
                        expression.coordinates.unwrap()
                    ));
                    let [read] = table.occurrences() else {
                        panic!("one current READ")
                    };
                    assert_eq!(read.usage, Usage::Read);
                    assert_eq!(
                        expression.authored_span(read.span),
                        Some(syntax.source().span())
                    );
                    assert_eq!(syntax.source().span().slice(source), read.name);
                    let binding = resolution.binding(read.binding).unwrap();
                    assert!(resolution.accepts(binding) && setup.binding(binding).is_ok());
                    assert!(
                        binding.same_owner(
                            file.lookup(setup.scope(), read.name, BindingNamespace::Value)
                                .unwrap()
                        )
                    );
                    let declaration = binding.declaration().unwrap();
                    assert!(
                        declaration.is_direct_program() && declaration.initializer.is_primitive()
                    );
                    assert_eq!(
                        (declaration.scope, declaration.unit),
                        (setup.scope(), setup.unit())
                    );
                }
                _ => panic!("original source order and concrete child kinds"),
            }
        }
        assert!(children.next().is_none());
        if index == 1 {
            let otherwise = conditional.else_input().unwrap();
            let element_node = node(file, branch.region.ops.first().unwrap());
            assert_eq!(otherwise.body_node(), Some(element_node));
            assert!(otherwise.accepts_body(element_node, element));
        }
    }
}

#[test]
fn four_exact_historical_prefix_suffix_sources_have_source_ordered_original_successors() {
    for (source, shapes) in [
        // Source SHA256 936e98abf0e50626b273965e41fb3b612644baa4db83579f19fdf75f6b9296a7.
        (
            "<script setup>let ready=true;let message='text'</script><template><p v-if='ready'>prefix{{message}}</p><b v-else>{{message}}</b></template>",
            ["TI", "I"],
        ),
        // Source SHA256 a63f53117f8fa0adc3ee51048317f7bc1294121fda8cd26820b99912af9338de.
        (
            "<script setup>let ready=true;let message='text'</script><template><p v-if='ready'>{{message}}</p><b v-else>prefix{{message}}</b></template>",
            ["I", "TI"],
        ),
        // Source SHA256 269a8e8be051f108d5a6f9f7b2bb9d2806e2ad7f200e104be8cf66095cdbfb7b.
        (
            "<script setup>let ready=true;let message='text'</script><template><p v-if='ready'>{{message}}suffix</p><b v-else>{{message}}</b></template>",
            ["IT", "I"],
        ),
        // Source SHA256 f54abb4d57cea1a4131b45416fcce82c0c065dfac039edc79fa04cfa4cdffa13.
        (
            "<script setup>let ready=true;let message='text'</script><template><p v-if='ready'>{{message}}</p><b v-else>{{message}}suffix</b></template>",
            ["I", "IT"],
        ),
    ] {
        let arena = Allocator::default();
        assert_original(&completed(&arena, source), shapes);
    }
}

#[test]
fn original_unicode_quotes_backslashes_and_const_reads_keep_whole_authored_part_spans() {
    let arena = Allocator::default();
    let source = "\r\n<script setup lang='ts'>let ready:boolean=true;let left:string='x';const right:string='y'</script>\r\n<template><p v-if='ready'>雪🌸\"\\!{{ left }}</p><b v-else>{{\r\nright\r\n}}戻🌸\"\\!</b></template>\r\n";
    let output = completed(&arena, source);
    assert_original(&output, ["TI", "IT"]);
    let file = output.file().unwrap();
    let [left, right] = file.native_interpolations() else {
        panic!("two original reads")
    };
    let (State::Admitted(a), State::Admitted(b)) = (left.state(), right.state()) else {
        panic!("normal original nodes")
    };
    let a = file.expression(a).unwrap();
    let b = file.expression(b).unwrap();
    let [a_read] = a.table().unwrap().occurrences() else {
        panic!("left")
    };
    let [b_read] = b.table().unwrap().occurrences() else {
        panic!("right")
    };
    assert_eq!(
        a.binding(a_read.binding)
            .unwrap()
            .declaration()
            .unwrap()
            .kind,
        crate::file::DeclarationKind::Let
    );
    assert_eq!(
        b.binding(b_read.binding)
            .unwrap()
            .declaration()
            .unwrap()
            .kind,
        crate::file::DeclarationKind::Const
    );
}

#[test]
fn same_binding_composites_keep_distinct_ast_nodes_and_reject_foreign_file_authority() {
    let arena = Allocator::default();
    let source = "<script setup>let ready=true</script><template><p v-if='ready'>prefix{{ready}}</p><b v-else>{{ready}}suffix</b></template>";
    let own = completed(&arena, source);
    let foreign = completed(&arena, source);
    assert_original(&own, ["TI", "IT"]);
    let file = own.file().unwrap();
    let other = foreign.file().unwrap();
    let [left, right] = file.native_interpolations() else {
        panic!("two operands")
    };
    let (State::Admitted(a), State::Admitted(b)) = (left.state(), right.state()) else {
        panic!("two nodes")
    };
    assert_ne!(a, b);
    let a_row = file.expression(a).unwrap();
    let b_row = file.expression(b).unwrap();
    let [a_read] = a_row.table().unwrap().occurrences() else {
        panic!("first")
    };
    let [b_read] = b_row.table().unwrap().occurrences() else {
        panic!("second")
    };
    assert_eq!(a_read.binding, b_read.binding);
    let [conditional] = file.native_conditionals() else {
        panic!("If")
    };
    assert_eq!(conditional.binding(), Some(a_read.binding));
    assert!(!core::ptr::eq(
        a_row.table().unwrap().expression().ast,
        b_row.table().unwrap().expression().ast
    ));
    let candidate = other.expression(a).unwrap();
    assert!(!a_row.accepts(candidate.binding(a_read.binding).unwrap()));
    assert!(!candidate.accepts(a_row.binding(a_read.binding).unwrap()));
    let foreign_element = foreign
        .selected()
        .children()
        .next()
        .unwrap()
        .into_element()
        .unwrap();
    let foreign_child = foreign_element.children().nth(1).unwrap();
    assert!(
        left.input()
            .operand()
            .admitted_for(foreign.selected(), foreign_child)
            .is_none()
    );
}

#[test]
fn old_singleton_profiles_and_single_if_composite_refusals_remain_separate() {
    for (source, shapes) in [
        (
            "<script setup>let ready=true</script><template><p v-if='ready'/><b v-else>{{ready}}</b></template>",
            ["", "I"],
        ),
        (
            "<script setup>let ready=true</script><template><p v-if='ready'>kept</p><b v-else>{{ready}}</b></template>",
            ["T", "I"],
        ),
        (
            "<script setup>let ready=true</script><template><p v-if='ready'>{{ready}}</p><b v-else/></template>",
            ["I", ""],
        ),
    ] {
        let arena = Allocator::default();
        assert_original(&completed(&arena, source), shapes);
    }
    for body in ["prefix{{message}}", "{{message}}suffix"] {
        let arena = Allocator::default();
        let source = alloc::format!(
            "<script setup>let ready=true;let message='text'</script><template><i v-if='ready'>{body}</i></template>"
        );
        let mut owner = owner(&arena, &source);
        {
            let mut walk = owner.begin().unwrap();
            let child = walk.selected().children().next().unwrap();
            assert_eq!(walk.child(child).unwrap_err().kind, Kind::UnsupportedChild);
            assert!(walk.complete().is_err());
        }
        let output = owner.finish();
        assert!(output.view().is_err() && output.setup().is_err());
    }
}
