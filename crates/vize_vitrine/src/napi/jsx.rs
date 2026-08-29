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
//! When `ssr` is true, that mode is kept as client-hydration metadata while
//! generated code is routed through JSX SSR output.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use napi_derive::napi;
use vize_atelier_jsx::{
    JsxCompatMode, JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx as jsx_compile,
};
use vize_s0::Allocator;

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
    /// JSX semantics: `"native"` (default) or `"babel"`. Mirrors the
    /// `compiler.jsxCompat` config key (#3391). `"babel"` opts into
    /// `@vue/babel-plugin-jsx` semantics; it has no Vapor meaning and is
    /// diagnosed when combined with Vapor output.
    pub jsx_compat: Option<String>,
    /// Emit a v3 source map for the generated render code. When `true`, the
    /// result's `map` carries the map JSON; when `false` (default), `map` is
    /// `null` and no mapping work is done (#1533).
    pub source_map: Option<bool>,
    /// Emit SSR render code instead of client render code.
    pub ssr: Option<bool>,
}

/// A JSX component's extracted `<style scoped>` block, surfaced to the bundler
/// plugins so a `.jsx`/`.tsx` component's scoped CSS reaches the same emission
/// path as SFC `<style>` blocks (#1495, #1533).
#[napi(object)]
pub struct JsxScopedStyleNapi {
    /// The generated scope id, e.g. `data-v-1a2b3c4d`. Already injected into the
    /// component's rendered elements; surfaced here so the bundler can name the
    /// emitted stylesheet deterministically.
    pub scope_id: String,
    /// The scoped-rewritten CSS, with the `data-v-<hash>` attribute already
    /// applied to selectors. A bundler emits this verbatim.
    pub css: String,
}

/// Result of [`compile_jsx`].
#[napi(object)]
pub struct JsxCompileResultNapi {
    /// Generated render code for the module: the deduplicated runtime-helper
    /// preamble (`import { … } from "vue"`) followed by every component's render
    /// code in source order. This is a self-contained module string, matching
    /// the shape `compileSfc` returns, so a bundler can emit it directly
    /// (the runtime-helper imports are no longer dropped, #1533).
    pub code: String,
    /// v3 source map (JSON) for `code`, present only when `sourceMap` was
    /// requested and the module is a single component (the per-file shape the
    /// bundler plugins consume). `null` otherwise (#1533).
    pub map: Option<String>,
    /// Error-severity diagnostic messages.
    pub errors: Vec<String>,
    /// Warning-severity diagnostic messages.
    pub warnings: Vec<String>,
    /// Extracted `<style scoped>` blocks across the module's components, in
    /// source order (#1495). Empty when no component had a `<style scoped>`. Each
    /// entry's CSS is already scope-rewritten; the bundler plugins emit it
    /// through the same path SFC styles use (#1533).
    pub scoped_styles: Vec<JsxScopedStyleNapi>,
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
    // Compilation is infallible at this layer (errors surface as `errors` in the
    // result), so the `napi::Result` is always `Ok`. The work lives in
    // [`compile_jsx_impl`] so unit tests can exercise it without linking the
    // Node N-API runtime (a `napi::Result`/`napi::Error` would pull in N-API
    // symbols unavailable to a standalone test binary).
    Ok(compile_jsx_impl(source, options))
}

fn compile_jsx_impl(
    source: String,
    options: Option<JsxCompileOptionsNapi>,
) -> JsxCompileResultNapi {
    let opts = options.unwrap_or_default();

    let lang = match opts.lang.as_deref() {
        Some(lang) => JsxLang::from_lang(Some(lang)),
        None => match opts.filename.as_deref() {
            Some(filename) => JsxLang::from_path(filename),
            None => JsxLang::Jsx,
        },
    };

    let default_mode = resolve_default_mode(opts.jsx_mode.as_deref(), opts.vapor);

    // An unrecognized `jsxCompat` string falls back to `native` rather than
    // erroring, matching how an unrecognized `jsxMode` string is handled: a
    // stray value must never block compilation.
    let compat = opts
        .jsx_compat
        .as_deref()
        .and_then(JsxCompatMode::from_config_str)
        .unwrap_or_default();

    let mut config = JsxCompileConfig {
        default_mode,
        compat,
        ..Default::default()
    };
    config.ssr = opts.ssr.unwrap_or(false);
    // Surface a v3 source map when requested. The flag only affects client VDOM
    // codegen; enabling it is a no-op for Vapor and SSR output.
    config.vdom.source_map = opts.source_map.unwrap_or(false);

    let allocator = Allocator::new();
    let output = jsx_compile(&allocator, &source, lang, &config);

    // A single self-contained module: the deduplicated runtime-helper preamble
    // followed by every component's render code (the preamble is no longer
    // dropped, #1533).
    let code = output.module_code().as_str().to_string();
    let map = output.source_map().map(|map| map.to_string());

    let mut scoped_styles = Vec::new();
    for component in &output.components {
        if let Some(style) = component.scoped_style() {
            scoped_styles.push(JsxScopedStyleNapi {
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

    JsxCompileResultNapi {
        code,
        map,
        errors,
        warnings,
        scoped_styles,
    }
}

#[cfg(test)]
mod tests;
