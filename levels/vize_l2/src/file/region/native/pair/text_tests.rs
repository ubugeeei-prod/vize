//! Explicit successors of two original whole-source lower refusals.
//!
//! Predecessor 69f0ec08fec7302dd619cd5629194f96a49d6f16 pair/tests.rs:
//! blob 88c13a550551258e704f32b5312478aa581af0a4,
//! SHA256 efbad814c4b683024bb16955d507f597fc69683bbeea731e8fd3333ed9f64178.
//! Its full oracle was UnsupportedChild, cursor zero, failed complete/view,
//! Refused conditional, no original If pointer, and retained original source.

extern crate std;

use super::tests::{completed, owner};
use super::*;
use crate::expr::ExprRef;
use crate::file::{Namespace as BindingNamespace, NativeFileInterpolationState as State};
use crate::lang::js::NativeTemplateFile;
use crate::op::Op;
use crate::resolution::Usage;
use crate::walk::NodeRef;
use vize_l0::Allocator;

mod refusal;

const FIRST_SUCCESSOR: &str = "<script setup>let ready=true</script><template><p v-if='ready'>{{ready}}</p><b v-else/></template>";
const ELSE_SUCCESSOR: &str = "<script setup>let ready=true</script><template><p v-if='ready'/><b v-else>{{ready}}</b></template>";

fn assert_reads(output: &NativeTemplateFile<'_>, names: &[&str]) {
    let setup = output.setup().unwrap();
    let file = output.file().unwrap();
    let source = file.artifact().source();
    assert!(output.view().is_ok() && file.is_complete());
    assert_eq!(file.scopes().len(), 2);
    assert!(file.template_declarations().next().is_none());
    assert_eq!(file.native_interpolations().len(), names.len());
    let [conditional] = file.native_conditionals() else {
        panic!("one original conditional");
    };
    let NativeFileConditionalState::Admitted(if_node) = conditional.state() else {
        panic!("whole pair closed");
    };
    let [Op::If(original)] = file.artifact().root().ops.as_slice() else {
        panic!("actual pair");
    };
    assert!(conditional.accepts(original));
    let [first, last] = original.branches.as_slice() else {
        panic!("two actual branches");
    };
    let Some(ExprRef::Js(condition)) = first.condition else {
        panic!("actual condition");
    };
    let otherwise = conditional.else_input().unwrap();
    assert!(last.condition.is_none());
    assert_eq!(first.span.end, last.span.start);
    assert_eq!(otherwise.operand().element_span(), last.span);
    let mut index = 0;
    for branch in [first, last] {
        let [Op::Element(element)] = branch.region.ops.as_slice() else {
            panic!("one real direct element");
        };
        assert!(element.attributes.is_empty() && element.bindings.is_empty());
        let [Op::Interpolation(actual)] = element.children.ops.as_slice() else {
            assert!(element.children.ops.is_empty());
            continue;
        };
        let record = file.native_interpolations().get(index).unwrap();
        let name = *names.get(index).unwrap();
        index += 1;
        let State::Admitted(node) = record.state() else {
            panic!("normally minted original interpolation");
        };
        assert_ne!(node, if_node);
        assert!(core::ptr::eq(
            record,
            file.native_interpolation(node).unwrap()
        ));
        let mut actual_node = None;
        file.artifact()
            .visit_nodes(&mut |id, current| {
                if let NodeRef::Op(Op::Interpolation(candidate)) = current
                    && core::ptr::eq(candidate.as_ref(), actual.as_ref())
                {
                    actual_node = Some(id);
                }
            })
            .unwrap();
        assert_eq!(actual_node, Some(node));
        let ExprRef::Js(expression) = actual.expression else {
            panic!("original body expression");
        };
        assert!(!core::ptr::eq(condition.ast, expression.ast));
        let syntax = record.input().operand().syntax();
        assert!(core::ptr::eq(syntax.expression().unwrap(), expression.ast));
        assert!(core::ptr::eq(syntax.source().text(), expression.source));
        assert!(core::ptr::eq(syntax.source().authored_root(), source));
        assert_eq!(actual.span, record.input().operand().full_span());
        assert_eq!(syntax.source().span().slice(source), name);
        let resolution = file.expression(node).unwrap();
        assert_eq!(resolution.scope(), Some(setup.scope()));
        assert_eq!(conditional.scope(), setup.scope());
        let table = resolution.table().unwrap();
        assert!(core::ptr::eq(table.expression().ast, expression.ast));
        assert!(core::ptr::eq(table.expression().source, expression.source));
        assert!(core::ptr::eq(
            table.expression().coordinates.unwrap(),
            expression.coordinates.unwrap()
        ));
        let [read] = table.occurrences() else {
            panic!("one authentic read per original operand");
        };
        assert_eq!((read.name, read.usage), (name, Usage::Read));
        assert_eq!(
            expression.authored_span(read.span),
            Some(syntax.source().span())
        );
        let binding = resolution.binding(read.binding).unwrap();
        assert!(resolution.accepts(binding) && setup.binding(binding).is_ok());
        assert!(
            binding.same_owner(
                file.lookup(setup.scope(), name, BindingNamespace::Value)
                    .unwrap()
            )
        );
        let declaration = binding.declaration().unwrap();
        assert!(declaration.is_direct_program() && declaration.initializer.is_primitive());
        assert_eq!(
            (declaration.scope, declaration.unit),
            (setup.scope(), setup.unit())
        );
        assert_eq!(declaration.span.slice(source), name);
    }
    assert_eq!(index, names.len());
}

