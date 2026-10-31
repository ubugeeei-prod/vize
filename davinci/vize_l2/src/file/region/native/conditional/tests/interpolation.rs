//! One real setup read shares its root conditional's original File scope.

use super::*;
use crate::expr::ExprRef;
use crate::file::{Namespace, NativeFileInterpolationState};
use crate::resolution::Usage;

#[test]
fn root_if_text_keeps_original_ast_single_read_current_scope_and_distinct_real_nodes() {
    for (source, name, full) in [
        // The exact prior static-only refusal has this explicit current successor.
        (
            "<script setup>let ready=true</script><template><div v-if='ready'>{{ready}}</div></template>",
            "ready",
            "{{ready}}",
        ),
        (
            "<script setup>let ready=false;let message='before'</script><template><p v-if='ready'>{{ message }}</p></template>",
            "message",
            "{{ message }}",
        ),
        (
            "<script setup>const ready=true;const message=null</script><template><i v-if='ready'>{{message}}</i></template>",
            "message",
            "{{message}}",
        ),
        (
            "\r\n<script setup lang='ts'>var 可視:boolean=true;let 文:string='雪🌸'</script>\r\n<template><b v-if='可視'>{{文}}</b></template>\r\n",
            "文",
            "{{文}}",
        ),
    ] {
        let arena = Allocator::default();
        let output = completed(&arena, source);
        assert!(output.view().is_ok());
        let file = output.file().unwrap();
        assert!(file.is_complete());
        let [unit] = file.units() else {
            panic!("one setup");
        };
        assert_eq!(file.scopes().len(), 2);
        assert_eq!(file.artifact().node_count(), 3);
        assert!(file.template_declarations().next().is_none());
        let [conditional] = file.native_conditionals() else {
            panic!("original If input");
        };
        let NativeFileConditionalState::Admitted(if_node) = conditional.state() else {
            panic!("closed If");
        };
        let [Op::If(original)] = file.artifact().root().ops.as_slice() else {
            panic!("actual If");
        };
        assert!(conditional.accepts(original));
        let [branch] = original.branches.as_slice() else {
            panic!("one branch");
        };
        let Some(ExprRef::Js(condition)) = branch.condition else {
            panic!("condition AST");
        };
        let [Op::Element(element)] = branch.region.ops.as_slice() else {
            panic!("one Element");
        };
        let [Op::Interpolation(actual)] = element.children.ops.as_slice() else {
            panic!("one interpolation");
        };
        let ExprRef::Js(expression) = actual.expression else {
            panic!("body AST");
        };
        let [record] = file.native_interpolations() else {
            panic!("whole original input");
        };
        let NativeFileInterpolationState::Admitted(node) = record.state() else {
            panic!("actual mint");
        };
        assert_ne!(node, if_node);
        assert!(core::ptr::eq(
            record,
            file.native_interpolation(node).unwrap()
        ));
        assert!(!core::ptr::eq(condition.ast, expression.ast));
        assert_eq!(actual.span, record.input().operand().full_span());
        assert_eq!(actual.span.slice(source), full);
        let syntax = record.input().operand().syntax();
        assert!(core::ptr::eq(syntax.expression().unwrap(), expression.ast));
        assert!(core::ptr::eq(syntax.source().text(), expression.source));
        assert!(core::ptr::eq(syntax.source().authored_root(), source));
        assert_eq!(syntax.comments().count(), 0);
        assert!(syntax.source().decode_map().is_none());
        let resolution = file.expression(node).unwrap();
        assert_eq!(resolution.scope(), Some(unit.scope));
        assert_eq!(conditional.scope(), unit.scope);
        let table = resolution.table().unwrap();
        assert!(core::ptr::eq(table.expression().ast, expression.ast));
        assert!(core::ptr::eq(table.expression().source, expression.source));
        assert!(core::ptr::eq(
            table.expression().coordinates.unwrap(),
            expression.coordinates.unwrap()
        ));
        let [read] = table.occurrences() else {
            panic!("one actual read");
        };
        assert_eq!((read.name, read.usage), (name, Usage::Read));
        assert_eq!(
            expression.authored_span(read.span),
            Some(syntax.source().span())
        );
        assert_eq!(syntax.source().span().slice(source), name);
        let binding = resolution.binding(read.binding).unwrap();
        assert!(resolution.accepts(binding));
        assert!(binding.same_owner(file.lookup(unit.scope, name, Namespace::Value).unwrap()));
        let declaration = binding.declaration().unwrap();
        assert_eq!((declaration.scope, declaration.unit), (unit.scope, unit.id));
        assert_eq!(declaration.span.slice(source), name);
        assert!(declaration.is_direct_program() && declaration.initializer.is_primitive());
        if name == "ready" {
            assert_eq!(conditional.binding(), Some(read.binding));
            assert_ne!(condition.span, expression.span);
        } else {
            assert_ne!(conditional.binding(), Some(read.binding));
        }
    }
}

