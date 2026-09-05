//! P2-11: production-option families for `v-model` and `<slot>` outlets.
//! The combination matrix covers broad DOM shapes; this witness keeps the
//! option bundle pinned on two late high-churn families whose props,
//! handlers, dynamic keys, cache slots and scope pairs interact.

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

const MODEL_FAMILY: &[(&str, &str)] = &[
    ("model_text_trim", r#"<input v-model.trim="msg">"#),
    (
        "model_checkbox_handler",
        r#"<input type="checkbox" v-model="checked" @change="track(checked)">"#,
    ),
    (
        "model_select_vfor",
        r#"<select v-model="choice"><option v-for="item in items" :key="item.id" :value="item.value">{{ item.label }}</option></select>"#,
    ),
    (
        "model_component_named",
        r#"<MyComp v-model:title="title" />"#,
    ),
    (
        "model_component_dynamic_modifier",
        r#"<MyComp v-model:[field].trim="msg" />"#,
    ),
    (
        "model_component_spread",
        r#"<MyComp v-bind="modelProps" v-model="msg" />"#,
    ),
    (
        "model_listener_order",
        r#"<MyComp @update:modelValue="track" v-model="msg" />"#,
    ),
    (
        "model_component_vfor",
        r#"<MyComp v-for="item in items" v-model="item.value" :key="item.id" />"#,
    ),
];

const OUTLET_FAMILY: &[(&str, &str)] = &[
    (
        "outlet_dynamic_name_props",
        r#"<slot :name="slotName" foo="bar" :item="row">fallback {{ msg }}</slot>"#,
    ),
    (
        "outlet_dynamic_event_modifier",
        r#"<slot @[event].once.capture="handler" :[propKey]="value"></slot>"#,
    ),
    (
        "outlet_spread_mixed",
        r#"<slot v-bind="slotProps" @pick="choose(row)" v-on="listeners"></slot>"#,
    ),
    (
        "outlet_forwarded_scoped",
        r#"<Bar v-slot="{ row }"><Foo><slot @[row.event]="row.handler"></slot></Foo></Bar>"#,
    ),
    (
        "outlet_vfor_dynamic_event",
        r#"<slot v-for="item in items" @[item.event]="item.handler">x</slot>"#,
    ),
    ("outlet_style_prop", r#"<slot :style="{ color }"></slot>"#),
];

fn metadata() -> BindingMetadata {
    support::bindings::script_setup_metadata(&[
        ("msg", BindingType::SetupLet),
        ("checked", BindingType::SetupRef),
        ("choice", BindingType::SetupRef),
        ("items", BindingType::SetupConst),
        ("track", BindingType::SetupConst),
        ("MyComp", BindingType::SetupConst),
        ("title", BindingType::Props),
        ("field", BindingType::SetupRef),
        ("modelProps", BindingType::SetupLet),
        ("slotName", BindingType::SetupRef),
        ("row", BindingType::SetupLet),
        ("event", BindingType::SetupRef),
        ("handler", BindingType::SetupConst),
        ("propKey", BindingType::SetupRef),
        ("value", BindingType::SetupMaybeRef),
        ("slotProps", BindingType::SetupLet),
        ("choose", BindingType::SetupConst),
        ("listeners", BindingType::SetupLet),
        ("Bar", BindingType::SetupConst),
        ("Foo", BindingType::SetupConst),
        ("color", BindingType::SetupRef),
    ])
}

fn dom_options(metadata: &BindingMetadata, inline: bool) -> DomCompilerOptions {
    DomCompilerOptions {
        mode: CodegenMode::Module,
        prefix_identifiers: true,
        inline,
        cache_handlers: true,
        scope_id: Some(SCOPE_ID.into()),
        binding_metadata: Some(metadata.clone()),
        is_ts: true,
        ..Default::default()
    }
}

fn emit_options<'a>(table: &'a BindingTable, inline: bool) -> DomEmitOptions<'a> {
    DomEmitOptions {
        mode: DomEmitMode::Module,
        prefix_identifiers: true,
        inline,
        cache_handlers: true,
        scope_id: Some(SCOPE_ID),
        is_ts: true,
        bindings: Some(table),
        ..DomEmitOptions::DEFAULT
    }
}

fn assert_family(inline: bool) {
    let metadata = metadata();
    let table = support::bindings::binding_table(&metadata);
    let dom = dom_options(&metadata, inline);
    let emit = emit_options(&table, inline);
    support::assert_s2_matches_shipped_with_options(
        MODEL_FAMILY,
        &dom,
        &CodegenOptions::default(),
        &emit,
    );
    support::assert_s2_matches_shipped_with_options(
        OUTLET_FAMILY,
        &dom,
        &CodegenOptions::default(),
        &emit,
    );
}

#[test]
fn model_battery_with_cache_handlers_and_scope_id_matches_the_shipped_lane() {
    support::assert_s2_matches_shipped_with_options(
        support::battery::model::MODEL_BATTERY,
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
fn outlet_battery_with_cache_handlers_and_scope_id_matches_the_shipped_lane() {
    support::assert_s2_matches_shipped_with_options(
        support::battery::outlets::OUTLET_BATTERY,
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
fn script_setup_option_families_match_the_shipped_lane() {
    assert_family(false);
}

#[test]
fn inline_script_setup_option_families_match_the_shipped_lane() {
    assert_family(true);
}
