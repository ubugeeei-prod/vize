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
use vize_atelier_ssr::differential::with_legacy_lane;

fn compile(source: &str) -> String {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse");
    let filename: vize_s0::String = "Fixture.vue".into();
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
