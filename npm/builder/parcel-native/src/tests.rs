use super::*;

const FILENAME: &str = "/project/src/Card.vue";
const PROJECT_ROOT: &str = "/project";

fn config() -> TransformerConfig {
    TransformerConfig::default()
}

fn compile(source: &str, is_production: bool, config: &TransformerConfig) -> CompileOutput {
    compile_source(source, FILENAME, PROJECT_ROOT, is_production, config)
        .unwrap_or_else(|error| std::panic::panic_any(error))
}

fn feature_vector(output: &CompileOutput) -> [bool; 8] {
    [
        output.code.contains("const message = \"hello\""),
        output.code.contains("__vize_css__"),
        output.code.contains("__scopeId"),
        output.code.contains(".message[data-v-"),
        output.code.contains("from \"url:./logo.svg\""),
        output.code.contains("_imports_0"),
        output.map.is_some(),
        output.warnings.is_empty(),
    ]
}

#[test]
fn default_config_is_the_documented_public_contract() {
    let config = config();
    assert_eq!(
        (
            config.ssr,
            config.vapor,
            config.styles,
            config.is_production,
            config.transform_asset_urls,
        ),
        (false, false, StyleMode::Inject, None, true),
    );
}

#[test]
fn config_accepts_every_documented_mode_and_rejects_invalid_shapes() {
    let cases = [
        (br#"{}"#.as_slice(), (false, false, StyleMode::Inject, None, true)),
        (
            br#"{"ssr":true,"vapor":true,"styles":"ignore","isProduction":false,"transformAssetUrls":false}"#.as_slice(),
            (true, true, StyleMode::Ignore, Some(false), false),
        ),
        (
            br#"{"styles":"error","isProduction":true}"#.as_slice(),
            (false, false, StyleMode::Error, Some(true), true),
        ),
    ];
    for (source, expected) in cases {
        let parsed =
            parse_transformer_config(source).unwrap_or_else(|error| std::panic::panic_any(error));
        assert_eq!(
            (
                parsed.ssr,
                parsed.vapor,
                parsed.styles,
                parsed.is_production,
                parsed.transform_asset_urls,
            ),
            expected,
        );
    }

    let invalid = [
        br#"{"unknown":true}"#.as_slice(),
        br#"{"styles":"extract"}"#.as_slice(),
        br#"{"ssr":"yes"}"#.as_slice(),
        br#"[]"#.as_slice(),
        br#"{"#.as_slice(),
    ];
    assert_eq!(
        invalid.map(|source| parse_transformer_config(source).is_err()),
        [true; 5],
    );
}

#[test]
fn scoped_typescript_fixture_covers_code_style_scope_map_and_asset_paths() {
    let source = concat!(
        "<script setup lang=\"ts\">const message: string = 'hello'</script>\n",
        "<template><p class=\"message\"><img src=\"./logo.svg\">{{ message }}</p></template>\n",
        "<style scoped>.message { color: red }</style>",
    );
    let output = compile(source, false, &config());
    // Asset substitutions deliberately discard the now-inaccurate generated map.
    assert_eq!(
        feature_vector(&output),
        [true, true, true, true, true, true, false, true],
    );
}

#[test]
fn plain_fixture_preserves_a_parseable_authored_source_map() {
    let output = compile(
        "<script setup lang=\"ts\">const message: string = 'hello'</script>\n<template>{{ message }}</template>",
        false,
        &config(),
    );
    let map = output.map.unwrap_or_default();
    let map = serde_json::from_str::<serde_json::Value>(&map)
        .unwrap_or_else(|error| std::panic::panic_any(error));
    assert_eq!(
        (
            map.get("version").and_then(serde_json::Value::as_u64),
            map.get("sources")
                .and_then(serde_json::Value::as_array)
                .and_then(|sources| sources.first())
                .and_then(serde_json::Value::as_str),
            map.get("mappings")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|mappings| !mappings.is_empty()),
        ),
        (Some(3), Some(FILENAME), true),
    );
}

#[test]
fn style_policies_inject_ignore_and_error_without_silent_fallback() {
    let source = "<template><p /></template><style>p { color: red }</style>";
    let injected = compile(source, false, &config());
    let ignored = compile(
        source,
        false,
        &TransformerConfig {
            styles: StyleMode::Ignore,
            ..config()
        },
    );
    let rejected = compile_source(
        source,
        FILENAME,
        PROJECT_ROOT,
        false,
        &TransformerConfig {
            styles: StyleMode::Error,
            ..config()
        },
    )
    .map(|_| ());
    assert_eq!(
        (
            injected.code.contains("__vize_css__"),
            ignored.code.contains("__vize_css__"),
            rejected,
        ),
        (
            true,
            false,
            Err(cstr!(
                "{FILENAME}: SFC styles are disabled by the native transformer configuration"
            )),
        ),
    );
}