#[test]
fn equal_if_text_files_and_ids_never_exchange_original_binding_or_ast_ownership() {
    let arena = Allocator::default();
    let source = "<script setup>let ready=true;let message=2</script><template><i v-if='ready'>{{message}}</i></template>";
    let output = completed(&arena, source);
    let foreign = completed(&arena, source);
    let file = output.file().unwrap();
    let other = foreign.file().unwrap();
    let [record] = file.native_interpolations() else {
        panic!("own original");
    };
    let [foreign_record] = other.native_interpolations() else {
        panic!("foreign original");
    };
    assert_eq!(record.state(), foreign_record.state());
    let NativeFileInterpolationState::Admitted(node) = record.state() else {
        panic!("real node");
    };
    let own = file.expression(node).unwrap();
    let candidate = other.expression(node).unwrap();
    let [read] = own.table().unwrap().occurrences() else {
        panic!("own read");
    };
    let binding = own.binding(read.binding).unwrap();
    assert!(!own.accepts(candidate.binding(read.binding).unwrap()));
    assert!(!candidate.accepts(binding));
    assert!(!core::ptr::eq(
        own.table().unwrap().expression().ast,
        candidate.table().unwrap().expression().ast
    ));
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

#[test]
fn non_bare_or_unowned_if_text_is_refused_with_whole_original_input_custody() {
    for body in [
        "{{1}}",
        "{{true}}",
        "{{null}}",
        "{{'text'}}",
        "{{message+message}}",
        "{{message()}}",
        "{{message.length}}",
        "{{[message]}}",
        "{{({message})}}",
        "{{missing}}",
        "{{\\u006dessage}}",
        "{{m&#101;ssage}}",
        "{{message /* kept */}}",
        "{{message as string}}",
        "<p>{{message}}</p>",
        "prefix{{message}}",
        "{{message}}suffix",
        "{{message}}{{message}}",
        "{{}}",
        "{{message +}}",
    ] {
        let arena = Allocator::default();
        let source = alloc::format!(
            "<script setup>let ready=true;let message='text'</script><template><i v-if='ready'>{body}</i></template>"
        );
        let mut owner = owner(&arena, &source);
        {
            let mut walk = owner.begin().unwrap();
            let child = walk.selected().children().next().unwrap();
            assert!(walk.child(child).is_err(), "{body}");
            assert!(walk.complete().is_err());
        }
        let output = owner.finish();
        assert!(output.view().is_err(), "{body}");
        let rejected = output.rejected_file().unwrap();
        assert!(core::ptr::eq(rejected.source(), source.as_str()));
        let [conditional] = rejected.native_conditionals() else {
            panic!("retained If input");
        };
        assert!(matches!(
            conditional.state(),
            NativeFileConditionalState::Refused(_)
        ));
        assert!(conditional.original.is_none());
        assert_eq!(rejected.scopes().len(), 2);
        for row in rejected.native_interpolations() {
            assert!(core::ptr::eq(
                row.input().operand().syntax().source().authored_root(),
                source.as_str()
            ));
            assert!(
                row.input()
                    .operand()
                    .full_span()
                    .slice(&source)
                    .starts_with("{{")
            );
        }
    }
}

#[test]
fn late_if_text_suffix_refusal_keeps_real_prefix_read_and_denies_whole_if_completion() {
    let arena = Allocator::default();
    let source = "<script setup>let ready=true;let message='text'</script><template><i v-if='ready'>{{message}}suffix</i></template>";
    let mut owner = owner(&arena, source);
    {
        let mut walk = owner.begin().unwrap();
        let child = walk.selected().children().next().unwrap();
        assert_eq!(walk.child(child).unwrap_err().kind, Kind::UnsupportedChild);
        assert!(walk.complete().is_err());
    }
    let output = owner.finish();
    assert!(output.view().is_err());
    let rejected = output.rejected_file().unwrap();
    let [conditional] = rejected.native_conditionals() else {
        panic!("whole If source");
    };
    assert_eq!(
        conditional.state(),
        NativeFileConditionalState::Refused(Kind::UnsupportedChild)
    );
    assert!(conditional.original.is_none());
    let [record] = rejected.native_interpolations() else {
        panic!("completed real prefix");
    };
    assert!(matches!(
        record.state(),
        NativeFileInterpolationState::Admitted(_)
    ));
    let [Op::If(original)] = rejected.artifact().parts.root.ops.as_slice() else {
        panic!("incomplete If");
    };
    assert!(!conditional.accepts(original));
    let [branch] = original.branches.as_slice() else {
        panic!("original branch");
    };
    let [Op::Element(element)] = branch.region.ops.as_slice() else {
        panic!("retained body");
    };
    let [Op::Interpolation(actual)] = element.children.ops.as_slice() else {
        panic!("only actual prefix");
    };
    let ExprRef::Js(expression) = actual.expression else {
        panic!("actual AST");
    };
    assert!(core::ptr::eq(
        expression.ast,
        record.input().operand().syntax().expression().unwrap()
    ));
    assert_eq!(actual.span.slice(source), "{{message}}");
}

#[test]
fn narrower_body_refusal_preserves_complete_existing_resolution_and_original_syntax() {
    for (body, reads, comments) in [
        ("{{42}}", 0, 0),
        ("{{message+message}}", 2, 0),
        ("{{message /* kept */}}", 1, 1),
        ("{{m&#101;ssage}}", 1, 0),
    ] {
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
        assert!(output.view().is_err());
        let rejected = output.rejected_file().unwrap();
        let [record] = rejected.native_interpolations() else {
            panic!("original whole body input");
        };
        assert_eq!(
            record.state(),
            NativeFileInterpolationState::Refused(Kind::UnsupportedChild)
        );
        assert_eq!(record.input().operand().full_span().slice(&source), body);
        assert_eq!(
            record.input().operand().syntax().comments().count(),
            comments
        );
        let mut joins = rejected.facts.native_interpolation_nodes.iter();
        let (node, index) = joins.next().unwrap();
        assert_eq!(*index, 0);
        assert!(joins.next().is_none());
        let resolution = rejected.facts.expressions.get(node).unwrap();
        assert_eq!(resolution.table.occurrences().len(), reads);
        assert!(core::ptr::eq(
            resolution.table.expression().ast,
            record.input().operand().syntax().expression().unwrap()
        ));
        let [conditional] = rejected.native_conditionals() else {
            panic!("incomplete If");
        };
        assert!(conditional.original.is_none());
        assert_eq!(conditional.scope(), resolution.scope);
    }
}

#[test]
fn if_text_body_unwind_retains_both_real_inputs_without_completing_the_if() {
    let arena = Allocator::default();
    let source =
        "<script setup>let ready=true</script><template><i v-if='ready'>{{ready}}</i></template>";
    let mut owner = owner(&arena, source);
    {
        let mut walk = owner.begin().unwrap();
        let child = walk.selected().children().next().unwrap();
        let scope = walk.root.scope;
        FAULT.with(|fault| fault.set(4));
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| walk.child(child))).is_err()
        );
        assert_eq!(walk.root.scope, scope);
        assert_eq!(walk.root.facts.scopes.len(), 2);
        let [record] = walk.root.facts.native_interpolations.as_slice() else {
            panic!("real read");
        };
        assert!(matches!(
            record.state(),
            NativeFileInterpolationState::Admitted(_)
        ));
        assert_eq!(walk.complete().unwrap_err().kind, Kind::Interrupted);
    }
    let output = owner.finish();
    assert!(output.view().is_err());
    let rejected = output.rejected_file().unwrap();
    assert!(rejected.template_interruption().is_some());
    let [conditional] = rejected.native_conditionals() else {
        panic!("parked If");
    };
    assert_eq!(conditional.state(), NativeFileConditionalState::Pending);
    assert!(conditional.original.is_none());
    assert_eq!(rejected.native_interpolations().len(), 1);
    assert!(output.retained_setup().is_some());
}
