//! P2-11 witness: dynamic `v-bind` modifier keys under the production
//! option bundle. Default-lane tests cover the modifier spelling; this
//! keeps the S2 DOM emitter aligned when module output, prefixing,
//! binding metadata, TypeScript erasure, scoped CSS and cached handlers
//! all share the same props object.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_atelier_core::options::{BindingMetadata, BindingType, CodegenMode, CodegenOptions};
use vize_atelier_dom::DomCompilerOptions;
use vize_s1_to_s2::{BindingTable, DomEmitMode, DomEmitOptions};

const SCOPE_ID: &str = "data-v-dyn";

const BATTERY: &[(&str, &str)] = &[
    (
        "native_dynamic_camel_scoped",
        r#"<div :[key].camel="value"></div>"#,
    ),
    (
        "native_dynamic_prop_attr_ts_handler",
        r#"<input :[key].prop.attr="value as any" @change="track(value)">"#,
    ),
    (
        "component_dynamic_template_literal_camel",
        r#"<Widget :[`data-${id}`].camel="value" :title="title" />"#,
    ),
    (
        "component_v_for_dynamic_prop_key",
        r#"<Widget v-for="item in items" :key="item.id" :[field].prop="item.value" />"#,
    ),
    (
        "slot_outlet_dynamic_camel_scoped",
        r#"<slot :[slotProp].camel="value" name="row">fallback {{ title }}</slot>"#,
    ),
    (
        "spread_then_dynamic_attr",
        r#"<div v-bind="bag" :[key].attr="value"></div>"#,
    ),
];

fn metadata() -> BindingMetadata {
    support::bindings::script_setup_metadata(&[
        ("key", BindingType::SetupRef),
        ("value", BindingType::SetupMaybeRef),
        ("track", BindingType::SetupConst),
        ("Widget", BindingType::SetupConst),
        ("id", BindingType::SetupLet),
        ("title", BindingType::Props),
        ("items", BindingType::SetupConst),
        ("field", BindingType::SetupRef),
        ("slotProp", BindingType::SetupRef),
        ("bag", BindingType::SetupLet),
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
        bindings: Some(table),
        is_ts: true,
        ..DomEmitOptions::DEFAULT
    }
}

fn assert_dynamic_bind_options(inline: bool) {
    let metadata = metadata();
    let table = support::bindings::binding_table(&metadata);
    support::assert_s2_matches_shipped_with_options(
        BATTERY,
        &dom_options(&metadata, inline),
        &CodegenOptions::default(),
        &emit_options(&table, inline),
    );
}

#[test]
fn dynamic_bind_modifiers_match_the_script_setup_option_bundle() {
    assert_dynamic_bind_options(false);
}

#[test]
fn dynamic_bind_modifiers_match_the_inline_script_setup_option_bundle() {
    assert_dynamic_bind_options(true);
}
