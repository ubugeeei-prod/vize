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

/// JSX/TSX compile result for WASM.
#[derive(Serialize)]
struct JsxWasmResult {
    /// Generated render code for every component in the module, in source
    /// order, concatenated.
    code: String,
    /// Error-severity diagnostic messages.
    errors: Vec<String>,
    /// Warning-severity diagnostic messages.
    warnings: Vec<String>,
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
    }
}

/// Compile JSX/TSX to render code (free function).
#[wasm_bindgen(js_name = "compileJsx")]
pub fn compile_jsx(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    to_json_js_value(&compile_jsx_internal(source, &options))
}
