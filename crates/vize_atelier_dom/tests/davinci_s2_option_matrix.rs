//! P2-11: the production-option **combination** matrix. Every option has
//! its own witness, but a real `<script setup>` SFC with `<style scoped>`
//! compiles with several of them at once, and the emitter's shared
//! decisions — how many pairs a props object holds, whether it goes
//! multiline, which argument a `mergeProps` pair rides — are exactly where
//! options interact. Compared byte-for-byte with the shipped lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_atelier_core::options::{BindingMetadata, BindingType, CodegenMode, CodegenOptions};
use vize_atelier_dom::DomCompilerOptions;
use vize_s1_to_s2::{BindingTable, DomEmitMode, DomEmitOptions};

const SCOPE_ID: &str = "data-v-abc123";

const PRODUCTION_DOM_BATTERY_NAMES: &[&str] = &[
    "class_attr",
    "id_and_class",
    "data_attr",
    "boolean_attr",
    "nested_class",
    "hoisted_class_interp",
    "static_and_dynamic_class",
    "attr_then_object_bind",
    "object_bind_then_attr",
    "class_then_object_bind",
    "static_dynamic_class_then_object",
    "class_object_then_dynamic_class",
    "component_static_class",
    "component_class_and_id",
    "component_static_id_text_slot",
    "component_static_class_text_slot",
    "component_static_two_attrs_text_slot",
    "component_mixed_static_bind_text_slot",
    "component_span_class_slot",
    "component_span_class_text_slot",
    "component_static_tree_with_text",
];

