//! Production SSR compiles that diverged from the legacy walker on the
//! hydrated corpus: destructured shorthand, a non-ASCII string in an
//! interpolation, and a double-escaped parenthesis in static text.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::expect_used,
    reason = "tests assert by panicking; fixtures use std strings"
)]

use vize_atelier_core::CodegenOptions;
use vize_atelier_core::options::{CustomElementMatcher, TemplateSyntaxMode};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileExperimentalOptions, SfcCompileOptions, SfcParseOptions,
    SfcScriptOutputMode, StyleCompileOptions, TemplateCompileOptions,
    compile_sfc_for_adapter_with_experimental_options, parse_sfc,
};
use vize_atelier_ssr::differential::{record_lanes, with_legacy_lane};

mod component_camel_boundaries;
mod literal_slot_boundaries;
mod nested_dynamic_arguments;
mod v_pre_boundaries;

fn compile(source: &str) -> String {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse");
    let filename: vize_l0::String = "Fixture.vue".into();
    let options = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some(filename.clone()),
            is_ts: true,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            id: Some(filename),
            ssr: true,
            is_ts: true,
            ..Default::default()
        },
        style: StyleCompileOptions::default(),
        vapor: false,
        scope_id: None,
    };
    compile_sfc_for_adapter_with_experimental_options(
        &descriptor,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
        SfcCompileExperimentalOptions::default(),
    )
    .expect("compile")
    .code
    .to_string()
}

fn assert_matches_legacy(label: &str, source: &str) {
    let selected = compile(source);
    let legacy = with_legacy_lane(|| compile(source));
    assert_eq!(selected, legacy, "{label}");
}

#[test]
fn destructured_shorthand_matches_the_legacy_walker() {
    assert_matches_legacy(
        "shorthand",
        r#"<script setup lang="ts">
const items = [{ id: 1, description: "a" }]
function open(payload: { id: number, description: string }) { return payload }
</script>
<template>
  <button v-for="{ id, description } in items" @click="open({ id, description: description ?? '' })" />
</template>"#,
    );
}

#[test]
fn non_ascii_interpolation_matches_the_legacy_walker() {
    assert_matches_legacy(
        "ideo",
        r#"<script setup lang="ts">
const rows: [string, string[]][] = []
</script>
<template>
  <li v-for="[name, extensions] in rows">
    {{ name }}：{{ extensions.map((ext) => `.${ext}`).join("、") }}
  </li>
</template>"#,
    );
}

#[test]
fn double_escaped_parenthesis_text_matches_the_legacy_walker() {
    assert_matches_legacy(
        "paren",
        r#"<template lang="pug">
div
  template(#pug).
    w-confirm(@cancel="$waveui.notify&amp;#40;'Canceled.', 'error'&amp;#41;") Ask
</template>"#,
    );
    let slot = r#"<template>
  <Child>w-confirm(@cancel="$waveui.notify&amp;#40;'Canceled.', 'error'&amp;#41;")</Child>
</template>"#;
    assert_matches_legacy("paren-slot", slot);
}

fn complete_v_pre_ssr(source: &str) -> serde_json::Value {
    let filename: vize_l0::String = "format-v-pre-content.vue".into();
    let parse = SfcParseOptions {
        filename: filename.clone(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("parse original SFC");
    let options = SfcCompileOptions {
        parse,
        script: ScriptCompileOptions {
            id: Some(filename.clone()),
            ..Default::default()
        },
        template: TemplateCompileOptions {
            id: Some(filename),
            ssr: true,
            ..Default::default()
        },
        style: StyleCompileOptions::default(),
        vapor: false,
        scope_id: None,
    };
    let result = compile_sfc_for_adapter_with_experimental_options(
        &descriptor,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
        SfcCompileExperimentalOptions::default(),
    )
    .expect("compile original SFC");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    serde_json::to_value(result).expect("complete serialized result")
}

#[test]
fn v_pre_whitespace_matches_complete_legacy_ssr_results() {
    let original = include_str!(
        "../../../tests/_fixtures/differential/compiler/v-pre-whitespace/format-v-pre-content.vue.txt"
    );
    assert_eq!(original.len(), 228);
    for source in [
        original,
        "<template><div v-pre><i></i> <b></b></div></template>",
        "<template><div v-pre>\n  {{ literal }}\n</div><p>{{ active }}</p></template>",
        "<template><pre v-pre>x\n  <i></i>\n  y</pre></template>",
        "<template><div v-pre><i></i>\u{a0}<b></b></div></template>",
        "<template><div v-pre>\n<!--keep--><i :id=\"raw\"></i>\n</div></template>",
        "<template><div v-pre>\n<i></i>\r\n<b></b>\n</div><p :id=\"active\"></p></template>",
    ] {
        let (selected, lanes) = record_lanes(|| complete_v_pre_ssr(source));
        assert_eq!(
            lanes,
            ["s4"],
            "native SSR provider must execute: {source:?}"
        );
        let legacy = with_legacy_lane(|| complete_v_pre_ssr(source));
        assert_eq!(selected, legacy, "whole SSR result: {source:?}");
    }
}
