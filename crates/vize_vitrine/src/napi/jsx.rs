//! NAPI binding for JSX/TSX compilation.
//!
//! Mirrors the SFC `compileSfc` binding: a `.jsx`/`.tsx` source string in,
//! generated render code + diagnostics out. The per-component
//! `"use vue:vapor"` / `"use vue:vdom"` directive prologue is handled inside
//! [`vize_atelier_jsx::compile_jsx`]; the `vapor` option here only selects the
//! default mode for components without an explicit directive.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use napi_derive::napi;
use vize_atelier_jsx::{JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx as jsx_compile};
use vize_carton::Bump;

/// Options for [`compile_jsx`].
#[napi(object)]
#[derive(Default)]
pub struct JsxCompileOptionsNapi {
    /// Source filename, used to infer the language when `lang` is omitted.
    pub filename: Option<String>,
    /// Source language: `"jsx"` or `"tsx"`. Defaults to `"jsx"` (or is inferred
    /// from a `.tsx` `filename`).
    pub lang: Option<String>,
    /// Default output mode: `true` compiles components to Vapor, `false`
    /// (default) to VDOM. Per-component `"use vue:vapor"` / `"use vue:vdom"`
    /// directives override this.
    pub vapor: Option<bool>,
}

/// Result of [`compile_jsx`].
#[napi(object)]
pub struct JsxCompileResultNapi {
    /// Generated render code for every component in the module, in source order,
    /// concatenated.
    pub code: String,
    /// Error-severity diagnostic messages.
    pub errors: Vec<String>,
    /// Warning-severity diagnostic messages.
    pub warnings: Vec<String>,
}

#[napi(js_name = "compileJsx")]
pub fn compile_jsx(
    source: String,
    options: Option<JsxCompileOptionsNapi>,
) -> napi::Result<JsxCompileResultNapi> {
    let opts = options.unwrap_or_default();

    let lang = match opts.lang.as_deref() {
        Some(lang) => JsxLang::from_lang(Some(lang)),
        None => match opts.filename.as_deref() {
            Some(filename) => JsxLang::from_path(filename),
            None => JsxLang::Jsx,
        },
    };

    let default_mode = if opts.vapor.unwrap_or(false) {
        JsxOutputMode::Vapor
    } else {
        JsxOutputMode::Vdom
    };

    let config = JsxCompileConfig {
        default_mode,
        ..Default::default()
    };

    let bump = Bump::new();
    let output = jsx_compile(&bump, &source, lang, &config);

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

    Ok(JsxCompileResultNapi {
        code,
        errors,
        warnings,
    })
}
