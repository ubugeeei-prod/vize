extern crate std;

use super::super::hooks;
use super::*;
use crate::SurfaceParseOptions;
use crate::container::Vue;
use crate::embed::syntax::EmbedHole;
use crate::{SurfaceChild, check_fidelity};
use oxc_ast::ast::Expression;
use std::panic::{AssertUnwindSafe, catch_unwind};
use vize_l0::config::{VueDialect, VueVersion};
use vize_l0::{Allocator, String};

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V2,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

#[test]
fn whole_original_envelope_body_and_callback_positions_survive_consumption() {
    let arena = Allocator::default();
    let source = "雪🦀\r\n<!--head-->\n<template lang='html'><i>literal{{a&#43;1 | wrap(雪, &#x31;)}}{{b+}}</i></TeMpLaTe >尾";
    let owner = Vue.observe_vue2_descriptor(&arena, source, options());
    let selected = owner.selected().unwrap();
    let frame = (
        selected.container_index(),
        selected.opening_name(),
        selected.closing_name(),
    );
    let block = selected.block();
    let container = vize_l0::cstr!("{:?}", owner.container());
    let blocks = owner.container().blocks.as_ptr();
    let issues = owner.issues().as_ptr();
    let component = selected.component();
    let tree = vize_l0::cstr!("{:?}", component.tree());
    let children = component.tree().children.as_ptr();
    let errors = vize_l0::cstr!("{:?}", component.errors());
    let error_storage = component.errors().as_ptr();
    let unsupported = vize_l0::cstr!("{:?}", component.unsupported());
    let original_bindings: alloc::vec::Vec<_> = component
        .bindings()
        .iter()
        .map(|binding| {
            (
                binding.span(),
                binding.raw_content().as_ptr(),
                binding.raw_content(),
                binding.source().text().as_ptr(),
                binding.source().span(),
            )
        })
        .collect();
    let pool = owner.into_expression_pool();
    assert!(core::ptr::eq(pool.source(), source));
    assert!(core::ptr::eq(pool.root().unwrap().source(), source));
    assert_eq!(pool.options(), options());
    assert_eq!(vize_l0::cstr!("{:?}", pool.container()), container);
    assert_eq!(pool.container().blocks.as_ptr(), blocks);
    assert_eq!(pool.issues().as_ptr(), issues);
    let selected = pool.selected().unwrap();
    assert!(core::ptr::eq(selected.observation(), &pool));
    assert!(core::ptr::eq(
        selected.component(),
        pool.component().unwrap()
    ));
    assert_eq!(
        (
            selected.container_index(),
            selected.opening_name(),
            selected.closing_name()
        ),
        frame
    );
    assert_eq!(selected.block().span(), block.span());
    assert!(core::ptr::eq(selected.block().source(), block.source()));
    assert!(core::ptr::eq(
        selected.block().root_source(),
        block.root_source()
    ));
    let component = selected.component();
    assert_eq!(vize_l0::cstr!("{:?}", component.tree()), tree);
    assert_eq!(component.tree().children.as_ptr(), children);
    assert_eq!(check_fidelity(component.tree()), Ok(()));
    assert_eq!(vize_l0::cstr!("{:?}", component.errors()), errors);
    assert_eq!(component.errors().as_ptr(), error_storage);
    assert_eq!(vize_l0::cstr!("{:?}", component.unsupported()), unsupported);
    let moved_bindings: alloc::vec::Vec<_> = component
        .bindings()
        .iter()
        .map(|binding| {
            (
                binding.span(),
                binding.raw_content().as_ptr(),
                binding.raw_content(),
                binding.source().text().as_ptr(),
                binding.source().span(),
            )
        })
        .collect();
    assert_eq!(moved_bindings, original_bindings);
    let [admitted, rejected] = component.bindings() else {
        panic!("all original bindings");
    };
    assert!(admitted.admitted().is_some());
    assert_eq!(
        rejected.chain().unwrap().base().hole(),
        Some(EmbedHole::Syntax)
    );
    assert!(rejected.admitted().is_none());
    // Selected remains envelope-only, despite this actual original body hole.
    assert!(pool.selected().is_ok());
}

#[test]
fn refused_descriptor_preserves_every_block_option_and_original_issue() {
    let arena = Allocator::default();
    for (source, options) in [
        (
            "<template>{{value}}</template><script>let x=1</script>",
            options(),
        ),
        (
            "<template>{{value}}</template><style>.a{}</style>",
            options(),
        ),
        (
            "<template>{{value}}</template>",
            DescriptorOptions {
                version: VueVersion::V3,
                ..options()
            },
        ),
    ] {
        let owner = Vue.observe_vue2_descriptor(&arena, source, options);
        let refusal = owner.selected().unwrap_err();
        let issues = vize_l0::cstr!("{:?}", refusal.issues());
        let errors = vize_l0::cstr!("{:?}", refusal.errors());
        let container = vize_l0::cstr!("{:?}", owner.container());
        assert!(owner.component().is_none());
        let pool = owner.into_expression_pool();
        assert_eq!(pool.options(), options);
        assert!(core::ptr::eq(pool.source(), source));
        assert_eq!(vize_l0::cstr!("{:?}", pool.container()), container);
        let refusal = pool.selected().unwrap_err();
        assert_eq!(vize_l0::cstr!("{:?}", refusal.issues()), issues);
        assert_eq!(vize_l0::cstr!("{:?}", refusal.errors()), errors);
        assert!(pool.component().is_none());
    }
}

