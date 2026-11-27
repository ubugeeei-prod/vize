//! The render code of a component that has a script maps back to its
//! template, and asking for the map leaves the module untouched.
//!
//! The component is the report's `Broken.vue`, whose template expression
//! throws while rendering. Its server module is the pinned differential
//! reference; the map is decoded here the way a stack frame resolves.

use oxc_sourcemap::SourceMap;
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    SfcCompileExperimentalOptions, SfcCompileOptions, SfcCompileResult, SfcParseOptions,
    SfcScriptOutputMode, compile_sfc, compile_sfc_for_adapter_with_project, parse_sfc,
};

const SOURCE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/template-source-map-13547/Layout.vue.txt"
);
const SERVER_MODULE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/template-source-map-13547/module.expected.txt"
);

fn options(ssr: bool) -> SfcCompileOptions {
    let mut options = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: "Layout.vue".into(),
            ..Default::default()
        },
        ..Default::default()
    };
    options.template.ssr = ssr;
    options
}

/// The module the Vite plugin serves: a render function beside `setup`.
fn mapped(ssr: bool) -> SfcCompileResult {
    let options = options(ssr);
    let descriptor = parse_sfc(SOURCE, options.parse.clone()).expect("SFC");
    compile_sfc_for_adapter_with_project(
        &descriptor,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions {
            source_map: true,
            ..Default::default()
        },
        SfcScriptOutputMode::SeparateTemplate,
        SfcCompileExperimentalOptions::default(),
        false,
        None,
    )
    .expect("mapped module")
}

/// The authored 1-based line and 0-based column of the generated position
/// where `needle` starts: the last segment at or before it on its line.
fn frame(result: &SfcCompileResult, needle: &str) -> Option<(u32, u32)> {
    let at = result.code.find(needle).expect("generated needle");
    let head = result.code.get(..at).unwrap_or_default();
    let line = head.matches('\n').count() as u32;
    let column = (at - head.rfind('\n').map_or(0, |start| start + 1)) as u32;
    let json = serde_json::to_string(result.map.as_ref()?).ok()?;
    let map = SourceMap::from_json_string(&json).ok()?;
    map.get_tokens()
        .filter(|token| token.get_dst_line() == line && token.get_dst_col() <= column)
        .last()
        .map(|token| (token.get_src_line() + 1, token.get_src_col()))
}

#[test]
fn the_server_module_is_the_pinned_reference_with_or_without_a_map() {
    let descriptor = parse_sfc(SOURCE, options(true).parse).expect("SFC");
    let plain = compile_sfc(&descriptor, options(true)).expect("server module");
    assert_eq!(plain.errors.len(), 0);
    assert_eq!(plain.map, None);
    assert_eq!(plain.code.as_str(), SERVER_MODULE);
    assert_eq!(mapped(true).code.as_str(), SERVER_MODULE);
}

#[test]
fn a_frame_in_the_render_resolves_to_the_template_expression() {
    // `  <p>{{ trip.title }}</p>` is line 9, and `trip` starts at column 8.
    assert_eq!(frame(&mapped(false), "$setup.trip.title"), Some((9, 8)));
    assert_eq!(frame(&mapped(true), "$setup.trip.title"), Some((9, 8)));
    // The thrown property read points at `title`, as the Vue renderer does.
    assert_eq!(frame(&mapped(false), "title),"), Some((9, 13)));
    assert_eq!(frame(&mapped(true), "title)}"), Some((9, 13)));
    // `  <Child label="before" />` is line 8, and `Child` starts at column 3.
    assert_eq!(frame(&mapped(false), "$setup[\"Child\"]"), Some((8, 3)));
    assert_eq!(frame(&mapped(true), "$setup.Child"), Some((8, 3)));
}
