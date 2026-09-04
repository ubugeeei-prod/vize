//! P2-11 installment 96 witness: **the order cache slots are numbered
//! in**.
//!
//! The shipped codegen takes a `_cache` slot each time it *reaches* a
//! construct while printing, so its numbering follows the printed order
//! of the render function. This lane renders slot bodies where the
//! children sit — source order — because that is the order `_hoisted_N`
//! needs: the shipped lane assigns *those* in the transform. The two
//! orders differ whenever a slot object prints a named `<template #x>`
//! ahead of an implicit default body the author wrote first, so the
//! emitted sites are recorded and the numbering is re-derived at the end.
//! Compared byte-for-byte with the shipped lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_atelier_core::options::{BindingMetadata, BindingType, CodegenMode, CodegenOptions};
use vize_atelier_dom::DomCompilerOptions;
use vize_s0::FxHashMap;
use vize_s1_to_s2::{BindingKind, BindingTable, DomEmitMode, DomEmitOptions};

const BATTERY: &[(&str, &str)] = &[
    // The shape that reorders: the default body is authored first and
    // prints last.
    (
        "default_then_named",
        r#"<MyComp><b @click="go(1)">d</b><template #x><i @click="go(2)">x</i></template></MyComp>"#,
    ),
    (
        "default_then_two_named",
        r#"<MyComp><b @click="go(1)">d</b><template #a><i @click="go(2)">a</i></template><template #b><i @click="go(3)">b</i></template></MyComp>"#,
    ),
    // Already in printed order: nothing to move.
    (
        "named_then_default",
        r#"<MyComp><template #x><i @click="go(2)">x</i></template><b @click="go(1)">d</b></MyComp>"#,
    ),
    (
        "two_named",
        r#"<MyComp><template #a><i @click="go(1)">a</i></template><template #b><i @click="go(2)">b</i></template></MyComp>"#,
    ),
    (
        "named_only",
        r#"<MyComp><template #x><i @click="go(1)">x</i></template></MyComp>"#,
    ),
    (
        "default_only",
        r#"<MyComp><b @click="go(1)">d</b></MyComp>"#,
    ),
    // A slot taking a slot: the inner object renumbers inside the outer
    // one's range.
    (
        "nested_slots",
        r#"<MyComp><template #x><Inner><b @click="go(1)">i</b><template #y><i @click="go(2)">y</i></template></Inner></template><b @click="go(3)">d</b></MyComp>"#,
    ),
    // Slots beside handlers outside them.
    (
        "outside_then_slot",
        r#"<p @click="go(0)"></p><MyComp><b @click="go(1)">d</b><template #x><i @click="go(2)">x</i></template></MyComp>"#,
    ),
    (
        "slot_then_outside",
        r#"<MyComp><b @click="go(1)">d</b><template #x><i @click="go(2)">x</i></template></MyComp><p @click="go(0)"></p>"#,
    ),
    // `v-memo` takes its slot before it writes the wrapper, and prints
    // the number after the body — the one construct whose digits are not
    // where its slot was taken.
    (
        "memo_around_handler",
        r#"<div v-memo="[count]"><b @click="go(1)">x</b></div>"#,
    ),
    (
        "memo_in_slot",
        r#"<MyComp><b @click="go(1)">d</b><template #x><div v-memo="[count]"><i @click="go(2)">x</i></div></template></MyComp>"#,
    ),
    (
        "memo_then_handler",
        r#"<div v-memo="[count]">x</div><p @click="go(1)"></p>"#,
    ),
    (
        "v_for_memo",
        r#"<li v-for="i in items" :key="i" v-memo="[i]">{{ i }}</li><p @click="go(1)"></p>"#,
    ),
    // `v-once` shares the counter.
    (
        "once_then_handler",
        r#"<div v-once>x</div><p @click="go(1)"></p>"#,
    ),
    (
        "once_in_slot",
        r#"<MyComp><b @click="go(1)">d</b><template #x><div v-once>x</div></template></MyComp>"#,
    ),
    // `v-model` update handlers take slots too.
    (
        "model_then_named",
        r#"<MyComp><input v-model="msg"><template #x><input v-model="msg"></template></MyComp>"#,
    ),
    // `_createSlots`: a conditional or dynamically named slot leaves the
    // object and prints in the `{ name, fn }` array *after* the `default:`
    // bucket, so a body authored second still prints first. The entry is
    // captured like every other piece, so it has to be re-registered
    // where it prints or the whole renumbering goes uncovered.
    (
        "create_slots_conditional_then_default",
        r#"<MyComp><template #x v-if="ok"><i @click="go(2)">x</i></template><b @click="go(1)">d</b></MyComp>"#,
    ),
    (
        "create_slots_dynamic_name_then_default",
        r#"<MyComp><template #[name]><i @click="go(2)">x</i></template><b @click="go(1)">d</b></MyComp>"#,
    ),
    (
        "create_slots_two_entries_then_default",
        r#"<MyComp><template #x v-if="ok"><i @click="go(2)">x</i></template><template #[name]><i @click="go(3)">y</i></template><b @click="go(1)">d</b></MyComp>"#,
    ),
    (
        "create_slots_default_then_conditional",
        r#"<MyComp><b @click="go(1)">d</b><template #x v-if="ok"><i @click="go(2)">x</i></template></MyComp>"#,
    ),
    (
        "create_slots_model_in_entry",
        r#"<MyComp><template #x v-if="ok"><input v-model="msg"></template><input v-model="msg"></MyComp>"#,
    ),
    (
        "create_slots_nested_entry",
        r#"<MyComp><template #x v-if="ok"><Inner><b @click="go(2)">i</b><template #y><i @click="go(3)">y</i></template></Inner></template><b @click="go(1)">d</b></MyComp>"#,
    ),
];

