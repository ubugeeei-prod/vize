//! NAPI binding for JSX/TSX compilation.
//!
//! Mirrors the SFC `compileSfc` binding: a `.jsx`/`.tsx` source string in,
//! generated render code + diagnostics out. The per-component
//! `"use vue:vapor"` / `"use vue:vdom"` directive prologue is handled inside
//! [`vize_atelier_jsx::compile_jsx`]; the `jsxMode` / `vapor` options here only
//! select the *default* mode for components without an explicit directive.
//!
//! Mode selection follows the project's `compiler.jsxMode` config: the explicit
//! `jsxMode` string (`"vdom"` / `"vapor"`) wins when present; otherwise the
//! legacy `vapor` bool applies for back-compat; otherwise the default is VDOM.
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
    /// Default output mode: `"vdom"` (default) or `"vapor"`. Mirrors the
    /// `compiler.jsxMode` config key and takes precedence over `vapor`.
    /// Per-component `"use vue:vapor"` / `"use vue:vdom"` directives override it.
    pub jsx_mode: Option<String>,
    /// Legacy default-mode toggle: `true` compiles components to Vapor, `false`
    /// (default) to VDOM. Kept for back-compat; prefer `jsxMode`. Ignored when
    /// `jsxMode` is set.
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

/// Resolve the default JSX output mode from the binding options, following the
/// `compiler.jsxMode` precedence: an explicit `jsxMode` string wins, then the
/// legacy `vapor` bool, then VDOM. An unrecognized `jsxMode` string falls
/// through to the same `vapor`/VDOM fallback rather than erroring, so a stray
/// value never blocks compilation.
fn resolve_default_mode(jsx_mode: Option<&str>, vapor: Option<bool>) -> JsxOutputMode {
    if let Some(mode) = jsx_mode.and_then(JsxOutputMode::from_config_str) {
        return mode;
    }
    if vapor.unwrap_or(false) {
        JsxOutputMode::Vapor
    } else {
        JsxOutputMode::Vdom
    }
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

    let default_mode = resolve_default_mode(opts.jsx_mode.as_deref(), opts.vapor);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsx_mode_takes_precedence_over_vapor() {
        // Explicit `jsxMode` wins even when `vapor` disagrees.
        assert_eq!(
            resolve_default_mode(Some("vapor"), Some(false)),
            JsxOutputMode::Vapor
        );
        assert_eq!(
            resolve_default_mode(Some("vdom"), Some(true)),
            JsxOutputMode::Vdom
        );
    }

    #[test]
    fn falls_back_to_vapor_bool_then_vdom() {
        assert_eq!(resolve_default_mode(None, Some(true)), JsxOutputMode::Vapor);
        assert_eq!(resolve_default_mode(None, Some(false)), JsxOutputMode::Vdom);
        assert_eq!(resolve_default_mode(None, None), JsxOutputMode::Vdom);
        // An unrecognized jsxMode string falls through rather than erroring.
        assert_eq!(
            resolve_default_mode(Some("react"), Some(true)),
            JsxOutputMode::Vapor
        );
    }
}
