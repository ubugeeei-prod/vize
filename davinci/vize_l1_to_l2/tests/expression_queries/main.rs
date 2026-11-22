//! Compare File expression queries to both the original SFC assembly and
//! Croquis's independent parser-owned reference packet.
#![expect(clippy::unwrap_used, reason = "a refused fixture is a test failure")]
use std::{collections::BTreeSet, ptr};
use vize_atelier_core::Parser;
use vize_croquis::drawer::{TemplateReferenceOwner, collect_template_references};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::{NativeSfcObservation, lower_sfc_native};
use vize_l2::query::{
    ExpressionPositionIndex, ExpressionReferenceRef, Model, PositionQueryError, TemplateQueryError,
};

#[cfg(feature = "setup-component-imports")]
mod component_imports;
#[cfg(feature = "setup-component-imports")]
mod component_projection;
mod corpus;
mod input;
#[cfg(feature = "template-operand-queries")]
mod operand;
mod owner;
mod profile;

fn observe<'a>(arena: &'a Allocator, source: &'a str) -> NativeSfcObservation<'a> {
    lower_sfc_native(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    )
}

fn equal(left: ExpressionReferenceRef<'_, '_>, right: ExpressionReferenceRef<'_, '_>) {
    assert!(left.binding().same_owner(right.binding()));
    assert_eq!(left.binding().id(), right.binding().id());
    assert_eq!(left.node(), right.node());
    assert_eq!(left.scope(), right.scope());
    assert_eq!(left.span(), right.span());
    assert!(ptr::eq(left.occurrence(), right.occurrence()));
}

#[derive(Default, Debug)]
struct Counts {
    files: usize,
    invalid_utf8: usize,
    refused: usize,
    admitted: usize,
    templates: usize,
    uses: usize,
    positions: usize,
    bindings: usize,
    parser_diagnostics: usize,
    #[cfg(feature = "template-operand-queries")]
    operands: usize,
}

fn compare(source: &str, path: &str, counts: &mut Counts, every_byte: bool) -> bool {
    counts.files += 1;
    let arena = Allocator::default();
    let observe = observe(&arena, source);
    let Some(native) = observe.admitted() else {
        counts.refused += 1;
        return false;
    };
    counts.admitted += 1;
    let file = native.file().file();
    let model = Model::new(file);
    let index = ExpressionPositionIndex::new(file);
    let mut rows = Vec::new();
    model
        .for_each_expression_reference(|row| rows.push(row))
        .unwrap();
    let key = |row: &ExpressionReferenceRef<'_, '_>| {
        (row.span().start, row.span().end, row.node().index())
    };
    rows.sort_by_key(key);
    counts.uses += rows.len();

    let mut expected = Vec::new();
    for binding in file.bindings() {
        counts.bindings += 1;
        let mut original = Vec::new();
        native
            .for_each_reference_to(binding, |row| {
                if row.template().is_some() {
                    original.push(row);
                }
            })
            .unwrap();
        original.sort_by_key(|row| {
            (
                row.span().start,
                row.span().end,
                row.node().unwrap().index(),
            )
        });
        let mut by_binding = Vec::new();
        model
            .for_each_expression_reference_to(binding, |row| by_binding.push(row))
            .unwrap();
        by_binding.sort_by_key(key);
        assert_eq!(by_binding.len(), original.len(), "{path}");
        for (&old, &new) in original.iter().zip(&by_binding) {
            assert!(ptr::eq(old.file(), new.binding().file()), "{path}");
            assert_eq!(old.binding().id(), new.binding().id(), "{path}");
            assert_eq!(old.scope(), new.scope(), "{path}");
            assert_eq!(old.span(), new.span(), "{path}");
            assert_eq!(old.node(), Some(new.node()), "{path}");
            assert!(ptr::eq(old.template().unwrap(), new.occurrence()), "{path}");
            expected.push(new);
        }
    }
    expected.sort_by_key(key);
    assert_eq!(rows.len(), expected.len(), "{path}");
    for (&row, &original) in rows.iter().zip(&expected) {
        equal(row, original);
    }

    let mut probes = BTreeSet::from([0, u32::try_from(source.len()).unwrap()]);
    for row in &rows {
        let span = row.span();
        probes.extend([span.start, span.end - 1, span.end]);
    }
    if every_byte {
        probes.extend(0..u32::try_from(source.len()).unwrap());
    }
    for offset in probes {
        counts.positions += 1;
        let scan = model.expression_reference_at_offset(offset);
        let indexed = index.expression_reference_at_offset(offset);
        if !source.is_char_boundary(offset as usize) {
            let err = Some(TemplateQueryError::Position(
                PositionQueryError::NotCharBoundary,
            ));
            assert_eq!(scan.err(), err, "{path}");
            assert_eq!(indexed.err(), err, "{path}");
            continue;
        }
        let old = native
            .reference_at_offset(offset)
            .unwrap()
            .filter(|s| s.template().is_some());
        let scan = scan.unwrap();
        let indexed = indexed.unwrap();
        assert_eq!(old.is_some(), scan.is_some(), "position {offset} in {path}");
        assert_eq!(
            old.is_some(),
            indexed.is_some(),
            "index at {offset} in {path}"
        );
        if let (Some(old), Some(row), Some(hit)) = (old, scan, indexed) {
            assert_eq!(old.span(), row.span(), "{path}");
            assert!(ptr::eq(old.template().unwrap(), row.occurrence()), "{path}");
            equal(row, hit);
        }
    }

    if let Some(template) = observe.descriptor().admitted().unwrap().template() {
        counts.templates += 1;
        let block = template.block();
        let (root, diagnostics, syntax) =
            Parser::new(&arena, block.source()).parse_with_template_syntax_facts();
        counts.parser_diagnostics += diagnostics
            .iter()
            .filter(|d| !d.is_compatibility_notice())
            .count();
        #[cfg(feature = "template-operand-queries")]
        {
            counts.operands += operand::compare(&root, block.start(), model, path);
        }
        let croquis = collect_template_references(&root, &syntax);
        assert!(croquis.gaps().is_empty(), "{path}: {:?}", croquis.gaps());
        let mut lexical: Vec<_> = croquis
            .references()
            .iter()
            .map(|r| {
                assert_eq!(r.owner, TemplateReferenceOwner::ComponentContext, "{path}");
                (
                    r.span.start + block.start(),
                    r.span.end + block.start(),
                    r.name.as_str(),
                )
            })
            .collect();
        lexical.sort_unstable();
        let actual: Vec<_> = rows
            .iter()
            .map(|r| (r.span().start, r.span().end, r.occurrence().name))
            .collect();
        assert_eq!(actual, lexical, "{path}");
    } else {
        assert!(rows.is_empty(), "{path}");
    }
    true
}

