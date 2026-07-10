//! WASM binding for JSX/TSX compilation.
//!
//! Mirrors the NAPI `compileJsx` binding (`crate::napi::jsx`) and the WASM
//! `compileSfc` binding: a `.jsx`/`.tsx` source string in, generated render
//! code + diagnostics out. The per-component `"use vue:vapor"` /
//! `"use vue:vdom"` directive prologue is handled inside
//! [`vize_atelier_jsx::compile_jsx`]; the `jsxMode` / `vapor` options here only
//! select the *default* mode for components without an explicit directive.
//!
//! Mode selection mirrors the NAPI binding and the `compiler.jsxMode` config:
//! the explicit `jsxMode` string (`"vdom"` / `"vapor"`) wins, then the legacy
//! `vapor` bool, then VDOM. When `ssr` is true, that mode is kept as
//! client-hydration metadata while generated code is routed through JSX SSR
//! output.

use serde::Serialize;
use wasm_bindgen::prelude::*;

use vize_atelier_jsx::{JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx as jsx_compile};
use vize_carton::Bump;

use super::serde::to_json_js_value;

/// A JSX component's extracted `<style scoped>` block for WASM. Mirrors the NAPI
/// `JsxScopedStyleNapi`: scope id + scope-rewritten CSS, surfaced so the bundler
/// plugins emit JSX scoped CSS through the SFC-style path (#1495, #1533).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsxScopedStyleWasm {
    /// The generated scope id, e.g. `data-v-1a2b3c4d`.
    scope_id: String,
    /// The scoped-rewritten CSS, with the `data-v-<hash>` attribute applied.
    css: String,
}

/// JSX/TSX compile result for WASM.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsxWasmResult {
    /// Generated render code for the module: the deduplicated runtime-helper
    /// preamble (`import { … } from "vue"`) followed by every component's render
    /// code in source order — a self-contained module, matching the NAPI binding
    /// and `compileSfc` (the helper imports are no longer dropped, #1533).
    code: String,
    /// v3 source map (JSON) for `code`, present only when `sourceMap` was
    /// requested and the module is a single component. `null` otherwise (#1533).
    map: Option<String>,
    /// Error-severity diagnostic messages.
    errors: Vec<String>,
    /// Warning-severity diagnostic messages.
    warnings: Vec<String>,
    /// Extracted `<style scoped>` blocks across the module's components, in
    /// source order (#1495). Empty when no component had a `<style scoped>`.
    scoped_styles: Vec<JsxScopedStyleWasm>,
}

/// Resolve the source language from the JS options object, mirroring the NAPI
/// binding: explicit `lang` wins, otherwise infer from `filename`, otherwise
/// default to JSX.
fn resolve_lang(options: &JsValue) -> JsxLang {
    let lang = js_sys::Reflect::get(options, &JsValue::from_str("lang"))
        .ok()
        .and_then(|v| v.as_string());

    match lang.as_deref() {
        Some(lang) => JsxLang::from_lang(Some(lang)),
        None => {
            let filename = js_sys::Reflect::get(options, &JsValue::from_str("filename"))
                .ok()
                .and_then(|v| v.as_string());
            match filename.as_deref() {
                Some(filename) => JsxLang::from_path(filename),
                None => JsxLang::Jsx,
            }
        }
    }
}

