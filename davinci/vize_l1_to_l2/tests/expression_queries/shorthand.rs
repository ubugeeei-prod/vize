//! Same-name binds share the authored argument and retain its expression owner.
use super::{component_props, observe};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
    line_index::LineBreaks,
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::{
    native::NativeHoleKind,
    native_file::{NativeSfcObservation, lower_sfc_with_authored_bind_shorthand},
};
use vize_l2::query::Model;

fn shorthand<'a>(
    arena: &'a Allocator,
    source: &'a str,
    version: VueVersion,
) -> NativeSfcObservation<'a> {
    lower_sfc_with_authored_bind_shorthand(
        arena,
        source,
        DescriptorOptions {
            version,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    )
}

#[test]
fn authored_argument_is_the_original_operand_use_and_editor_position() {
    let source = "<script setup>const title=1;const 作者=2;</script>\r\n<template>😀<div :title v-bind:作者 :value='title' /></template>";
    let arena = Allocator::default();
    let old = observe(&arena, source);
    let holes: Vec<_> = old
        .template()
        .unwrap()
        .holes()
        .iter()
        .map(|hole| (hole.kind, hole.span.slice(source)))
        .collect();
    assert_eq!(
        holes,
        [
            (NativeHoleKind::Directive, ":title"),
            (NativeHoleKind::Directive, "v-bind:作者")
        ]
    );
    assert!(old.admitted().is_none());
    let new = shorthand(&arena, source, VueVersion::V3);
    let native = new.admitted().unwrap();
    let embeds = new.template().unwrap().embeds();
    let file = native.file().file();
    let mut actual = Vec::new();
    native
        .for_each_authored_template_operand(|op| {
            let embed = &embeds[actual.len()];
            let Some(oxc_ast::ast::Expression::Identifier(identifier)) = embed.syntax.expression()
            else {
                panic!("expected an identifier in the original parser tree")
            };
            assert_eq!(identifier.name.as_str(), op.text());
            assert!(std::ptr::eq(op.text(), op.span().slice(source)));
            assert_eq!(Some(op.node()), embed.node);
            let position = LineBreaks::Lsp.offset_to_position(source, op.span().start as usize);
            let mut uses = 0;
            op.for_each_reference(|usage| {
                uses += 1;
                let hit = Model::new(file)
                    .expression_reference_at_offset(op.span().start)
                    .unwrap()
                    .unwrap();
                super::equal(usage, hit);
                assert_eq!(usage.span(), op.span());
                let original = native
                    .reference_at_offset(op.span().start)
                    .unwrap()
                    .unwrap();
                assert!(std::ptr::eq(
                    original.template().unwrap(),
                    usage.occurrence()
                ));
                assert_eq!(
                    original.binding().declaration().unwrap().span.slice(source),
                    op.text()
                );
            })
            .unwrap();
            assert_eq!(uses, 1);
            actual.push((op.text(), position));
        })
        .unwrap();
    assert_eq!(
        actual,
        [("title", (1, 18)), ("作者", (1, 31)), ("title", (1, 42))]
    );
}

#[test]
fn component_diagnostics_and_imports_read_the_same_bound_value() {
    let source = "<script setup>import Card from './Card.vue';const title='a',other=3;</script><template><Card :title flag v-bind:other /></template>";
    let arena = Allocator::default();
    assert!(observe(&arena, source).setup_component_imports().is_err());
    let new = shorthand(&arena, source, VueVersion::V3);
    let imports = new.setup_component_imports().unwrap();
    assert_eq!(component_props::compare(&imports), Ok((1, 3, 2)));
    let mut rows = Vec::new();
    imports.sites()[0]
        .visit_passed_props(|row| rows.push((row.name, row.value, row.expression, row.name_span)))
        .unwrap();
    assert_eq!(rows.len(), 3);
    for (name, value, expression, name_span) in rows {
        assert_eq!(name_span.slice(source), name);
        if name == "flag" {
            assert_eq!((value, expression), (None, None));
        } else {
            assert_eq!((value, expression), (Some(name), Some(name_span)));
        }
    }
}

#[test]
fn unauthored_names_modifiers_and_legacy_dialects_keep_their_decisions() {
    for attribute in [
        ":user-name",
        "v-bind:user-name",
        ":title.camel",
        ":[title]",
        ".title",
        ":title=",
        ":true",
        ":class",
        ":await",
        ":3",
        "v-bind",
        ":",
    ] {
        let source = format!(
            "<script setup>const title=1;</script><template><div {attribute} /></template>"
        );
        let arena = Allocator::default();
        let old = observe(&arena, &source);
        let new = shorthand(&arena, &source, VueVersion::V3);
        assert!(new.admitted().is_none(), "{attribute}");
        assert_eq!(old.issues(), new.issues(), "{attribute}");
        assert_eq!(
            old.template().unwrap().holes(),
            new.template().unwrap().holes(),
            "{attribute}"
        );
    }
    let pre = "<script setup>const title=1;</script><template><div v-pre :title>{{ title }}</div></template>";
    let arena = Allocator::new();
    let before = observe(&arena, pre);
    let after = shorthand(&arena, pre, VueVersion::V3);
    assert_eq!(after.issues(), before.issues());
    assert_eq!(
        after.template().unwrap().holes(),
        before.template().unwrap().holes()
    );
    assert!(after.template().unwrap().embeds().is_empty());
    let arena = Allocator::default();
    for version in VueVersion::ALL
        .into_iter()
        .filter(|version| version.is_legacy())
    {
        let source = "<template><div :title /></template>";
        let new = shorthand(&arena, source, version);
        assert!(new.admitted().is_none());
        assert!(new.template().is_none());
    }
}

mod corpus;
mod profile;
