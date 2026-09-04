//! P2-11 installment 93 witness: **`is_const_handler` on the merged
//! props path**.
//!
//! A `v-on` handler that is nothing but a constant setup binding never
//! changes, so the shipped lane leaves it out of `dynamicProps` and off
//! the `PROPS` flag. That rule is one arm of a single per-prop loop,
//! which runs whether or not the element also carries an object
//! `v-bind` and ends up in `mergeProps` / `FULL_PROPS`. The port
//! reconstructed the loop twice and only carried the rule into one of
//! them. Compared byte-for-byte with the shipped lane.

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
    // The plain path already carried the rule; it is the control.
    ("plain_const_handler", r#"<div @click="handler"></div>"#),
    ("plain_dynamic_handler", r#"<div @click="msg"></div>"#),
    // The merged path: an object `v-bind` puts the element in
    // `mergeProps` with `FULL_PROPS`, and the handler still decides its
    // own `dynamicProps` entry.
    (
        "merged_const_handler",
        r#"<div v-bind="attrs" @click="handler"></div>"#,
    ),
    (
        "merged_literal_const_handler",
        r#"<div v-bind="attrs" @click="LIMIT"></div>"#,
    ),
    (
        "merged_dynamic_handler",
        r#"<div v-bind="attrs" @click="msg"></div>"#,
    ),
    (
        "merged_ref_handler",
        r#"<div v-bind="attrs" @click="count"></div>"#,
    ),
    (
        "merged_unknown_handler",
        r#"<div v-bind="attrs" @click="other"></div>"#,
    ),
    (
        "merged_call_handler",
        r#"<div v-bind="attrs" @click="handler(1)"></div>"#,
    ),
    (
        "merged_two_handlers",
        r#"<div v-bind="attrs" @click="handler" @input="msg"></div>"#,
    ),
    (
        "merged_const_and_bind",
        r#"<div v-bind="attrs" :id="count" @click="handler"></div>"#,
    ),
    // Components take the same loop with component key casing.
    (
        "component_merged_const_handler",
        r#"<MyComp v-bind="attrs" @finish="handler" />"#,
    ),
    (
        "component_merged_custom_event",
        r#"<MyComp v-bind="attrs" @some-event="handler" />"#,
    ),
    (
        "component_merged_dynamic",
        r#"<MyComp v-bind="attrs" @finish="msg" />"#,
    ),
    // Modifiers, key modifiers and native hydration events still read
    // their own rules around the const one.
    (
        "merged_const_handler_modifier",
        r#"<div v-bind="attrs" @click.stop="handler"></div>"#,
    ),
    (
        "merged_const_handler_key",
        r#"<input v-bind="attrs" @keyup.enter="handler">"#,
    ),
    (
        "merged_const_handler_capture",
        r#"<div v-bind="attrs" @click.capture="handler"></div>"#,
    ),
    (
        "merged_const_handler_custom_event",
        r#"<div v-bind="attrs" @my-event="handler"></div>"#,
    ),
    // A `v-for` item and a `v-if` branch keep their own key handling
    // around the merged object.
    (
        "merged_in_v_for",
        r#"<li v-for="i in items" :key="i" v-bind="attrs" @click="handler"></li>"#,
    ),
    (
        "merged_in_v_if",
        r#"<p v-if="count" v-bind="attrs" @click="handler"></p>"#,
    ),
    // `v-model` and `v-html` entries on the merged object are unchanged.
    (
        "merged_with_model",
        r#"<input v-bind="attrs" v-model="msg" @click="handler">"#,
    ),
    (
        "merged_with_html",
        r#"<div v-bind="attrs" v-html="msg" @click="handler"></div>"#,
    ),
];

fn metadata() -> BindingMetadata {
    let entries: &[(&str, BindingType)] = &[
        ("handler", BindingType::SetupConst),
        ("LIMIT", BindingType::LiteralConst),
        ("msg", BindingType::SetupLet),
        ("count", BindingType::SetupRef),
        ("attrs", BindingType::SetupConst),
        ("items", BindingType::SetupConst),
        ("MyComp", BindingType::SetupConst),
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
fn merged_props_read_the_same_const_handler_rule() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, true),
        &CodegenOptions::default(),
        &emit_options(&table, true),
    );
}

/// The rule only fires where the transform leaves the binding bare, so
/// the same battery must agree with `inline` off — where every handler
/// is a `_ctx` member and none of them is constant.
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

/// The `dynamicProps` array itself, pinned on both sides of the option.
#[test]
fn a_constant_handler_leaves_the_merged_dynamic_props_array() {
    let metadata = metadata();
    let table = table(&metadata);
    let allocator = vize_s0::Allocator::new();
    let dynamic_props = |src: &str, inline: bool| {
        let assembled = vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &emit_options(&table, inline),
        )
        .expect("const handler witness must emit")
        .assembled();
        let line = assembled
            .lines()
            .find(|line| line.contains("FULL_PROPS"))
            .unwrap_or_default()
            .trim()
            .to_string();
        line
    };
    // The constant handler is gone from the array; the dynamic one stays.
    assert_eq!(
        dynamic_props(
            r#"<div v-bind="attrs" :id="count" @click="handler"></div>"#,
            true
        ),
        "}), null, 16 /* FULL_PROPS */, [\"id\"]))"
    );
    assert_eq!(
        dynamic_props(
            r#"<div v-bind="attrs" :id="count" @click="msg"></div>"#,
            true
        ),
        "}), null, 16 /* FULL_PROPS */, [\"id\", \"onClick\"]))"
    );
    // Without `inline` the same handler is a `_ctx` member, so it stays.
    assert_eq!(
        dynamic_props(
            r#"<div v-bind="attrs" :id="count" @click="handler"></div>"#,
            false
        ),
        "}), null, 16 /* FULL_PROPS */, [\"id\", \"onClick\"]))"
    );
}
