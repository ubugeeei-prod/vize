//! P2-11 installment 92 witness: **inline template refs**.
//!
//! An inlined render function keeps its template refs as the setup
//! bindings they name: `ref="foo"` on a writable setup binding becomes
//! the `ref_key: "foo", ref: foo` pair, so the runtime's `setRef` writes
//! back into `instance.refs` — which is what `useTemplateRef` reads. The
//! same element's `NEED_PATCH` flag survives alongside `NEED_HYDRATION`,
//! a combination only an inlined render function reaches. Compared
//! byte-for-byte with the shipped lane, in both directions.

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
    // The three writable setup kinds take the pair.
    ("ref_to_setup_ref", r#"<div ref="elRef"></div>"#),
    ("ref_to_setup_let", r#"<div ref="looseRef"></div>"#),
    ("ref_to_setup_maybe_ref", r#"<div ref="maybeRef"></div>"#),
    // Every other binding kind, and an unknown name, keep the string.
    ("ref_to_setup_const", r#"<div ref="constRef"></div>"#),
    ("ref_to_reactive_const", r#"<div ref="state"></div>"#),
    ("ref_to_prop", r#"<div ref="title"></div>"#),
    ("ref_to_literal_const", r#"<div ref="LIMIT"></div>"#),
    ("ref_to_unknown", r#"<div ref="unknownRef"></div>"#),
    // With siblings, so the props object goes multiline.
    ("ref_with_class", r#"<div ref="elRef" class="a"></div>"#),
    (
        "ref_with_dynamic_prop",
        r#"<div ref="elRef" :id="count"></div>"#,
    ),
    // `ref_for` precedes the pair, as it does the plain attribute.
    (
        "ref_in_v_for",
        r#"<li v-for="i in items" :key="i" ref="elRef"></li>"#,
    ),
    (
        "ref_in_template_v_for",
        r#"<template v-for="i in items" :key="i"><li ref="elRef"></li></template>"#,
    ),
    // The `v-if` branch and component prop objects are their own codegen
    // paths in the shipped lane, with the same rule in each.
    ("ref_in_v_if", r#"<p v-if="count" ref="elRef"></p>"#),
    (
        "ref_in_v_if_else",
        r#"<p v-if="count" ref="elRef"></p><p v-else ref="looseRef"></p>"#,
    ),
    ("ref_on_component", r#"<MyComp ref="elRef" />"#),
    (
        "ref_on_component_with_slot",
        r#"<MyComp ref="elRef"><span>x</span></MyComp>"#,
    ),
    ("ref_on_builtin", r#"<Transition ref="elRef" />"#),
    // A dynamic `:ref` is an expression, not a name to look up.
    ("dynamic_ref_bind", r#"<div :ref="elRef"></div>"#),
    (
        "dynamic_ref_bind_expr",
        r#"<div :ref="el => elRef = el"></div>"#,
    ),
    // `NEED_PATCH` from the ref plus `NEED_HYDRATION` from a handler the
    // inline lane reads as constant: the shipped lane keeps both.
    (
        "ref_with_constant_handler",
        r#"<div ref="elRef" @scroll="handler"></div>"#,
    ),
    (
        "ref_with_constant_handler_and_children",
        r#"<div ref="elRef" @scroll="handler"><span>x</span></div>"#,
    ),
    (
        "string_ref_with_constant_handler",
        r#"<div ref="unknownRef" @scroll="handler"></div>"#,
    ),
    (
        "ref_with_dynamic_handler",
        r#"<div ref="elRef" @scroll="other"></div>"#,
    ),
    // A ref beside `v-show` / a custom directive, the other NEED_PATCH
    // sources.
    (
        "ref_with_v_show",
        r#"<div ref="elRef" v-show="count"></div>"#,
    ),
    ("ref_with_directive", r#"<div ref="elRef" v-focus></div>"#),
    (
        "ref_with_v_model",
        r#"<input ref="elRef" v-model="looseRef">"#,
    ),
];

fn metadata() -> BindingMetadata {
    let entries: &[(&str, BindingType)] = &[
        ("elRef", BindingType::SetupRef),
        ("looseRef", BindingType::SetupLet),
        ("maybeRef", BindingType::SetupMaybeRef),
        ("constRef", BindingType::SetupConst),
        ("state", BindingType::SetupReactiveConst),
        ("LIMIT", BindingType::LiteralConst),
        ("title", BindingType::Props),
        ("count", BindingType::SetupRef),
        ("handler", BindingType::SetupConst),
        ("items", BindingType::SetupConst),
        ("MyComp", BindingType::SetupConst),
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
fn inline_template_refs_match_the_shipped_dom_lane() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, true),
        &CodegenOptions::default(),
        &emit_options(&table, true),
    );
}

/// The pair is an inline-only spelling: with the option off the same
/// battery keeps every `ref` as its authored string.
#[test]
fn the_same_battery_keeps_string_refs_without_the_inline_option() {
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
fn the_pair_and_its_patch_flag_are_inline_only() {
    let metadata = metadata();
    let table = table(&metadata);
    let allocator = vize_s0::Allocator::new();
    let render = |src: &str, inline: bool| {
        vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &emit_options(&table, inline),
        )
        .expect("template ref witness must emit")
        .assembled()
        .lines()
        .find(|line| line.contains("return "))
        .unwrap_or_default()
        .trim()
        .to_string()
    };
    assert_eq!(
        render(r#"<div ref="elRef"></div>"#, true),
        "return (_openBlock(), _createElementBlock(\"div\", { ref_key: \"elRef\", ref: elRef }, null, 512 /* NEED_PATCH */))"
    );
    assert_eq!(
        render(r#"<div ref="elRef"></div>"#, false),
        "return (_openBlock(), _createElementBlock(\"div\", { ref: \"elRef\" }, null, 512 /* NEED_PATCH */))"
    );
    // A binding the script cannot rebind keeps the string.
    assert_eq!(
        render(r#"<div ref="constRef"></div>"#, true),
        "return (_openBlock(), _createElementBlock(\"div\", { ref: \"constRef\" }, null, 512 /* NEED_PATCH */))"
    );
    // `NEED_PATCH` survives beside `NEED_HYDRATION` — the ref and a
    // handler the inline lane reads as constant, with no prop flag
    // between them. The flag closes the multiline call, so read that
    // line rather than the `return`.
    let flag_line = |src: &str, inline: bool| {
        vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &emit_options(&table, inline),
        )
        .expect("template ref witness must emit")
        .assembled()
        .lines()
        .find(|line| line.contains("/*") && line.contains("*/)"))
        .unwrap_or_default()
        .trim()
        .to_string()
    };
    assert_eq!(
        flag_line(r#"<div ref="unknownRef" @scroll="handler"></div>"#, true),
        "}, null, 544 /* NEED_HYDRATION, NEED_PATCH */))"
    );
}
