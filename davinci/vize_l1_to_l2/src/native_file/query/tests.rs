extern crate std;

use super::{NativePositionQueryError as Error, NativeSfc};
use crate::native_file::{NativeSfcObservation, lower_sfc_native};
use alloc::{vec, vec::Vec};
use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
};
use vize_l1::{SurfaceParseOptions, container::vue::DescriptorOptions};
use vize_l2::{file::PositionQueryError, resolution::Usage};

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

fn at(source: &str, needle: &str, index: usize) -> u32 {
    source.match_indices(needle).nth(index).unwrap().0 as u32
}

fn spelling(source: &str, needle: &str, index: usize) -> Span {
    let start = at(source, needle, index);
    Span::new(start, start + needle.len() as u32)
}

fn uses(native: &NativeSfc<'_, '_>, at: u32) -> Vec<Span> {
    let binding = native.binding_at_offset(at).unwrap().unwrap();
    let mut spans = Vec::new();
    native
        .for_each_reference_to(binding, |reference| spans.push(reference.span()))
        .unwrap();
    spans.sort_unstable_by_key(|span| (span.start, span.end));
    spans
}

#[test]
fn original_program_and_template_uses_share_one_borrowed_file_without_new_arena_work() {
    let arena = Allocator::default();
    let source = "<script setup>const value=1;value;</script><template><p :title='value'>{{value}}</p></template>";
    let owner = observe(&arena, source);
    let native = owner.admitted().unwrap();
    let file = native.file().file();
    let before = arena.allocated_bytes();
    let declaration = native
        .binding_at_offset(at(source, "value", 0))
        .unwrap()
        .unwrap();
    assert!(
        native
            .reference_at_offset(at(source, "value", 0))
            .unwrap()
            .is_none()
    );
    assert_eq!(
        uses(&native, at(source, "value", 0)),
        vec![
            spelling(source, "value", 1),
            spelling(source, "value", 2),
            spelling(source, "value", 3)
        ]
    );
    for index in 1..4 {
        let reference = native
            .reference_at_offset(at(source, "value", index))
            .unwrap()
            .unwrap();
        assert!(core::ptr::eq(reference.file(), file));
        assert!(reference.binding().same_owner(declaration));
        assert_eq!(reference.binding().id(), declaration.id());
        assert_eq!(reference.span(), spelling(source, "value", index));
        assert_eq!(reference.usage(), Usage::Read);
        assert_eq!(reference.scope(), native.file().setup().unwrap().scope());
        assert_eq!(reference.script().is_some(), index == 1);
        assert_eq!(reference.template().is_some(), index != 1);
        if let Some(node) = reference.node() {
            let original = owner
                .template()
                .unwrap()
                .embeds()
                .iter()
                .find(|embed| embed.node == Some(node))
                .unwrap();
            let table = file.expression(node).unwrap().table().unwrap();
            assert!(core::ptr::eq(
                table.expression().ast,
                original.syntax.expression().unwrap()
            ));
            assert!(core::ptr::eq(
                reference.template().unwrap(),
                &table.occurrences()[0]
            ));
        }
    }
    assert_eq!(arena.allocated_bytes(), before);
}

#[test]
fn original_entity_and_escaped_identifier_spans_preserve_complete_authored_spellings() {
    let arena = Allocator::default();
    let source = "<script setup lang=ts>const fj=1;const café=2;</script>\r\n<template><p :title='&fjlig;'>{{caf\\u00e9}}</p></template>";
    let owner = observe(&arena, source);
    let native = owner.admitted().unwrap();
    let entity = spelling(source, "&fjlig;", 0);
    for position in entity.start..entity.end {
        let reference = native.reference_at_offset(position).unwrap().unwrap();
        assert_eq!(reference.span(), entity);
        assert_eq!(reference.template().unwrap().name, "fj");
        assert_eq!(
            reference.binding().declaration().unwrap().span,
            spelling(source, "fj", 0)
        );
    }
    assert_eq!(uses(&native, entity.start), vec![entity]);
    let escaped = native
        .reference_at_offset(at(source, "caf\\u00e9", 0))
        .unwrap()
        .unwrap();
    assert_eq!(escaped.span(), spelling(source, "caf\\u00e9", 0));
    assert_eq!(escaped.template().unwrap().name, "café");
    assert_eq!(
        escaped.binding().declaration().unwrap().span,
        spelling(source, "café", 0)
    );
}

