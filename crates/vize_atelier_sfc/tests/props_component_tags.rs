//! The pinned `compiler/sfc/props-component-tags` differential case: a
//! `<script setup>` prop used as a component tag reads the props object in
//! every lane, and a renamed props destructure resolves as an asset. Vue's
//! `resolveSetupReference` (`@vue/compiler-sfc` 3.6.0-rc.17) is the reference:
//! `_unref(__props["myIcon"])` in the inlined DOM render function,
//! `_unref($props["myIcon"])` in the SSR render function, `$props.myIcon` in
//! Vapor, and `_resolveComponent("Icon")` for `{ icon: Icon }` everywhere.
//! `VIZE_BLESS_PROPS_COMPONENT_TAGS=1` rewrites the pinned SSR module.

use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc, parse_sfc,
};

fn fixture(file: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/compiler/props-component-tags")
        .join(file)
}

fn compile(ssr: bool, vapor: bool) -> String {
    let source = std::fs::read_to_string(fixture("Layout.vue.txt")).expect("input");
    let parse = SfcParseOptions {
        filename: "Layout.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(&source, parse.clone()).expect("SFC");
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            parse,
            vapor,
            template: TemplateCompileOptions {
                ssr,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("compile");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    result.code.as_str().to_owned()
}

/// Compare `code` with the pinned module at `path`, or rewrite it under
/// `VIZE_BLESS_PROPS_COMPONENT_TAGS`.
fn assert_pinned(code: &str, path: std::path::PathBuf) {
    if std::env::var_os("VIZE_BLESS_PROPS_COMPONENT_TAGS").is_some() {
        std::fs::create_dir_all(path.parent().expect("fixture directory")).expect("directory");
        std::fs::write(&path, code).expect("bless module");
        return;
    }
    let expected = std::fs::read_to_string(&path).expect("pinned module");
    assert_eq!(code, expected, "{}", path.display());
}

/// Lane modules pinned beside this test; the SSR module is the differential
/// case's reference.
fn lane_fixture(file: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/props_component_tags")
        .join(file)
}

#[test]
fn the_ssr_module_matches_its_pinned_reference() {
    assert_pinned(&compile(true, false), fixture("module.expected.txt"));
}

/// `_createVNode(_unref(__props["myIcon"]))` and `_resolveComponent("Icon")`.
#[test]
fn the_dom_module_reads_the_prop_and_resolves_the_renamed_destructure() {
    assert_pinned(&compile(false, false), lane_fixture("dom.expected.txt"));
}

/// `const _component_my_icon = $props.myIcon` and `_resolveComponent("Icon")`.
#[test]
fn the_vapor_module_reads_dollar_props() {
    assert_pinned(&compile(false, true), lane_fixture("vapor.expected.txt"));
}
