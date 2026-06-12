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
//! `vapor` bool, then VDOM.

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
    /// Generated render code for every component in the module, in source
    /// order, concatenated.
    code: String,
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

fn compile_jsx_internal(source: &str, options: &JsValue) -> JsxWasmResult {
    let lang = resolve_lang(options);
    let default_mode = resolve_default_mode(options);
    build_jsx_wasm_result(source, lang, default_mode)
}

/// Build the JSX compile result from already-resolved options. Kept free of
/// `JsValue` so it can be unit-tested on the host (the wasm-bindgen reflection
/// helpers only run inside a JS runtime).
fn build_jsx_wasm_result(
    source: &str,
    lang: JsxLang,
    default_mode: JsxOutputMode,
) -> JsxWasmResult {
    let config = JsxCompileConfig {
        default_mode,
        ..Default::default()
    };

    let bump = Bump::new();
    let output = jsx_compile(&bump, source, lang, &config);

    let mut code = String::new();
    for component in &output.components {
        if !code.is_empty() {
            code.push('\n');
        }
        code.push_str(component.code());
    }

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
        let result = build_jsx_wasm_result(source, JsxLang::Jsx, JsxOutputMode::Vdom);

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
        );
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        assert!(
            result.scoped_styles.is_empty(),
            "no scoped styles without a block"
        );
    }
}