#[test]
fn asset_url_policy_is_explicit_and_controls_map_custody() {
    let source = concat!(
        "<script setup>const logo = 'logo'</script>",
        "<template><img src=\"./logo.svg\" :alt=\"logo\"></template>",
    );
    let enabled = compile(source, false, &config());
    let disabled = compile(
        source,
        false,
        &TransformerConfig {
            transform_asset_urls: false,
            ..config()
        },
    );
    assert_eq!(
        (
            enabled.code.contains("from \"url:./logo.svg\""),
            enabled.map.is_none(),
            disabled.code.contains("from \"url:./logo.svg\""),
            disabled.code.contains("./logo.svg"),
            disabled.map.is_some(),
        ),
        (true, true, false, true, true),
    );
}

#[test]
fn development_production_ssr_and_vapor_modes_all_compile() {
    let source =
        "<script setup>const message = 'hello'</script><template><p>{{ message }}</p></template>";
    let development = compile(source, false, &config());
    let production = compile(source, true, &config());
    let ssr = compile(
        source,
        true,
        &TransformerConfig {
            ssr: true,
            ..config()
        },
    );
    let vapor = compile(
        source,
        true,
        &TransformerConfig {
            vapor: true,
            ..config()
        },
    );
    assert_eq!(
        (
            development.code.contains("render"),
            production.code.contains("render"),
            ssr.code.contains("ssrRender"),
            vapor.code.contains("function render"),
            [development, production, ssr, vapor]
                .iter()
                .all(|output| output.warnings.is_empty()),
        ),
        // Production uses the inline-template path, so there is no separate
        // render declaration. SSR and Vapor retain their target entrypoints.
        (true, false, true, true, true),
    );
}

#[test]
fn descriptor_rejections_cover_every_unsupported_parcel_boundary() {
    let cases = [
        (
            "<script src=\"./script.ts\"></script><template />",
            "external script and template blocks are not supported by the native Parcel transformer",
        ),
        (
            "<script setup src=\"./script.ts\"></script><template />",
            "external script and template blocks are not supported by the native Parcel transformer",
        ),
        (
            "<template src=\"./template.html\"></template>",
            "external script and template blocks are not supported by the native Parcel transformer",
        ),
        (
            "<template /><docs>hello</docs>",
            "custom SFC blocks need a dedicated Parcel transformer",
        ),
        (
            "<template /><style src=\"./style.css\"></style>",
            "<style src> needs a Parcel CSS pipeline and is not supported by the native transformer",
        ),
        (
            "<template /><style module>.root { color: red }</style>",
            "CSS modules need a Parcel CSS pipeline and are not supported by the native transformer",
        ),
        (
            "<template /><style lang=\"scss\">.root { color: red }</style>",
            "style preprocessors need a Parcel CSS pipeline and are not supported by the native transformer",
        ),
        (
            "<template /><style>@import './base.css';</style>",
            "CSS @import needs a Parcel CSS pipeline and is not supported by the native transformer",
        ),
    ];
    for (source, reason) in cases {
        let descriptor = parse_sfc(
            source,
            SfcParseOptions {
                filename: FILENAME.into(),
                ..Default::default()
            },
        )
        .unwrap_or_else(|error| std::panic::panic_any(error.message));
        assert_eq!(
            validate_descriptor(&descriptor, FILENAME),
            Err(cstr!("{FILENAME}: {reason}")),
        );
    }
}

#[test]
fn scope_attachment_covers_named_anonymous_and_missing_exports() {
    assert_eq!(
        attach_scope(String::from("export default _sfc_main;"), "abc"),
        "_sfc_main.__scopeId = \"data-v-abc\";\nexport default _sfc_main;",
    );
    assert_eq!(
        attach_scope(String::from("export default { name: 'Card' };"), "abc"),
        concat!(
            "const _sfc_main = { name: 'Card' };\n",
            "_sfc_main.__scopeId = \"data-v-abc\";\n",
            "export default _sfc_main;\n",
        ),
    );
    assert_eq!(
        attach_scope(String::from("export const value = 1;"), "abc"),
        "export const value = 1;",
    );
}

#[test]
fn style_runtime_is_exact_idempotent_dom_code() {
    assert_eq!(
        style_injection("p { color: red; }", "abc"),
        Ok(String::from(concat!(
            "export const __vize_css__ = \"p { color: red; }\";\n",
            "const __vize_css_id__ = \"vize-style-abc\";\n",
            "if (typeof document !== \"undefined\") {\n",
            "let style = document.getElementById(__vize_css_id__);\n",
            "if (!style) {\n",
            "style = document.createElement(\"style\");\n",
            "style.id = __vize_css_id__;\n",
            "document.head.appendChild(style);\n",
            "}\n",
            "style.textContent = __vize_css__;\n",
            "}\n",
        ))),
    );
}

#[test]
fn source_map_shift_handles_zero_missing_non_string_and_string_mappings() {
    let ordinary = serde_json::json!({"version": 3, "mappings": "AAAA"});
    assert_eq!(shift_map(ordinary.clone(), 0), ordinary);
    assert_eq!(
        shift_map(ordinary, 2),
        serde_json::json!({"version": 3, "mappings": ";;AAAA"}),
    );
    assert_eq!(
        shift_map(serde_json::json!({"version": 3}), 2),
        serde_json::json!({"version": 3}),
    );
    assert_eq!(
        shift_map(serde_json::json!({"version": 3, "mappings": null}), 2),
        serde_json::json!({"version": 3, "mappings": null}),
    );
}
