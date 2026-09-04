//! P2-11 installment 99 witness: **a component props object the
//! prefixed constant rule hoists**.
//!
//! The hoist pass publishes `props_hoistable` under the *unprefixed*
//! constant rule and stays option-free by its own contract, so any
//! identifier reference makes an expression dynamic there — allowed
//! globals included. The shipped lane reads an expression's constness off
//! `processExpression`, which only runs with `prefix_identifiers`, so a
//! prop the pass called dynamic is asked again against the same
//! runtime-dependency rule `normalizeStyle` already uses.
//!
//! That is how `:range="[new Date(2019, 2, 4), new Date(2019, 2, 24)]"`
//! reaches `const _hoisted_1 = { … }`: its only free name is `Date`.
//!
//! **This installment widens the component path only.** The shipped lane
//! hoists a plain element's props object under the same rule, but gates
//! it on the subtree — measured, `<div :data-a="[new Date(0)]"></div>`
//! hoists while
//! `<div :data-a="[new Date(0)]"><span :id="msg"></span></div>` does not
//! — and widening the element path without that gate regressed the
//! prefixed and bindings corpus lanes from 0 to 2. The element rule is
//! its own installment; the batteries here stay on the component side
//! and on the elements both lanes already agree about. Compared
//! byte-for-byte with the shipped lane.

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
    // The shape that hoists: a component prop whose only free name is an
    // allowed global.
    (
        "component_global_constructor_array",
        r#"<MyComp :range="[new Date(2019, 2, 4), new Date(2019, 2, 24)]" />"#,
    ),
    ("component_global_call", r#"<MyComp :at="Date.now()" />"#),
    ("component_global_member", r#"<MyComp :pi="Math.PI" />"#),
    (
        "component_two_global_props",
        r#"<MyComp :a="Math.PI" :b="[new Date(0)]" />"#,
    ),
    // A reactive read is not constant on either.
    ("component_reactive_prop", r#"<MyComp :n="count" />"#),
    ("element_reactive_prop", r#"<div :data-n="count"></div>"#),
    // A `v-for` alias is bound by the render function, never constant.
    (
        "component_prop_reads_for_alias",
        r#"<MyComp v-for="i in items" :key="i" :n="[new Date(i)]" />"#,
    ),
    // `ref` and `class` stay excluded from the hoist key set.
    (
        "component_ref_prop",
        r#"<MyComp :ref="'r'" :n="Math.PI" />"#,
    ),
    (
        "component_class_prop",
        r#"<MyComp :class="'c'" :n="Math.PI" />"#,
    ),
    // A constant script binding is constant only where the prefixer
    // leaves it bare, which is the inlined render function.
    ("component_const_binding_prop", r#"<MyComp :n="theme" />"#),
];

fn metadata() -> BindingMetadata {
    let entries: &[(&str, BindingType)] = &[
        ("count", BindingType::SetupRef),
        ("items", BindingType::SetupRef),
        ("theme", BindingType::SetupConst),
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
fn prefixed_component_props_hoist_like_the_shipped_lane() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, true),
        &CodegenOptions::default(),
        &emit_options(&table, true),
    );
}

/// The rule is `prefix_identifiers`, not `inline`: the globals cases
/// hoist without it too, and only the constant *binding* case turns on
/// the inlined spelling.
#[test]
fn the_same_battery_agrees_without_inline() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, false),
        &CodegenOptions::default(),
        &emit_options(&table, false),
    );
}

/// The asymmetry this installment creates, pinned: the same expression
/// hoists on a component and — for now — does not on an element, which
/// is the known gap the module docs name.
#[test]
fn only_the_component_path_takes_the_widened_rule() {
    let metadata = metadata();
    let table = table(&metadata);
    let allocator = vize_s0::Allocator::new();
    let hoists = |src: &str| {
        vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &emit_options(&table, true),
        )
        .expect("prefixed component props witness must emit")
        .assembled()
        .lines()
        .filter(|line| line.starts_with("const _hoisted_"))
        .map(|line| line.trim().to_string())
        .collect::<Vec<_>>()
    };
    assert_eq!(
        hoists(r#"<MyComp :range="[new Date(2019, 2, 4), new Date(2019, 2, 24)]" />"#),
        vec!["const _hoisted_1 = { range: [new Date(2019, 2, 4), new Date(2019, 2, 24)] }".to_string()]
    );
    // The element path is deliberately untouched here: it still answers
    // from the pass's unprefixed rule.
    assert_eq!(
        hoists(r#"<div :data-range="[new Date(2019, 2, 4)]"></div>"#),
        Vec::<String>::new()
    );
}