#[test]
fn original_bindings_scopes_usage_and_authored_offsets() {
    let cases = [
        "<script setup>const value=0;</script><template><p :title='value + value'>{{ value }}</p></template>",
        "<script setup lang=ts>const fj=1;const café=2;</script>\r\n<template><p :title='&fjlig;'>{{caf\\u00e9}}</p></template>",
        "<script setup>const value=1; const read=2;</script><template><p :title='({value, n: read})'>{{ value }}</p></template>",
        "<script setup>let value=1;</script><template>{{ value++ }}</template>",
        "<script setup>import { run as local } from 'dep';const value=2;</script><script>const value=1;</script><template>{{local(value)}}</template>",
        "<script setup>let n=0;const other=1;</script><template>{{n=other}}</template>",
        "<script setup>const Ctor=0;</script><template>{{new Ctor()}}</template>",
        "<template><p>no expressions here</p></template>",
    ];
    let mut count = Counts::default();
    for s in cases {
        assert!(compare(s, s, &mut count, true), "{s}");
    }
    assert!(count.uses >= 11, "{count:?}");
}

#[test]
fn a_foreign_binding_and_invalid_byte_positions_are_typed_refusals() {
    let a = Allocator::default();
    let b = Allocator::default();
    let source = "<script setup>const value=1;</script><template>雪{{value}}</template>";
    let first = observe(&a, source);
    let other = observe(&b, source);
    let first = first.admitted().unwrap();
    let other = other.admitted().unwrap();
    let file = first.file().file();
    let binding = other.file().file().bindings().next().unwrap();
    let model = Model::new(file);
    let mut visits = 0;
    assert_eq!(
        model.for_each_expression_reference_to(binding, |_| visits += 1),
        Err(TemplateQueryError::ForeignSymbol)
    );
    assert_eq!(visits, 0);
    let index = ExpressionPositionIndex::new(file);
    for (at, error) in [
        (u32::MAX, PositionQueryError::OutOfBounds),
        (
            source.find('雪').unwrap() as u32 + 1,
            PositionQueryError::NotCharBoundary,
        ),
    ] {
        assert_eq!(
            model.expression_reference_at_offset(at).err(),
            Some(TemplateQueryError::Position(error))
        );
        assert_eq!(
            index.expression_reference_at_offset(at).err(),
            Some(TemplateQueryError::Position(error))
        );
    }
    assert!(ptr::eq(index.into_file(), file));
}

#[cfg(feature = "component-prop-queries")]
mod component_props;

#[cfg(all(
    feature = "authored-bind-shorthand",
    feature = "component-prop-queries",
    feature = "template-operand-queries"
))]
mod shorthand;

#[cfg(all(
    feature = "authored-bind-shorthand",
    feature = "component-prop-queries",
    feature = "template-operand-queries"
))]
mod shorthand_products;
