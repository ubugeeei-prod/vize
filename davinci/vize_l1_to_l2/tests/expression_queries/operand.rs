//! Independent Croquis expression products and original File reference identities.
use super::{Model, observe};
use std::{hint::black_box, time::Instant};
use vize_atelier_core::{Parser, RootNode};
use vize_croquis::{Drawer, DrawerOptions, TemplateExpressionKind};
use vize_l0::{Allocator, Span};
use vize_l2::query::{Namespace, TemplateOperandKind as Kind};

#[derive(Debug, PartialEq, Eq)]
struct Row {
    span: Span,
    text: String,
    kind: Kind,
}

fn original(root: &RootNode<'_>, start: u32) -> Vec<Row> {
    let mut drawer = Drawer::with_options(DrawerOptions {
        collect_template_expressions: true,
        analyze_template_scopes: true,
        ..DrawerOptions::default()
    });
    drawer.draw_template(root);
    drawer
        .croquis()
        .template_expressions
        .iter()
        .filter_map(|expr| {
            let kind = match expr.kind {
                TemplateExpressionKind::Interpolation => Kind::Interpolation,
                TemplateExpressionKind::VBind => Kind::Bind,
                TemplateExpressionKind::VIf => Kind::Condition,
                TemplateExpressionKind::VModel => Kind::Model,
                _ => return None,
            };
            Some(Row {
                kind,
                text: expr.content.to_string(),
                span: Span::new(start + expr.start, start + expr.end),
            })
        })
        .collect()
}

fn output(model: Model<'_, '_>) -> Vec<Row> {
    let mut rows = Vec::new();
    model
        .for_each_resolved_template_operand(|op| {
            rows.push(Row {
                kind: op.kind(),
                text: op.text().to_string(),
                span: op.span(),
            });
        })
        .unwrap();
    rows
}

pub(super) fn compare(root: &RootNode<'_>, start: u32, model: Model<'_, '_>, path: &str) -> usize {
    let mut expected = original(root, start);
    let mut actual = output(model);
    let key = |r: &Row| (r.span.start, r.span.end);
    expected.sort_by_key(key);
    actual.sort_by_key(key);
    assert_eq!(actual, expected, "{path}");

    let mut occurrences = Vec::new();
    model
        .for_each_resolved_template_operand(|op| {
            let file = op.scope().file();
            let resolution = file.expression(op.node()).unwrap();
            assert!(std::ptr::eq(file, model.file()));
            assert_eq!(resolution.scope(), Some(op.scope().scope().id));
            let expression = resolution.table().unwrap().expression();
            assert_eq!((expression.span, expression.source), (op.span(), op.text()));
            op.for_each_reference(|r| {
                assert_eq!(r.node(), op.node());
                assert_eq!(r.scope(), op.scope().scope().id);
                assert!(op.span().start <= r.span().start && r.span().end <= op.span().end);
                let binding = op
                    .scope()
                    .lookup(r.occurrence().name, Namespace::Value)
                    .unwrap();
                assert!(binding.same_owner(r.binding()));
                assert_eq!(binding.id(), r.binding().id());
                occurrences.push(r);
            })
            .unwrap();
        })
        .unwrap();
    let mut expected_refs = Vec::new();
    model
        .for_each_expression_reference(|r| expected_refs.push(r))
        .unwrap();
    assert_eq!(occurrences.len(), expected_refs.len(), "{path}");
    for (row, expected) in occurrences.into_iter().zip(expected_refs) {
        super::equal(row, expected);
    }
    actual.len()
}

#[test]
fn reads_constants_entities_whitespace_and_repeated_uses() {
    for source in [
        "<script setup>const n=1;</script><template><p :title='0'>{{ n + n }}</p></template>",
        "<script setup>const fj=1;</script>\r\n<template><p :title='&fjlig;'>{{42}}</p></template>",
        "<template><div>plain</div></template>",
    ] {
        let mut counts = super::Counts::default();
        assert!(super::compare(source, source, &mut counts, true));
        assert_eq!(
            counts.operands,
            if source.contains(":title") { 2 } else { 0 }
        );
    }
}

#[test]
fn selected_conditions_and_mutable_models_use_the_original_operation_and_binding() {
    use vize_l1::markup::SelectedTemplateComponent;
    use vize_l2::{lang::js::NativeTemplateOwner, resolution::Usage};
    for (source, kind, usage) in [
        (
            "<script setup>let ready=true</script><template><div v-if='ready'>kept</div></template>",
            Kind::Condition,
            Usage::Read,
        ),
        (
            "\r\n<script setup lang='ts'>let café:bigint=9n</script>\r\n<template><input v-model=\"café\" /></template>\r\n",
            Kind::Model,
            Usage::ReadWrite,
        ),
    ] {
        let arena = Allocator::default();
        let observation = observe(&arena, source);
        let descriptor = observation.descriptor().admitted().unwrap();
        let selected = SelectedTemplateComponent::parse_in(&arena, descriptor)
            .unwrap()
            .unwrap();
        let mut owner = NativeTemplateOwner::new(selected)
            .unwrap_or_else(|_| panic!("original selected template"));
        owner.parse_setup_program().unwrap();
        let mut walk = owner.begin_setup().unwrap();
        for child in walk.selected().children() {
            walk.child(child).unwrap();
        }
        walk.complete().unwrap();
        let original = owner.finish();
        let file = original.file().unwrap();
        assert!(original.view().is_ok());
        let model = Model::new(file);
        let block = descriptor.template().unwrap().block();
        let (root, _) = Parser::new(&arena, block.source()).parse();
        assert_eq!(compare(&root, block.start(), model, source), 1);
        let mut refs = 0;
        model
            .for_each_resolved_template_operand(|op| {
                assert_eq!(op.kind(), kind);
                assert_eq!(op.span().slice(source), op.text());
                op.for_each_reference(|r| {
                    assert_eq!(r.occurrence().usage, usage);
                    refs += 1;
                })
                .unwrap();
            })
            .unwrap();
        assert_eq!(refs, 1);
    }
}
#[test]
#[ignore = "paired checker expression packets over retained inputs"]
fn paired_operand_products() {
    for repeats in [1, 8, 32, 128] {
        let arena = Allocator::default();
        let source = super::input::repeated(repeats);
        let retained = observe(&arena, &source);
        let native = retained.admitted().unwrap();
        let model = Model::new(native.file().file());
        let block = retained
            .descriptor()
            .admitted()
            .unwrap()
            .template()
            .unwrap()
            .block();
        let (root, _) = Parser::new(&arena, block.source()).parse();
        assert_eq!(compare(&root, block.start(), model, "profile"), 2 * repeats);
        let time = |joined| {
            let rounds = 512 / repeats;
            let start = Instant::now();
            for _ in 0..rounds {
                let rows = if joined {
                    output(black_box(model))
                } else {
                    original(black_box(&root), block.start())
                };
                drop(black_box(rows));
            }
            start.elapsed().as_nanos() / rounds as u128
        };
        let mut pairs = Vec::new();
        for i in 0..35 {
            let first = i % 2 == 0;
            let a = time(first);
            let b = time(!first);
            if i >= 4 {
                pairs.push(if first { (b, a) } else { (a, b) });
            }
        }
        eprintln!("template_operand repeats={repeats} ns={pairs:?}");
        let (mut old, mut new): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
        old.sort_unstable();
        new.sort_unstable();
        assert!(
            new[new.len() / 2] <= old[old.len() / 2],
            "complete packet for {repeats}"
        );
    }
}