#[test]
fn exact_two_historical_paired_text_inputs_have_explicit_original_owner_successors() {
    // Full source SHA256s, without inventing a new source or changing history:
    // d3da5c41da333984ff7b2d205667d71b8d05ecf6b699d7774cbbb466636359b2
    // 3df55f9463e1ca613c46d74ce839219af5c4c8b1c46468fc2d184afdd2b46b62
    for source in [FIRST_SUCCESSOR, ELSE_SUCCESSOR] {
        let arena = Allocator::default();
        assert_reads(&completed(&arena, source), &["ready"]);
    }
}

#[test]
fn both_branches_resolve_distinct_original_nodes_and_asts_in_the_same_setup_scope() {
    for (source, names, same_binding) in [
        (
            "<script setup>let ready=true;let left='before';var right=2</script><template><p v-if='ready'>{{left}}</p><b v-else>{{ right }}</b></template>",
            ["left", "right"],
            false,
        ),
        (
            "<script setup>let ready=true</script><template><p v-if='ready'>{{ready}}</p><b v-else>{{ready}}</b></template>",
            ["ready", "ready"],
            true,
        ),
        (
            "\r\n<script setup lang='ts'>let 可視:boolean=true;const 文:string='雪🌸';const 数:number=2</script>\r\n<template><p v-if='可視'>{{文}}</p><b v-else>{{数}}</b></template>\r\n",
            ["文", "数"],
            false,
        ),
        (
            "<script setup>const ready=false;const left=null;const right=false</script><template><p v-if='ready'>{{left}}</p><b v-else>{{right}}</b></template>",
            ["left", "right"],
            false,
        ),
    ] {
        let arena = Allocator::default();
        let output = completed(&arena, source);
        assert_reads(&output, &names);
        let file = output.file().unwrap();
        let [left, right] = file.native_interpolations() else {
            panic!("two distinct original receivers");
        };
        let (State::Admitted(a), State::Admitted(b)) = (left.state(), right.state()) else {
            panic!("two actual minted nodes");
        };
        assert_ne!(a, b);
        let a = file.expression(a).unwrap();
        let b = file.expression(b).unwrap();
        let a = a.table().unwrap();
        let b = b.table().unwrap();
        assert!(!core::ptr::eq(a.expression().ast, b.expression().ast));
        assert_ne!(a.expression().span, b.expression().span);
        let [a_read] = a.occurrences() else {
            panic!("left read")
        };
        let [b_read] = b.occurrences() else {
            panic!("right read")
        };
        assert_eq!(a_read.binding == b_read.binding, same_binding);
        assert_ne!(
            a.expression().authored_span(a_read.span),
            b.expression().authored_span(b_read.span)
        );
        if same_binding {
            let [conditional] = file.native_conditionals() else {
                panic!("condition")
            };
            assert_eq!(conditional.binding(), Some(a_read.binding));
        }
    }
}

#[test]
fn identical_pair_files_never_exchange_binding_or_original_expression_authority() {
    let arena = Allocator::default();
    let source = "<script setup>let ready=true</script><template><p v-if='ready'>{{ready}}</p><b v-else>{{ready}}</b></template>";
    let own = completed(&arena, source);
    let foreign = completed(&arena, source);
    let file = own.file().unwrap();
    let other = foreign.file().unwrap();
    assert_eq!(file.native_interpolations().len(), 2);
    assert_eq!(other.native_interpolations().len(), 2);
    for (record, foreign_record) in file
        .native_interpolations()
        .iter()
        .zip(other.native_interpolations())
    {
        assert_eq!(record.state(), foreign_record.state());
        let State::Admitted(node) = record.state() else {
            panic!("original node")
        };
        let resolution = file.expression(node).unwrap();
        let candidate = other.expression(node).unwrap();
        let [read] = resolution.table().unwrap().occurrences() else {
            panic!("own read")
        };
        let own_binding = resolution.binding(read.binding).unwrap();
        let foreign_binding = candidate.binding(read.binding).unwrap();
        assert!(!resolution.accepts(foreign_binding));
        assert!(!candidate.accepts(own_binding));
        assert!(!core::ptr::eq(
            record.input().operand().syntax().expression().unwrap(),
            foreign_record
                .input()
                .operand()
                .syntax()
                .expression()
                .unwrap()
        ));
    }
}

#[test]
fn single_if_and_paired_first_body_keep_the_same_original_resolution_semantics() {
    let arena = Allocator::default();
    let single = completed(
        &arena,
        "<script setup>let ready=true;let message='before'</script><template><p v-if='ready'>{{message}}</p></template>",
    );
    let paired = completed(
        &arena,
        "<script setup>let ready=true;let message='before'</script><template><p v-if='ready'>{{message}}</p><b v-else/></template>",
    );
    let a = single.file().unwrap();
    let b = paired.file().unwrap();
    assert!(single.setup().is_ok() && paired.setup().is_ok());
    let [a_record] = a.native_interpolations() else {
        panic!("single read")
    };
    let [b_record] = b.native_interpolations() else {
        panic!("paired read")
    };
    assert_eq!(a_record.state(), b_record.state());
    assert_eq!(
        a_record.input().operand().full_span(),
        b_record.input().operand().full_span()
    );
    let State::Admitted(node) = a_record.state() else {
        panic!("normal mint")
    };
    let a_resolution = a.expression(node).unwrap();
    let b_resolution = b.expression(node).unwrap();
    assert_eq!(a_resolution.scope(), b_resolution.scope());
    let [a_read] = a_resolution.table().unwrap().occurrences() else {
        panic!("single read")
    };
    let [b_read] = b_resolution.table().unwrap().occurrences() else {
        panic!("paired read")
    };
    assert_eq!(
        (a_read.name, a_read.usage, a_read.binding, a_read.span),
        (b_read.name, b_read.usage, b_read.binding, b_read.span)
    );
    assert!(!a_resolution.accepts(b_resolution.binding(b_read.binding).unwrap()));
}
