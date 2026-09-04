//! P2-11 installment 91 witness: **where `_unref` lists in the preamble**.
//!
//! `_unref` is registered by `process_expression` — a *transform* call —
//! so the shipped lane lists it among `root.helpers`, at the node whose
//! expression needed it, ahead of the helpers that node's own transform
//! step registers afterwards. `v-for` is the visible case: the lane
//! processes its source expression and *then* registers `renderList`, so
//! a loop over a `let` binding imports `unref` before `renderList`.
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
    // The source expression is processed before `renderList` is
    // registered, so `unref` precedes it.
    (
        "vfor_over_let",
        r#"<li v-for="i in items" :key="i">{{ i }}</li>"#,
    ),
    (
        "vfor_over_maybe_ref",
        r#"<li v-for="i in theme" :key="i">{{ i }}</li>"#,
    ),
    // A const source needs no `_unref` at all.
    (
        "vfor_over_const",
        r#"<li v-for="i in list" :key="i">{{ i }}</li>"#,
    ),
    // The `_unref` is in the loop *body*, so it is registered after the
    // source — still before `renderList`, which the lane registers last.
    (
        "vfor_body_unref",
        r#"<li v-for="i in list" :key="i">{{ msg }}</li>"#,
    ),
    // An `_unref` in a sibling *after* the loop is registered after
    // `renderList`.
    (
        "unref_after_vfor",
        r#"<ul><li v-for="i in list" :key="i">{{ i }}</li></ul><p>{{ msg }}</p>"#,
    ),
    // Two loops, the first over a const: the second source is what first
    // needs `_unref`, and `renderList` is already registered by then.
    (
        "second_vfor_unref",
        r#"<ul><li v-for="i in list" :key="i">{{ i }}</li></ul><ul><li v-for="j in items" :key="j">{{ j }}</li></ul>"#,
    ),
    // A compound handler: the shipped lane wraps it in `$event => (…)`
    // *after* rewriting the body, and the body's `_unref` is still the
    // transform's registration.
    (
        "handler_compound",
        r#"<button @click="toggle(1)">x</button>"#,
    ),
    (
        "handler_statements",
        r#"<button @click="toggle(1); toggle(2)">x</button>"#,
    ),
    ("handler_reference", r#"<button @click="toggle">x</button>"#),
    (
        "handler_member",
        r#"<button @click="theme.go(1)">x</button>"#,
    ),
    // Slot content: `withCtx` is a codegen helper, so it lists after
    // every transform one.
    (
        "slot_content_unref",
        r#"<MyComp><template #a>{{ msg }}</template></MyComp>"#,
    ),
    (
        "slot_content_vfor",
        r#"<MyComp><template #a><li v-for="i in items" :key="i">{{ i }}</li></template></MyComp>"#,
    ),
    // `vShow` is a codegen helper too.
    ("vshow_unref", r#"<div v-show="msg"></div>"#),
    (
        "vshow_and_vfor",
        r#"<div v-show="msg"></div><li v-for="i in items" :key="i">{{ i }}</li>"#,
    ),
    // `v-memo` / `v-once` cache helpers around an `_unref` read.
    ("vmemo_unref", r#"<div v-memo="[msg]">{{ msg }}</div>"#),
    ("vonce_unref", "<div v-once>{{ msg }}</div>"),
    // Interpolation, bind, model and directive readings of the same
    // binding kinds.
    ("interpolation", "<p>{{ msg }} {{ theme }}</p>"),
    ("bind_value", r#"<div :id="msg"></div>"#),
    ("model_let", r#"<input v-model="msg">"#),
    ("vhtml_let", r#"<div v-html="msg"></div>"#),
    ("component_tag_let", "<LetComp />"),
    (
        "component_tag_then_vfor",
        r#"<LetComp /><li v-for="i in list" :key="i">{{ i }}</li>"#,
    ),
];

fn metadata() -> BindingMetadata {
    let entries: &[(&str, BindingType)] = &[
        ("msg", BindingType::SetupLet),
        ("toggle", BindingType::SetupLet),
        ("items", BindingType::SetupLet),
        ("theme", BindingType::SetupMaybeRef),
        ("list", BindingType::SetupConst),
        ("count", BindingType::SetupRef),
        ("MyComp", BindingType::SetupConst),
        ("LetComp", BindingType::SetupLet),
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

fn dom_options(metadata: &BindingMetadata, inline: bool) -> DomCompilerOptions {
    DomCompilerOptions {
        mode: CodegenMode::Module,
        prefix_identifiers: true,
        inline,
        binding_metadata: Some(metadata.clone()),
        ..Default::default()
    }
}

fn emit_options(table: &BindingTable, inline: bool) -> DomEmitOptions<'_> {
    DomEmitOptions {
        mode: DomEmitMode::Module,
        prefix_identifiers: true,
        inline,
        bindings: Some(table),
        ..DomEmitOptions::DEFAULT
    }
}

#[test]
fn unref_lists_where_the_shipped_transform_registers_it() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, true),
        &CodegenOptions::default(),
        &emit_options(&table, true),
    );
}

/// `_unref` is an inline-only spelling, so the same battery must keep
/// agreeing with `inline` off — where the helper never appears.
#[test]
fn the_same_battery_agrees_without_the_inline_option() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, false),
        &CodegenOptions::default(),
        &emit_options(&table, false),
    );
}

/// The import list itself, pinned: the dual run above would still pass if
/// both lanes moved together, and the position of one helper in a list is
/// exactly what this installment changes.
#[test]
fn the_import_list_places_unref_by_the_node_that_needed_it() {
    let metadata = metadata();
    let table = table(&metadata);
    let allocator = vize_s0::Allocator::new();
    let imports = |src: &str| {
        vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &emit_options(&table, true),
        )
        .expect("unref order witness must emit")
        .assembled()
        .lines()
        .find(|line| line.starts_with("import {"))
        .unwrap_or_default()
        .to_string()
    };
    // The loop source is processed before `renderList` is registered.
    assert_eq!(
        imports(r#"<li v-for="i in items" :key="i">{{ i }}</li>"#),
        "import { toDisplayString as _toDisplayString, openBlock as _openBlock, \
         createElementBlock as _createElementBlock, Fragment as _Fragment, \
         unref as _unref, renderList as _renderList } from \"vue\""
    );
    // A const source leaves `renderList` first, with `unref` arriving
    // from the sibling that comes after the loop.
    assert_eq!(
        imports(r#"<ul><li v-for="i in list" :key="i">{{ i }}</li></ul><p>{{ msg }}</p>"#),
        "import { toDisplayString as _toDisplayString, createElementVNode as _createElementVNode, \
         openBlock as _openBlock, createElementBlock as _createElementBlock, \
         Fragment as _Fragment, renderList as _renderList, unref as _unref } from \"vue\""
    );
    // A compound handler wraps its rewritten body in `$event => (…)`; the
    // helper the body needed is still imported.
    assert_eq!(
        imports(r#"<button @click="toggle(1)">x</button>"#),
        "import { openBlock as _openBlock, createElementBlock as _createElementBlock, \
         unref as _unref } from \"vue\""
    );
}