/// Production-only shapes where a cached handler, static/cache hoist and
/// scope pair land in the same props object, or where one of them decides
/// the object's layout.
const PRODUCTION_LAYOUT_BATTERY: &[(&str, &str)] = &[
    ("handler_only", r#"<div @click="count++"></div>"#),
    (
        "handler_and_static",
        r#"<div class="a" @click="count++"></div>"#,
    ),
    (
        "handler_and_bind",
        r#"<div :id="x" @click="count++"></div>"#,
    ),
    ("reference_handler", r#"<div @click="onTap"></div>"#),
    ("let_handler", r#"<div @click="tapLet"></div>"#),
    (
        "handler_with_modifier",
        r#"<div @click.stop="count++"></div>"#,
    ),
    ("key_modifier", r#"<input @keyup.enter="submit">"#),
    ("two_handlers", r#"<div @click="a++" @input="b++"></div>"#),
    (
        "handler_and_class",
        r#"<div :class="cls" @click="count++"></div>"#,
    ),
    (
        "handler_and_spread",
        r#"<div v-bind="obj" @click="count++"></div>"#,
    ),
    ("spread_only", r#"<div v-bind="obj"></div>"#),
    ("spread_and_static", r#"<div class="a" v-bind="obj"></div>"#),
    ("on_spread", r#"<div v-on="handlers"></div>"#),
    ("static_only", r#"<div class="a">x</div>"#),
    ("static_tree", "<div><b>a</b><i>c</i></div>"),
    ("empty", "<div></div>"),
    ("interpolation", "<div>{{ msg }}</div>"),
    ("v_if_handler", r#"<div v-if="ok" @click="count++">x</div>"#),
    (
        "v_for_handler",
        r#"<li v-for="i in items" :key="i" @click="a++">{{ i }}</li>"#,
    ),
    (
        "v_for_static",
        r#"<li v-for="i in items" :key="i"><b>x</b></li>"#,
    ),
    ("v_once", "<div v-once>x</div>"),
    (
        "v_once_then_handler",
        r#"<div><i v-once>x</i><a @click="a++"></a></div>"#,
    ),
    ("v_model", r#"<input v-model="v">"#),
    ("v_model_and_handler", r#"<input v-model="v" @focus="a++">"#),
    ("component", "<MyComp />"),
    ("component_handler", r#"<MyComp @click="count++" />"#),
    ("component_static_prop", r#"<MyComp a="1" />"#),
    (
        "component_slot",
        r#"<MyComp><a @click="a++">x</a></MyComp>"#,
    ),
    (
        "scoped_slot",
        r#"<MyComp v-slot="{ row }"><a @click="go(row)">{{ row }}</a></MyComp>"#,
    ),
    ("slot_outlet", r#"<slot :item="x">fallback</slot>"#),
    ("svg", r#"<svg><path d="M0 0" @click="a++"/></svg>"#),
    (
        "teleport",
        r##"<Teleport to="#a"><div @click="a++">x</div></Teleport>"##,
    ),
    (
        "cached_static_scoped_child",
        r#"<section><Comp :format="fmt"/><span class="badge">ok</span></section>"#,
    ),
    (
        "cached_static_and_dynamic_scoped_child",
        r#"<section><Comp :format="fmt"/><span class="badge" :title="label">ok</span></section>"#,
    ),
    (
        "cached_style_object_scoped_child",
        r#"<section><Comp :format="fmt"/><span :style="{ marginLeft: 8 }">ok</span></section>"#,
    ),
    (
        "cached_static_style_scoped_child",
        r#"<section><Comp :format="fmt"/><span style="margin-left: 8px">ok</span></section>"#,
    ),
];

fn metadata() -> BindingMetadata {
    support::bindings::script_setup_metadata(&[
        ("onTap", BindingType::SetupConst),
        ("tapLet", BindingType::SetupLet),
        ("count", BindingType::SetupRef),
        ("a", BindingType::SetupRef),
        ("b", BindingType::SetupRef),
        ("submit", BindingType::SetupConst),
        ("msg", BindingType::SetupLet),
        ("name", BindingType::SetupLet),
        ("cls", BindingType::SetupLet),
        ("s", BindingType::SetupLet),
        ("foo", BindingType::SetupLet),
        ("x", BindingType::SetupLet),
        ("y", BindingType::SetupLet),
        ("z", BindingType::SetupLet),
        ("obj", BindingType::SetupLet),
        ("handlers", BindingType::SetupLet),
        ("ok", BindingType::SetupRef),
        ("k", BindingType::SetupRef),
        ("ka", BindingType::SetupRef),
        ("kb", BindingType::SetupRef),
        ("kc", BindingType::SetupRef),
        ("list", BindingType::SetupConst),
        ("n", BindingType::SetupConst),
        ("items", BindingType::SetupConst),
        ("h", BindingType::SetupConst),
        ("handler", BindingType::SetupConst),
        ("fmt", BindingType::SetupRef),
        ("label", BindingType::SetupLet),
        ("Comp", BindingType::SetupConst),
        ("Foo", BindingType::SetupConst),
        ("Bar", BindingType::SetupConst),
        ("MyComp", BindingType::SetupConst),
    ])
}

fn table(metadata: &BindingMetadata) -> BindingTable {
    support::bindings::binding_table(metadata)
}

fn production_option_battery() -> Vec<(&'static str, &'static str)> {
    let mut battery =
        Vec::with_capacity(PRODUCTION_DOM_BATTERY_NAMES.len() + PRODUCTION_LAYOUT_BATTERY.len());
    battery.extend(
        support::battery::dom::DOM_BATTERY
            .iter()
            .copied()
            .filter(|(name, _)| PRODUCTION_DOM_BATTERY_NAMES.contains(name)),
    );
    assert_eq!(
        battery.len(),
        PRODUCTION_DOM_BATTERY_NAMES.len(),
        "every selected shared DOM battery case must exist"
    );
    battery.extend(PRODUCTION_LAYOUT_BATTERY.iter().copied());
    battery
}

/// `cache_handlers` + `scope_id` with nothing else: the pair of options
/// that share a props object most often.
#[test]
fn cached_handlers_and_scope_id_agree_with_the_shipped_lane() {
    let battery = production_option_battery();
    support::assert_s2_matches_shipped_with_options(
        &battery,
        &DomCompilerOptions {
            cache_handlers: true,
            scope_id: Some(SCOPE_ID.into()),
            ..Default::default()
        },
        &CodegenOptions::default(),
        &DomEmitOptions {
            cache_handlers: true,
            scope_id: Some(SCOPE_ID),
            ..DomEmitOptions::DEFAULT
        },
    );
}

#[test]
fn disabled_static_hoists_agree_with_the_shipped_lane() {
    let battery = production_option_battery();
    support::assert_s2_matches_shipped_with_options(
        &battery,
        &DomCompilerOptions {
            hoist_static: false,
            ..Default::default()
        },
        &CodegenOptions::default(),
        &DomEmitOptions {
            hoist_static: false,
            ..DomEmitOptions::DEFAULT
        },
    );
}

/// The configuration a real `<script setup>` SFC with `<style scoped>`
/// compiles under: module mode, prefixed identifiers, binding metadata,
/// cached handlers and a scope id, all at once.
#[test]
fn the_script_setup_configuration_agrees_with_the_shipped_lane() {
    let metadata = metadata();
    let table = table(&metadata);
    let battery = production_option_battery();
    support::assert_s2_matches_shipped_with_options(
        &battery,
        &DomCompilerOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: true,
            cache_handlers: true,
            scope_id: Some(SCOPE_ID.into()),
            binding_metadata: Some(metadata.clone()),
            ..Default::default()
        },
        &CodegenOptions::default(),
        &DomEmitOptions {
            mode: DomEmitMode::Module,
            prefix_identifiers: true,
            cache_handlers: true,
            scope_id: Some(SCOPE_ID),
            bindings: Some(&table),
            ..DomEmitOptions::DEFAULT
        },
    );
}

#[test]
fn inline_component_static_props_stay_in_the_option_matrix() {
    let metadata = metadata();
    let table = table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        &[
            ("component_static_prop", r#"<MyComp a="1" />"#),
            (
                "teleport_static_prop",
                r##"<Teleport to="#a"><div @click="a++">x</div></Teleport>"##,
            ),
        ],
        &DomCompilerOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: true,
            inline: true,
            cache_handlers: true,
            scope_id: Some(SCOPE_ID.into()),
            binding_metadata: Some(metadata.clone()),
            ..Default::default()
        },
        &CodegenOptions::default(),
        &DomEmitOptions {
            mode: DomEmitMode::Module,
            prefix_identifiers: true,
            inline: true,
            cache_handlers: true,
            scope_id: Some(SCOPE_ID),
            bindings: Some(&table),
            ..DomEmitOptions::DEFAULT
        },
    );
}

/// The same, inlined into `setup()`, where setup bindings are read off the
/// closure and a `SetupConst` handler stops being cached.
#[test]
fn the_inline_script_setup_configuration_agrees_with_the_shipped_lane() {
    let metadata = metadata();
    let table = table(&metadata);
    let battery = production_option_battery();
    support::assert_s2_matches_shipped_with_options(
        &battery,
        &DomCompilerOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: true,
            inline: true,
            cache_handlers: true,
            scope_id: Some(SCOPE_ID.into()),
            binding_metadata: Some(metadata.clone()),
            ..Default::default()
        },
        &CodegenOptions::default(),
        &DomEmitOptions {
            mode: DomEmitMode::Module,
            prefix_identifiers: true,
            inline: true,
            cache_handlers: true,
            scope_id: Some(SCOPE_ID),
            bindings: Some(&table),
            ..DomEmitOptions::DEFAULT
        },
    );
}
