//! P2-11 installment 90 witness: the **`inline` root prop-hoist arm**
//! over *component* roots, the sibling of
//! `davinci_s2_inline_root_hoist`.
//!
//! A component's own hoist gate is reconstructed from the codegen shape
//! its props end up in — slots, builtin helpers, array children — and
//! the shipped transform's root arm sits in front of all of it, so the
//! arm is a disjunct of that gate rather than a term inside it. Compared
//! byte-for-byte with the shipped lane, in both directions.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_atelier_core::options::{BindingMetadata, BindingType, CodegenMode, CodegenOptions};
use vize_atelier_dom::DomCompilerOptions;
use vize_s1_to_s2::{BindingTable, DomEmitMode, DomEmitOptions};

const BATTERY: &[(&str, &str)] = &[
    ("root_component_child", r#"<div class="a"><MyComp /></div>"#),
    (
        "component_root_static",
        r#"<MyComp class="a"><em :id="count">x</em></MyComp>"#,
    ),
    (
        "component_root_empty",
        r#"<MyComp class="a" @click="handler" />"#,
    ),
    (
        "root_component_descendant",
        r#"<div class="a"><MyComp :id="count" /></div>"#,
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
];

fn metadata() -> BindingMetadata {
    support::bindings::script_setup_metadata(&[
        ("count", BindingType::SetupRef),
        ("msg", BindingType::SetupLet),
        ("handler", BindingType::SetupConst),
        ("title", BindingType::Props),
        ("MyComp", BindingType::SetupConst),
        ("vFocus", BindingType::SetupConst),
        ("items", BindingType::SetupConst),
    ])
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
    let table = support::bindings::binding_table(&metadata);
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
    let table = support::bindings::binding_table(&metadata);
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
    let table = support::bindings::binding_table(&metadata);
    let allocator = vize_s0::Allocator::new();
    let hoist_line = |src: &str, inline: bool| {
        vize_s1_to_s2::emit_dom_source_with_options(
            &allocator,
            src,
            vize_s1_to_s2::LegacyCaps::VUE3,
            &emit_options(&table, inline),
        )
        .expect("inline root hoist witness must emit")
        .assembled()
        .lines()
        .find(|line| line.starts_with("const _hoisted_"))
        .unwrap_or_default()
        .to_string()
    };

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