#[test]
fn repeated_callbacks_and_equal_bytes_keep_their_actual_distinct_original_roots() {
    let arena = Allocator::default();
    let other_arena = Allocator::default();
    let source = String::from("<template><i>{{value}}{{value}}</i><b>{{value}}</b></template>");
    let copy = String::from(source.as_str());
    let pool = Vue
        .observe_vue2_descriptor(&arena, &source, options())
        .into_expression_pool();
    let other = Vue
        .observe_vue2_descriptor(&other_arena, &copy, options())
        .into_expression_pool();
    let component = pool.selected().unwrap().component();
    let [first, second] = &*component.tree().children else {
        panic!("original direct parents");
    };
    let SurfaceChild::Element(first) = first else {
        panic!("original first element");
    };
    let SurfaceChild::Element(second) = second else {
        panic!("original second element");
    };
    assert_eq!((first.children.len(), second.children.len()), (2, 1));
    let [a, b, c] = component.bindings() else {
        panic!("three actual occurrences");
    };
    let roots = [a, b, c].map(|binding| binding.admitted().unwrap().base().expression().unwrap());
    assert!(!core::ptr::eq(
        *roots.first().unwrap(),
        *roots.get(1).unwrap()
    ));
    assert_ne!(a.span(), b.span());
    assert_ne!(b.span(), c.span());
    for (actual, foreign) in component
        .bindings()
        .iter()
        .zip(other.component().unwrap().bindings())
    {
        assert_eq!(actual.raw_content(), foreign.raw_content());
        assert_ne!(
            actual.source().authored_root().as_ptr(),
            foreign.source().authored_root().as_ptr()
        );
        assert!(!core::ptr::eq(
            actual.admitted().unwrap().base().expression().unwrap(),
            foreign.admitted().unwrap().base().expression().unwrap()
        ));
    }
}

#[test]
fn true_arena_roots_survive_pool_movement_normal_drop_and_caught_unwind() {
    let arena = Allocator::default();
    for unwind in [false, true] {
        let measured = hooks::Armed::new(hooks::Fault::None);
        let pool = Vue
            .observe_vue2_descriptor(&arena, "<template>{{a&#43;1}}{{b+}}</template>", options())
            .into_expression_pool();
        let syntax = pool
            .component()
            .unwrap()
            .bindings()
            .first()
            .unwrap()
            .admitted()
            .unwrap()
            .base();
        let root = syntax.expression().unwrap();
        assert_eq!(
            (
                measured.counts().splitters,
                measured.counts().components,
                measured.counts().dropped
            ),
            (1, 1, 0)
        );
        let mut owners = alloc::vec::Vec::from([pool]);
        owners.reserve(64);
        let moved = owners.first().unwrap();
        let moved_root = moved
            .component()
            .unwrap()
            .bindings()
            .first()
            .unwrap()
            .admitted()
            .unwrap()
            .base()
            .expression()
            .unwrap();
        assert!(core::ptr::eq(root, moved_root));
        if unwind {
            assert!(
                catch_unwind(AssertUnwindSafe(move || {
                    let _owners = owners;
                    panic!("after complete original pool custody");
                }))
                .is_err()
            );
        } else {
            drop(owners);
        }
        assert_eq!(
            (
                measured.counts().splitters,
                measured.counts().components,
                measured.counts().dropped
            ),
            (1, 1, 1)
        );
        let Expression::BinaryExpression(binary) = root else {
            panic!("genuine retained root");
        };
        let Expression::Identifier(left) = &binary.left else {
            panic!("original left");
        };
        assert_eq!(left.name.as_str(), "a");
        let next = Vue
            .observe_vue2_descriptor(&arena, "<template>{{next}}</template>", options())
            .into_expression_pool();
        assert_eq!(
            next.component()
                .unwrap()
                .bindings()
                .first()
                .unwrap()
                .admitted()
                .unwrap()
                .base()
                .source()
                .text(),
            "next"
        );
        drop(next);
        assert_eq!(
            (
                measured.counts().splitters,
                measured.counts().components,
                measured.counts().dropped
            ),
            (2, 2, 2)
        );
    }
}

#[test]
fn empty_and_literal_bodies_keep_their_complete_original_surface() {
    let arena = Allocator::default();
    for source in [
        "<template></template>",
        "<template>雪🦀 fixed<!--kept--></template>",
    ] {
        let owner = Vue.observe_vue2_descriptor(&arena, source, options());
        let original = vize_l0::cstr!("{:?}", owner.component().unwrap().tree());
        let pool = owner.into_expression_pool();
        let component = pool.selected().unwrap().component();
        assert_eq!(vize_l0::cstr!("{:?}", component.tree()), original);
        assert_eq!(check_fidelity(component.tree()), Ok(()));
        assert!(component.bindings().is_empty());
    }
}
