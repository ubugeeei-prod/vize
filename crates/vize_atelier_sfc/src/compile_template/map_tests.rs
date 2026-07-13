use super::{TemplateBlockCompileContext, compile_template_block};
use crate::types::{BlockLocation, SfcCompileOptions, SfcTemplateBlock, TemplateCompileOptions};
use crate::{SfcParseOptions, compile_sfc, parse_sfc};
use std::borrow::Cow;
use vize_atelier_core::source_map::SourceMapRegistrationState;

fn template_block(source: &str) -> SfcTemplateBlock<'_> {
    SfcTemplateBlock {
        content: Cow::Borrowed(source),
        loc: BlockLocation {
            start: 0,
            end: source.len(),
            tag_start: 0,
            tag_end: source.len(),
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: source.len() + 1,
        },
        lang: None,
        src: None,
        attrs: Default::default(),
    }
}

fn compile_with_source_map(source_map: bool) -> super::TemplateBlockCompileResult {
    let template = template_block("<div>{{ msg }}</div>");
    let options = TemplateCompileOptions {
        compiler_options: Some(vize_atelier_dom::DomCompilerOptions {
            source_map,
            ..Default::default()
        }),
        ..Default::default()
    };

    compile_template_block(
        &template,
        &options,
        TemplateBlockCompileContext {
            scope_id: "abc123",
            apply_scope_id: false,
            has_scoped: false,
            is_ts: false,
            inline: false,
            component_name: Some("MapProbe"),
            bindings: None,
            croquis: None,
        },
        vize_relief::TemplateSyntaxMode::Standard,
        &vize_relief::CodegenOptions::default(),
    )
    .expect("template should compile")
}

fn sfc_options_with_source_map() -> SfcCompileOptions {
    SfcCompileOptions {
        template: TemplateCompileOptions {
            compiler_options: Some(vize_atelier_dom::DomCompilerOptions {
                source_map: true,
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn dom_template_carries_source_map_fragment_without_changing_code() {
    let with_map = compile_with_source_map(true);
    let without_map = compile_with_source_map(false);

    assert_eq!(
        with_map.code, without_map.code,
        "source-map registration must not change generated code"
    );
    assert!(
        without_map.maps.source_map().is_none(),
        "source maps should remain absent by default"
    );

    let map = with_map
        .maps
        .source_map()
        .expect("DOM Atelier map fragment should be retained");
    let parsed: serde_json::Value =
        serde_json::from_str(map).expect("source map fragment should be valid JSON");

    assert_eq!(parsed["version"], 3);
    assert_eq!(parsed["sources"][0], "template.vue");
}

#[test]
fn dom_template_exposes_source_map_registration_mark() {
    let with_map = compile_with_source_map(true);
    let registration = with_map
        .source_map_registration(SourceMapRegistrationState::Composed)
        .expect("source-map registration should be available");
    let functions = with_map
        .module_sections
        .expect("DOM template output should carry module sections")
        .functions;

    assert_eq!(registration.generated, functions);
    assert_eq!(registration.source_name, Some("template.vue"));
    assert!(registration.has_fragment());
    assert!(registration.is_composed());
}

#[test]
fn template_only_sfc_surfaces_template_source_map_when_requested() {
    let source = "<template><div>hi</div></template>";
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("SFC should parse");
    let with_map = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            template: TemplateCompileOptions {
                compiler_options: Some(vize_atelier_dom::DomCompilerOptions {
                    source_map: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("template-only SFC should compile");
    let without_map =
        compile_sfc(&descriptor, SfcCompileOptions::default()).expect("SFC should compile");

    assert_eq!(
        with_map.code, without_map.code,
        "requesting an SFC source map must not change generated code"
    );
    assert!(
        without_map.map.is_none(),
        "SFC source maps should remain absent by default"
    );

    let map = with_map
        .map
        .expect("template-only SFC should surface a map");
    assert_eq!(map["version"], 3);
    assert_eq!(map["sources"][0], "anonymous.vue");
    assert_eq!(map["sourcesContent"][0], source);
    assert!(
        map["mappings"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
}

#[test]
fn normal_script_sfc_surfaces_a_composed_template_map() {
    let source = r#"<template><div>{{ msg }}</div></template>
<script>
export default {
  data() {
    return { msg: "hi" }
  }
}
</script>"#;
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("SFC should parse");
    let with_map = compile_sfc(&descriptor, sfc_options_with_source_map()).expect("SFC compiles");
    let without_map = compile_sfc(&descriptor, SfcCompileOptions::default()).expect("SFC compiles");

    assert_eq!(
        with_map.code, without_map.code,
        "requesting a source map must not change assembled SFC code"
    );
    let map = with_map
        .map
        .expect("normal script/template map is composed");
    assert_eq!(map["sourcesContent"][0], source);
    assert!(
        map["mappings"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
}

#[test]
fn script_setup_sfc_surfaces_a_composed_template_map() {
    let source = r#"<template><button @click="inc">{{ count }}</button></template>
<script setup>
import { ref } from "vue"
const count = ref(0)
const inc = () => count.value++
</script>"#;
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("SFC should parse");
    let with_map = compile_sfc(&descriptor, sfc_options_with_source_map()).expect("SFC compiles");
    let without_map = compile_sfc(&descriptor, SfcCompileOptions::default()).expect("SFC compiles");

    assert_eq!(
        with_map.code, without_map.code,
        "requesting a source map must not change inline script setup output"
    );
    let map = with_map.map.expect("script setup/template map is composed");
    assert_eq!(map["sourcesContent"][0], source);
    assert!(
        map["mappings"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
}
