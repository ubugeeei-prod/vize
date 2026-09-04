//! P2-11 installment 95 witness: **`cache_handlers`**.
//!
//! `compile_template_block` turns the option on with `inline`, so every
//! production `<script setup>` compile hoists its inline `v-on` handlers
//! into the render function's `_cache` array — the closure is built once
//! instead of on every render. A cached handler stops being a patch
//! target, a cached *reference* is guarded and forwarded rather than
//! stored bare, and the synthesized `v-model` update handler takes a
//! slot like any other. Compared byte-for-byte with the shipped lane.

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
    ("ref_const", r#"<div @click="handler"></div>"#),
    ("ref_let", r#"<div @click="looseFn"></div>"#),
    ("ref_member", r#"<div @click="obj.go"></div>"#),
    ("call", r#"<div @click="handler(1)"></div>"#),
    ("stmts", r#"<div @click="handler(1); handler(2)"></div>"#),
    ("inline_arrow", r#"<div @click="() => handler(1)"></div>"#),
    ("modifier", r#"<div @click.stop="handler(1)"></div>"#),
    ("key_modifier", r#"<input @keyup.enter="handler(1)">"#),
    (
        "both_modifiers",
        r#"<input @keyup.enter.stop="handler(1)">"#,
    ),
    (
        "two_handlers",
        r#"<div @click="handler(1)" @input="handler(2)"></div>"#,
    ),
    (
        "with_once",
        r#"<div v-once>{{ count }}</div><p @click="handler(1)"></p>"#,
    ),
    (
        "with_memo",
        r#"<div v-memo="[count]">{{ count }}</div><p @click="handler(1)"></p>"#,
    ),
    (
        "in_slot",
        r#"<MyComp v-slot="s"><div @click="handler(s)"></div></MyComp>"#,
    ),
    (
        "in_v_for",
        r#"<li v-for="i in items" :key="i" @click="handler(i)"></li>"#,
    ),
    ("component", r#"<MyComp @finish="handler(1)" />"#),
    ("no_handler", r#"<div @click></div>"#),
    (
        "merged",
        r#"<div v-bind="attrs" @click="handler(1)"></div>"#,
    ),
    ("assign_ref", r#"<div @click="count = 1"></div>"#),
    ("increment", r#"<div @click="count++"></div>"#),
    ("model_native", r#"<input v-model="looseFn">"#),
    ("model_component", r#"<MyComp v-model="count" />"#),
    ("vnode_hook", r#"<div @vue:mounted="handler(1)"></div>"#),
    (
        "update_event",
        r#"<MyComp @update:modelValue="handler(1)" />"#,
    ),
    (
        "custom_event_upper",
        r#"<div @customEvent="handler(1)"></div>"#,
    ),
    ("once_modifier", r#"<div @click.once="handler(1)"></div>"#),
    (
        "capture_passive",
        r#"<div @click.capture.passive="handler(1)"></div>"#,
    ),
    ("dynamic_event", r#"<div @[evt]="handler(1)"></div>"#),
    (
        "nested_slot_scope",
        r#"<MyComp><template #a="s"><div @click="handler(s)"></div></template></MyComp>"#,
    ),
    ("slot_outlet", r#"<slot @click="handler(1)" />"#),
    (
        "nested_for_and_click",
        r#"<ul><li v-for="i in items" :key="i"><b @click="handler(i)"></b></li></ul>"#,
    ),
    (
        "if_branch",
        r#"<div v-if="count" @click="handler(1)"></div><p v-else @click="handler(2)"></p>"#,
    ),
    (
        "two_elements",
        r#"<div @click="handler(1)"></div><div @click="handler(2)"></div>"#,
    ),
    (
        "once_then_handler",
        r#"<div v-once>x</div><p @click="handler(1)"></p><p @click="handler(2)"></p>"#,
    ),
    (
        "directive_and_handler",
        r#"<div v-focus @click="handler(1)"></div>"#,
    ),
    (
        "teleport",
        r##"<Teleport to="#a" @click="handler(1)"><b>x</b></Teleport>"##,
    ),
    (
        "class_and_handler",
        r#"<div :class="count" @click="handler(1)"></div>"#,
    ),
    (
        "style_and_handler",
        r#"<div :style="count" @click="handler(1)"></div>"#,
    ),
    (
        "handler_with_event",
        r#"<div @click="handler($event)"></div>"#,
    ),
    (
        "arrow_with_param",
        r#"<div @click="(e) => handler(e)"></div>"#,
    ),
    ("optional_chain_ref", r#"<div @click="obj?.go"></div>"#),
    ("component_ref_handler", r#"<MyComp @finish="handler" />"#),
    ("component_let_handler", r#"<MyComp @finish="looseFn" />"#),
];

fn metadata() -> BindingMetadata {
    let entries: &[(&str, BindingType)] = &[
        ("handler", BindingType::SetupConst),
        ("looseFn", BindingType::SetupLet),
        ("obj", BindingType::SetupConst),
        ("count", BindingType::SetupRef),
        ("items", BindingType::SetupConst),
        ("attrs", BindingType::SetupConst),
        ("MyComp", BindingType::SetupConst),
        ("evt", BindingType::SetupRef),
        ("vFocus", BindingType::SetupConst),
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

fn table(m: &BindingMetadata) -> BindingTable {
    BindingTable::new(
        m.bindings
            .iter()
            .map(|(n, k)| (n.as_str(), binding_kind(*k))),
        [],
        m.is_script_setup,
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
fn cached_handlers_match_the_shipped_dom_lane() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, true),
        &CodegenOptions::default(),
        &emit_options(&table, true),
    );
}

/// The same battery with the option off: every handler stays inline and
/// keeps its `PROPS` flag, so a lane that cached unconditionally passes
/// the run above and fails this one.
#[test]
fn the_same_battery_keeps_its_handlers_inline_without_the_option() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, false),
        &CodegenOptions::default(),
        &emit_options(&table, false),
    );
}

/// The distinctive spellings, pinned on both sides of the option.
#[test]
fn the_slot_the_guard_and_the_dropped_patch_flag_are_option_only() {
    let metadata = metadata();
    let table = table(&metadata);
    let allocator = vize_s0::Allocator::new();
    let render = |src: &str, cache: bool| {
        vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &emit_options(&table, cache),
        )
        .expect("cache handler witness must emit")
        .assembled()
        .lines()
        .find(|line| line.contains("onClick"))
        .unwrap_or_default()
        .trim()
        .to_string()
    };
    // An inline statement takes a slot and stops being a patch target.
    assert_eq!(
        render(r#"<div @click="handler(1)"></div>"#, true),
        "onClick: _cache[0] || (_cache[0] = $event => (handler(1)))"
    );
    assert_eq!(
        render(r#"<div @click="handler(1)"></div>"#, false),
        "onClick: $event => (handler(1))"
    );
    // A reference the script can rebind is guarded and forwarded.
    assert_eq!(
        render(r#"<div @click="looseFn"></div>"#, true),
        "onClick: _cache[0] || (_cache[0] = (...args) => (looseFn && looseFn(...args)))"
    );
    // A `SetupConst` reference needs no slot at all — the shipped
    // `needs_von_handler_cache` carves it out.
    assert_eq!(
        render(r#"<div @click="handler"></div>"#, true),
        "return (_openBlock(), _createElementBlock(\"div\", { onClick: handler }))"
    );
    // Modifiers sit inside the slot, not around it.
    assert_eq!(
        render(r#"<div @click.stop="handler(1)"></div>"#, true),
        "onClick: _cache[0] || (_cache[0] = _withModifiers($event => (handler(1)), [\"stop\"]))"
    );
}

/// The patch flag and `dynamicProps`, which the cache rule reads without
/// the const-reference carve-out the *emission* applies.
#[test]
fn a_cached_handler_leaves_the_dynamic_props_array() {
    let metadata = metadata();
    let table = table(&metadata);
    let allocator = vize_s0::Allocator::new();
    let flag_line = |src: &str, cache: bool| {
        vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &emit_options(&table, cache),
        )
        .expect("cache handler witness must emit")
        .assembled()
        .lines()
        .find(|line| line.contains("/* PROPS */") || line.contains("/* NEED_PATCH */"))
        .unwrap_or_default()
        .trim()
        .to_string()
    };
    assert_eq!(
        flag_line(r#"<div :id="count" @click="handler(1)"></div>"#, true),
        "}, null, 8 /* PROPS */, [\"id\"]))"
    );
    assert_eq!(
        flag_line(r#"<div :id="count" @click="handler(1)"></div>"#, false),
        "}, null, 8 /* PROPS */, [\"id\", \"onClick\"]))"
    );
    // A native `v-model` alone: the cached update handler drops `PROPS`,
    // and `NEED_PATCH` — which the shipped gate names `v-model` in —
    // takes its place.
    assert_eq!(
        flag_line(r#"<input v-model="looseFn">"#, true),
        "}, null, 512 /* NEED_PATCH */)), ["
    );
}