#[test]
fn half_open_utf8_sites_and_original_source_holes_remain_precise() {
    let arena = Allocator::default();
    let source = "<script setup>/*😀*/const café=1;</script><template>{{café}}</template><style>.café{color:red}</style>";
    let owner = observe(&arena, source);
    let native = owner.admitted().unwrap();
    let name = spelling(source, "café", 1);
    assert!(native.binding_at_offset(name.end).unwrap().is_none());
    assert!(native.reference_at_offset(name.end).unwrap().is_none());
    assert!(
        native
            .binding_at_offset(at(source, "café", 2))
            .unwrap()
            .is_none()
    );
    assert!(
        native
            .reference_at_offset(at(source, "😀", 0))
            .unwrap()
            .is_none()
    );
    assert!(
        native
            .reference_at_offset(source.len() as u32)
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        native.reference_at_offset(name.start + 4),
        Err(Error::File(PositionQueryError::NotCharBoundary))
    ));
    assert!(matches!(
        native.binding_at_offset(u32::MAX),
        Err(Error::File(PositionQueryError::OutOfBounds))
    ));
}

#[test]
fn equal_bytes_and_equal_numeric_ids_do_not_accept_another_original_file_binding() {
    let first_arena = Allocator::default();
    let second_arena = Allocator::default();
    let source = "<script setup>const value=1;</script><template>{{value}}</template>";
    let copied = Vec::from(source.as_bytes());
    let copied = core::str::from_utf8(&copied).unwrap();
    let first_owner = observe(&first_arena, source);
    let second_owner = observe(&second_arena, copied);
    let first = first_owner.admitted().unwrap();
    let second = second_owner.admitted().unwrap();
    let at = at(source, "value", 1);
    let original = first.binding_at_offset(at).unwrap().unwrap();
    let foreign = second.binding_at_offset(at).unwrap().unwrap();
    assert_eq!(original.id(), foreign.id());
    assert!(!original.same_owner(foreign));
    let mut visits = 0;
    assert_eq!(
        first.for_each_reference_to(foreign, |_| visits += 1),
        Err(Error::ForeignBinding)
    );
    assert_eq!(visits, 0);
    assert_eq!(uses(&first, at), uses(&second, at));
}

#[test]
fn setup_shadow_and_import_aliases_use_original_declarations_not_name_fallback() {
    let arena = Allocator::default();
    let source = "<script setup>import { run as local } from 'dep';const value=2;</script><script>const value=1;</script><template>{{local(value)}}</template>";
    let owner = observe(&arena, source);
    let native = owner.admitted().unwrap();
    let value = native
        .binding_at_offset(at(source, "value", 2))
        .unwrap()
        .unwrap();
    assert_eq!(
        value.declaration().unwrap().span,
        spelling(source, "value", 0)
    );
    let local = native
        .reference_at_offset(at(source, "local", 1))
        .unwrap()
        .unwrap();
    assert_eq!(
        local.binding().declaration().unwrap().span,
        spelling(source, "local", 0)
    );
    assert!(
        native
            .binding_at_offset(at(source, "run", 0))
            .unwrap()
            .is_none()
    );
    assert!(uses(&native, at(source, "value", 1)).is_empty());
}

#[test]
fn original_refused_assembly_cannot_mint_a_query_view_from_partial_file_facts() {
    let arena = Allocator::default();
    for source in [
        "<script>const value=1;</script><template>{{value}}</template>",
        "<script setup>const value=1;</script><template><p v-if='value'>{{value}}</p></template>",
        "<script setup>const value=/x/uv;</script><template>{{value}}</template>",
    ] {
        let original = observe(&arena, source);
        assert!(original.admitted().is_none());
        assert!(!original.issues().is_empty());
        assert!(core::ptr::eq(original.descriptor().source(), source));
    }
}