fn metadata() -> BindingMetadata {
    let entries: &[(&str, BindingType)] = &[
        ("go", BindingType::SetupConst),
        ("count", BindingType::SetupRef),
        ("msg", BindingType::SetupLet),
        ("items", BindingType::SetupConst),
        ("ok", BindingType::SetupRef),
        ("name", BindingType::SetupRef),
        ("MyComp", BindingType::SetupConst),
        ("Inner", BindingType::SetupConst),
    ];
    let mut bindings = FxHashMap::default();
    for (name, kind) in entries {
        bindings.insert((*name).into(), *kind);
    }
    BindingMetadata {
        bindings,
        props_aliases: FxHashMap::default(),
        is_script_setup: true,
    }
}

fn binding_kind(kind: BindingType) -> BindingKind {
    match kind {
        BindingType::SetupLet => BindingKind::SetupLet,
        BindingType::SetupMaybeRef => BindingKind::SetupMaybeRef,
        BindingType::SetupRef => BindingKind::SetupRef,
        BindingType::SetupReactiveConst => BindingKind::SetupReactiveConst,
        BindingType::SetupConst => BindingKind::SetupConst,
        BindingType::Props => BindingKind::Props,
        BindingType::PropsAliased => BindingKind::PropsAliased,
        BindingType::Data => BindingKind::Data,
        BindingType::Options => BindingKind::Options,
        BindingType::LiteralConst => BindingKind::LiteralConst,
        BindingType::JsGlobalUniversal => BindingKind::JsGlobalUniversal,
        BindingType::JsGlobalBrowser => BindingKind::JsGlobalBrowser,
        BindingType::JsGlobalNode => BindingKind::JsGlobalNode,
        BindingType::JsGlobalDeno => BindingKind::JsGlobalDeno,
        BindingType::JsGlobalBun => BindingKind::JsGlobalBun,
        BindingType::VueGlobal => BindingKind::VueGlobal,
        BindingType::ExternalModule => BindingKind::ExternalModule,
    }
}

fn table(metadata: &BindingMetadata) -> BindingTable {
    BindingTable::new(
        metadata
            .bindings
            .iter()
            .map(|(name, kind)| (name.as_str(), binding_kind(*kind))),
        [],
        metadata.is_script_setup,
    )
}

fn dom_options(metadata: &BindingMetadata, cache_handlers: bool) -> DomCompilerOptions {
    DomCompilerOptions {
        mode: CodegenMode::Module,
        prefix_identifiers: true,
        inline: true,
        cache_handlers,
        binding_metadata: Some(metadata.clone()),
        ..Default::default()
    }
}

fn emit_options(table: &BindingTable, cache_handlers: bool) -> DomEmitOptions<'_> {
    DomEmitOptions {
        mode: DomEmitMode::Module,
        prefix_identifiers: true,
        inline: true,
        cache_handlers,
        bindings: Some(table),
        ..DomEmitOptions::DEFAULT
    }
}

#[test]
fn cache_slots_are_numbered_in_printed_order() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, true),
        &CodegenOptions::default(),
        &emit_options(&table, true),
    );
}

/// The same battery without `cache_handlers`: `v-once`, `v-memo` and the
/// `v-model` update handlers still take slots, so the ordering rule has
/// to hold there too.
#[test]
fn the_same_battery_agrees_without_cached_handlers() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, false),
        &CodegenOptions::default(),
        &emit_options(&table, false),
    );
}

/// The numbering itself, pinned.
#[test]
fn a_named_slot_printed_first_takes_the_first_slot() {
    let metadata = metadata();
    let table = table(&metadata);
    let allocator = vize_s0::Allocator::new();
    let slots = |src: &str| {
        let assembled = vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &emit_options(&table, true),
        )
        .expect("cache slot order witness must emit")
        .assembled();
        assembled
            .lines()
            .filter(|line| line.contains("_cache["))
            .map(|line| line.trim().to_string())
            .collect::<Vec<_>>()
    };
    let memo_tail = |src: &str| {
        vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &emit_options(&table, true),
        )
        .expect("cache slot order witness must emit")
        .assembled()
        .lines()
        .find(|line| line.ends_with(", _cache, 0)") || line.ends_with(", _cache, 1)"))
        .unwrap_or_default()
        .trim()
        .to_string()
    };
    // `#x` prints before the implicit default, so it takes slot 0 even
    // though the default body was authored — and rendered — first.
    assert_eq!(
        slots(
            r#"<MyComp><b @click="go(1)">d</b><template #x><i @click="go(2)">x</i></template></MyComp>"#
        ),
        vec![
            "onClick: _cache[0] || (_cache[0] = $event => (go(2)))".to_string(),
            "onClick: _cache[1] || (_cache[1] = $event => (go(1)))".to_string(),
        ]
    );
    // `withMemo` takes its slot before the wrapper and prints the number
    // after the body, so the handler inside it numbers second.
    assert_eq!(
        slots(r#"<div v-memo="[count]"><b @click="go(1)">x</b></div>"#),
        vec!["onClick: _cache[1] || (_cache[1] = $event => (go(1)))".to_string()]
    );
    assert_eq!(
        memo_tail(r#"<div v-memo="[count]"><b @click="go(1)">x</b></div>"#),
        "])), _cache, 0)"
    );
}
