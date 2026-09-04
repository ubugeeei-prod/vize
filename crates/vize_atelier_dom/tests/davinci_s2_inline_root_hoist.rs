//! P2-11 installment 90 witness: the **`inline` root prop-hoist arm**.
//!
//! `hoist_static_inner`'s `NotStatic` element arm carries a disjunct no
//! other option reaches — `is_root && ctx.options.inline &&
//! has_only_native_element_descendants(el)` — so an inlined render
//! function hoists the static props of a dynamic root whose subtree is
//! all native elements and text, where the same template keeps them
//! inline in every other mode. Compared byte-for-byte with the shipped
//! lane, in both directions: the same battery must *not* hoist with
//! `inline` off.

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
    (
        "empty_children_handler",
        r#"<div class="a" @click="handler"></div>"#,
    ),
    (
        "empty_children_bind",
        r#"<div class="a" :id="count"></div>"#,
    ),
    (
        "nested_dynamic_prop",
        r#"<div class="a"><em :id="count">x</em></div>"#,
    ),
    (
        "nested_handler",
        r#"<div class="a"><em @click="handler">x</em></div>"#,
    ),
    (
        "nested_static_ok",
        r#"<div class="a"><em>{{ count }}</em></div>"#,
    ),
    (
        "root_vif_child",
        r#"<div class="a"><em v-if="count">x</em></div>"#,
    ),
    ("root_component_child", r#"<div class="a"><MyComp /></div>"#),
    (
        "two_static_props",
        r#"<div class="a" id="b" @click="handler"></div>"#,
    ),
    (
        "self_closing_input",
        r#"<input class="a" @input="handler">"#,
    ),
    (
        "root_directive_own",
        r#"<div class="a" v-show="count"></div>"#,
    ),
    (
        "component_root_static",
        r#"<MyComp class="a"><em :id="count">x</em></MyComp>"#,
    ),
    (
        "component_root_empty",
        r#"<MyComp class="a" @click="handler" />"#,
    ),
    (
        "multi_root",
        r#"<div class="a" @click="handler"></div><p class="b" @click="handler"></p>"#,
    ),
    (
        "nested_root_only",
        r#"<section><div class="a" @click="handler"></div></section>"#,
    ),
    ("root_slot_child", r#"<div class="a"><slot /></div>"#),
    ("root_comment_child", r#"<div class="a"><!-- c --></div>"#),
    (
        "root_text_and_dyn",
        r#"<div class="a">t<em :id="count">x</em></div>"#,
    ),
    ("svg_root", r#"<svg class="a" @click="handler"></svg>"#),
    (
        "vfor_root",
        r#"<li v-for="i in items" class="a" @click="handler"></li>"#,
    ),
    (
        "vif_root",
        r#"<div v-if="count" class="a" @click="handler"></div>"#,
    ),
    (
        "vif_else_root",
        r#"<div v-if="count" class="a"><em :id="count">x</em></div><p v-else class="b"><em :id="count">y</em></p>"#,
    ),
    (
        "root_ref_attr",
        r#"<div class="a" ref="el"><em :id="count">x</em></div>"#,
    ),
    (
        "nested_same_shape",
        r#"<section><div class="a"><em :id="count">x</em></div></section>"#,
    ),
    (
        "root_component_descendant",
        r#"<div class="a"><MyComp :id="count" /></div>"#,
    ),
    (
        "root_slot_descendant",
        r#"<div class="a"><em><slot /></em></div>"#,
    ),
    (
        "root_vfor_descendant",
        r#"<div class="a"><em v-for="i in items" :key="i">{{ i }}</em></div>"#,
    ),
    (
        "root_deep_native",
        r#"<div class="a"><em><b><i :id="count">x</i></b></em></div>"#,
    ),
    (
        "root_comment_descendant",
        r#"<div class="a"><em :id="count"><!-- c --></em></div>"#,
    ),
    (
        "dynamic_component_root",
        r#"<component :is="count" class="a"><em :id="count">x</em></component>"#,
    ),
    (
        "transition_root",
        r#"<Transition class="a"><em :id="count">x</em></Transition>"#,
    ),
    (
        "keepalive_root",
        r#"<KeepAlive class="a"><em :id="count">x</em></KeepAlive>"#,
    ),
    ("root_vhtml", r#"<div class="a" v-html="msg"></div>"#),
    ("root_vtext", r#"<div class="a" v-text="msg"></div>"#),
    (
        "root_two_static_one_dyn",
        r#"<div class="a" id="i" :title="count"><em :id="count">x</em></div>"#,
    ),
    (
        "root_style_attr",
        r#"<div class="a" style="color:red"><em :id="count">x</em></div>"#,
    ),
    (
        "root_svg_ns",
        r#"<svg class="a"><circle :cx="count" /></svg>"#,
    ),
    (
        "root_template_vfor",
        r#"<template v-for="i in items"><div class="a"><em :id="i">x</em></div></template>"#,
    ),
    (
        "root_slot_outlet_child",
        r#"<div class="a"><slot name="x" /></div>"#,
    ),
    (
        "root_named_slot_component",
        r#"<MyComp class="a"><template #x><em :id="count">y</em></template></MyComp>"#,
    ),
    (
        "root_component_text_slot",
        r#"<MyComp class="a">text</MyComp>"#,
    ),
    (
        "root_component_dyn_prop",
        r#"<MyComp class="a" :id="count"><em :id="count">x</em></MyComp>"#,
    ),
    (
        "root_v_once",
        r#"<div class="a" v-once><em :id="count">x</em></div>"#,
    ),
    (
        "root_v_memo",
        r#"<div class="a" v-memo="[count]"><em :id="count">x</em></div>"#,
    ),
    (
        "root_custom_directive",
        r#"<div class="a" v-focus><em :id="count">x</em></div>"#,
    ),
    (
        "root_key_only",
        r#"<div key="k"><em :id="count">x</em></div>"#,
    ),
    (
        "root_interp_and_elem",
        r#"<div class="a">{{ count }}<em :id="count">x</em></div>"#,
    ),
    (
        "comp_root_custom_dir",
        r#"<MyComp class="a" v-focus><em :id="count">x</em></MyComp>"#,
    ),
    (
        "comp_root_vslot",
        r#"<MyComp class="a" v-slot="s"><em :id="count">{{ s }}</em></MyComp>"#,
    ),
    (
        "comp_root_vif",
        r#"<MyComp v-if="count" class="a"><em :id="count">x</em></MyComp>"#,
    ),
    (
        "comp_root_vfor",
        r#"<MyComp v-for="i in items" :key="i" class="a"><em :id="i">x</em></MyComp>"#,
    ),
    (
        "comp_root_vshow",
        r#"<MyComp class="a" v-show="count"><em :id="count">x</em></MyComp>"#,
    ),
    (
        "comp_root_vmodel",
        r#"<MyComp class="a" v-model="count"><em :id="count">x</em></MyComp>"#,
    ),
    (
        "comp_root_teleport",
        r##"<Teleport to="#a" class="b"><em :id="count">x</em></Teleport>"##,
    ),
    (
        "comp_root_suspense",
        r#"<Suspense class="a"><em :id="count">x</em></Suspense>"#,
    ),
    (
        "comp_root_transition_group",
        r#"<TransitionGroup class="a"><em :id="count" key="k">x</em></TransitionGroup>"#,
    ),
    (
        "comp_root_dyn_is",
        r#"<component is="MyComp" class="a"><em :id="count">x</em></component>"#,
    ),
    ("comp_root_no_children", r#"<MyComp class="a" />"#),
    (
        "comp_root_slot_outlet_child",
        r#"<MyComp class="a"><slot /></MyComp>"#,
    ),
    (
        "multi_root_component",
        r#"<MyComp class="a"><em :id="count">x</em></MyComp><p class="b"><em :id="count">y</em></p>"#,
    ),
    (
        "multi_root_builtin",
        r#"<Transition class="a"><em :id="count">x</em></Transition><p class="b"><em :id="count">y</em></p>"#,
    ),
    (
        "elem_root_custom_dir_dyn",
        r#"<div class="a" v-focus><em :id="count">x</em></div>"#,
    ),
    (
        "elem_root_vfor",
        r#"<div v-for="i in items" :key="i" class="a"><em :id="i">x</em></div>"#,
    ),
    (
        "elem_root_template_vfor_single",
        r#"<template v-for="i in items" :key="i"><div class="a"><em :id="i">x</em></div></template>"#,
    ),
    (
        "elem_root_vif_chain3",
        r#"<div v-if="count" class="a"><em :id="count">x</em></div><div v-else-if="msg" class="b"><em :id="count">y</em></div><div v-else class="c"><em :id="count">z</em></div>"#,
    ),
    (
        "nested_component_inside",
        r#"<div class="a"><em><MyComp /></em></div>"#,
    ),
    (
        "root_prop_bind_static",
        r#"<div :data-x="'s'"><em :id="count">x</em></div>"#,
    ),
    (
        "root_class_only",
        r#"<div class="a"><em :id="count">x</em></div>"#,
    ),
    (
        "root_id_and_dyn_class",
        r#"<div id="i" :class="count"><em :id="count">x</em></div>"#,
    ),
];

fn metadata() -> BindingMetadata {
    let entries: &[(&str, BindingType)] = &[
        ("count", BindingType::SetupRef),
        ("msg", BindingType::SetupLet),
        ("handler", BindingType::SetupConst),
        ("title", BindingType::Props),
        ("MyComp", BindingType::SetupConst),
        ("vFocus", BindingType::SetupConst),
        ("items", BindingType::SetupConst),
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

fn emit_options<'a>(table: &'a BindingTable, inline: bool) -> DomEmitOptions<'a> {
    DomEmitOptions {
        mode: DomEmitMode::Module,
        prefix_identifiers: true,
        inline,
        bindings: Some(table),
        ..DomEmitOptions::DEFAULT
    }
}

#[test]
fn inline_root_hoist_matches_the_shipped_dom_lane() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, true),
        &CodegenOptions::default(),
        &emit_options(&table, true),
    );
}

/// The same battery with `inline` off. The arm is the only difference
/// between the two runs, so a lane that hoisted unconditionally — or
/// read the `native_descendants` fact without the option — would pass
/// the run above and fail this one.
#[test]
fn the_same_battery_keeps_its_root_props_inline_without_the_option() {
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
fn the_arm_is_what_moves_the_root_props_into_the_preamble() {
    let metadata = metadata();
    let table = table(&metadata);
    let allocator = vize_s0::Allocator::new();
    let emit = |src: &str, inline: bool| {
        vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &emit_options(&table, inline),
        )
        .expect("inline root hoist witness must emit")
        .assembled()
    };
    let render_line = |src: &str, inline: bool| {
        emit(src, inline)
            .lines()
            .find(|line| line.contains("return "))
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    let hoist_line = |src: &str, inline: bool| {
        emit(src, inline)
            .lines()
            .find(|line| line.starts_with("const _hoisted_"))
            .unwrap_or_default()
            .to_string()
    };

    // A dynamic native root with static props and an all-native subtree:
    // the props move to the preamble under `inline`, and stay in the call
    // without it.
    let native_root = r#"<div class="a"><em :id="count">x</em></div>"#;
    assert_eq!(
        hoist_line(native_root, true),
        "const _hoisted_1 = { class: \"a\" }"
    );
    assert_eq!(hoist_line(native_root, false), "");
    assert_eq!(
        render_line(native_root, true),
        "return (_openBlock(), _createElementBlock(\"div\", _hoisted_1, ["
    );
    assert_eq!(
        render_line(native_root, false),
        "return (_openBlock(), _createElementBlock(\"div\", { class: \"a\" }, ["
    );

    // The arm reads the *root* position: the same element one level down
    // keeps its props inline under `inline` too.
    let nested = r#"<section><div class="a"><em :id="count">x</em></div></section>"#;
    assert_eq!(hoist_line(nested, true), "");

    // A component descendant is not a native descendant, so the arm does
    // not fire even at the root.
    let component_descendant = r#"<div class="a"><MyComp :id="count" /></div>"#;
    assert_eq!(hoist_line(component_descendant, true), "");

    // A component root reaches the arm the same way its element sibling
    // does — including the builtin helpers, whose ordinary hoist gate is
    // reconstructed from a codegen shape the arm sits in front of.
    assert_eq!(
        hoist_line(r#"<MyComp class="a"><em :id="count">x</em></MyComp>"#, true),
        "const _hoisted_1 = { class: \"a\" }"
    );
    assert_eq!(
        hoist_line(
            r#"<Transition class="a"><em :id="count">x</em></Transition>"#,
            true
        ),
        "const _hoisted_1 = { class: \"a\" }"
    );
    assert_eq!(
        hoist_line(
            r#"<KeepAlive class="a"><em :id="count">x</em></KeepAlive>"#,
            true
        ),
        "const _hoisted_1 = { class: \"a\" }"
    );
}