/// Resolve the default output mode from the JS options object, mirroring the
/// NAPI binding and `compiler.jsxMode` precedence: an explicit `jsxMode` string
/// (`"vdom"` / `"vapor"`) wins, then the legacy `vapor` bool, then VDOM. An
/// unrecognized `jsxMode` string falls through to the `vapor`/VDOM fallback.
fn resolve_default_mode(options: &JsValue) -> JsxOutputMode {
    let jsx_mode = js_sys::Reflect::get(options, &JsValue::from_str("jsxMode"))
        .ok()
        .and_then(|v| v.as_string());
    if let Some(mode) = jsx_mode.as_deref().and_then(JsxOutputMode::from_config_str) {
        return mode;
    }

    let vapor = js_sys::Reflect::get(options, &JsValue::from_str("vapor"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if vapor {
        JsxOutputMode::Vapor
    } else {
        JsxOutputMode::Vdom
    }
}

/// Resolve the `sourceMap` flag from the JS options object: `true` enables v3
/// source-map emission, anything else (including omission) leaves it off.
fn resolve_source_map(options: &JsValue) -> bool {
    js_sys::Reflect::get(options, &JsValue::from_str("sourceMap"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn resolve_ssr(options: &JsValue) -> bool {
    js_sys::Reflect::get(options, &JsValue::from_str("ssr"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn compile_jsx_internal(source: &str, options: &JsValue) -> JsxWasmResult {
    let lang = resolve_lang(options);
    let default_mode = resolve_default_mode(options);
    let source_map = resolve_source_map(options);
    let ssr = resolve_ssr(options);
    build_jsx_wasm_result(source, lang, default_mode, source_map, ssr)
}

/// Build the JSX compile result from already-resolved options. Kept free of
/// `JsValue` so it can be unit-tested on the host (the wasm-bindgen reflection
/// helpers only run inside a JS runtime).
fn build_jsx_wasm_result(
    source: &str,
    lang: JsxLang,
    default_mode: JsxOutputMode,
    source_map: bool,
    ssr: bool,
) -> JsxWasmResult {
    let mut config = JsxCompileConfig {
        default_mode,
        ..Default::default()
    };
    config.ssr = ssr;
    // Source maps are emitted by client VDOM codegen; a no-op for Vapor/SSR.
    config.vdom.source_map = source_map;

    let bump = Bump::new();
    let output = jsx_compile(&bump, source, lang, &config);

    // A single self-contained module: deduplicated runtime-helper preamble + every
    // component's render code (the preamble is no longer dropped, #1533).
    let code = output.module_code().as_str().to_string();
    let map = output.source_map().map(|map| map.to_string());

    let mut scoped_styles = Vec::new();
    for component in &output.components {
        if let Some(style) = component.scoped_style() {
            scoped_styles.push(JsxScopedStyleWasm {
                scope_id: style.scope_id.as_str().to_string(),
                css: style.css.as_str().to_string(),
            });
        }
    }

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for diagnostic in &output.diagnostics {
        if diagnostic.is_error() {
            errors.push(diagnostic.message.as_str().to_string());
        } else {
            warnings.push(diagnostic.message.as_str().to_string());
        }
    }

    JsxWasmResult {
        code,
        map,
        errors,
        warnings,
        scoped_styles,
    }
}

/// Compile JSX/TSX to render code (free function).
#[wasm_bindgen(js_name = "compileJsx")]
pub fn compile_jsx(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    to_json_js_value(&compile_jsx_internal(source, &options))
}

#[cfg(test)]
mod tests {
    use super::{JsxLang, JsxOutputMode, build_jsx_wasm_result};

    #[test]
    fn wasm_jsx_result_surfaces_scoped_style_css() {
        // A `.jsx` component's `<style scoped>` must reach the WASM compile
        // result, scope-rewritten, so the browser playground can emit it
        // (#1495, #1533).
        let source = r#"
            const App = () => (
                <div class="box">
                    <style scoped>{`.box { color: red }`}</style>
                </div>
            );
        "#;
        let result = build_jsx_wasm_result(source, JsxLang::Jsx, JsxOutputMode::Vdom, false, false);

        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        assert_eq!(
            result.scoped_styles.len(),
            1,
            "one scoped style is surfaced"
        );
        let style = &result.scoped_styles[0];
        assert!(
            style.scope_id.starts_with("data-v-"),
            "scope id is a data-v- attribute: {}",
            style.scope_id
        );
        assert!(
            style.css.contains(".box") && style.css.contains(&style.scope_id),
            "rewritten CSS applies the scope id: {}",
            style.css
        );
    }

    #[test]
    fn wasm_jsx_result_has_no_scoped_styles_without_style_block() {
        let result = build_jsx_wasm_result(
            "const App = () => <div class=\"box\">hi</div>;",
            JsxLang::Jsx,
            JsxOutputMode::Vdom,
            false,
            false,
        );
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        assert!(
            result.scoped_styles.is_empty(),
            "no scoped styles without a block"
        );
    }

    #[test]
    fn wasm_jsx_result_includes_runtime_helper_preamble() {
        // The WASM result `code` carries the runtime-helper imports so the
        // playground emits a self-contained module (preamble no longer dropped,
        // #1533).
        let result = build_jsx_wasm_result(
            "const App = () => <div>{message}</div>;",
            JsxLang::Jsx,
            JsxOutputMode::Vdom,
            false,
            false,
        );
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        assert!(
            result.code.contains("from \"vue\"") && result.code.contains("_createElementBlock"),
            "code carries the runtime-helper import: {}",
            result.code
        );
    }

    #[test]
    fn wasm_jsx_result_surfaces_source_map_when_requested() {
        let source = "const App = () => <div>{message}</div>;";

        let without =
            build_jsx_wasm_result(source, JsxLang::Jsx, JsxOutputMode::Vdom, false, false);
        assert!(without.map.is_none(), "no map unless requested");

        let with = build_jsx_wasm_result(source, JsxLang::Jsx, JsxOutputMode::Vdom, true, false);
        assert!(with.errors.is_empty(), "errors: {:?}", with.errors);
        let map = with.map.expect("a map is surfaced when requested");
        assert!(map.contains("\"version\":3"), "v3 source map: {map}");
    }

    #[test]
    fn wasm_jsx_result_supports_ssr_output() {
        let result = build_jsx_wasm_result(
            "const App = () => <div>{message}</div>;",
            JsxLang::Jsx,
            JsxOutputMode::Vdom,
            true,
            true,
        );

        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        assert!(
            result.code.contains("function ssrRender"),
            "{}",
            result.code
        );
        assert!(
            result.code.contains("@vue/server-renderer"),
            "{}",
            result.code
        );
        assert!(result.map.is_none(), "SSR output has no source map yet");
    }
}
