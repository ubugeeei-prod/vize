//! Compiler config model.

use serde::Deserialize;

use super::vue::VueVersion;

/// Template syntax compatibility mode from `compiler.templateSyntax`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RawTemplateSyntaxConfig {
    Standard,
    Strict,
    Quirks,
}

impl RawTemplateSyntaxConfig {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Strict => "strict",
            Self::Quirks => "quirks",
        }
    }
}

/// Default JSX/TSX output backend from `compiler.jsxMode` (#1496).
///
/// Selects the mode applied to `.jsx`/`.tsx` components that carry no
/// `"use vue:vapor"` / `"use vue:vdom"` directive prologue. Distinct from
/// `compiler.vapor`, which only toggles Vapor for `.vue` SFCs; a project can
/// keep SFCs on VDOM while defaulting JSX to Vapor, or vice versa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JsxMode {
    /// Virtual DOM output — the default, matching Vue's default renderer.
    #[default]
    Vdom,
    /// Vapor output.
    Vapor,
}

impl JsxMode {
    /// The canonical `compiler.jsxMode` config value for this mode.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vdom => "vdom",
            Self::Vapor => "vapor",
        }
    }
}

/// JSX/TSX compilation semantics from `compiler.jsxCompat` (#3391).
///
/// `Babel` opts `.jsx`/`.tsx` compilation into `@vue/babel-plugin-jsx`
/// semantics for projects migrating off the babel plugin. It is orthogonal to
/// [`JsxMode`]: the plugin is vdom-era, so `babel` with `jsxMode: "vapor"` is
/// rejected with a diagnostic rather than silently ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JsxCompat {
    /// Vize's own JSX semantics — the default. Must stay the default: flipping
    /// it would silently change output for every existing project.
    #[default]
    Native,
    /// `@vue/babel-plugin-jsx` semantics.
    Babel,
}

impl JsxCompat {
    /// The canonical `compiler.jsxCompat` config value for this mode.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Babel => "babel",
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct RawCompilerCompatibilityConfig {
    pub(crate) vue_version: Option<VueVersion>,
    pub(crate) host_compiler: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct RawCompilerConfig {
    /// Explicit SFC Vapor mode switch from `compiler.vapor`.
    pub(crate) vapor: Option<bool>,
    pub(crate) template_syntax: Option<RawTemplateSyntaxConfig>,
    /// Default JSX output mode (`compiler.jsxMode`); `None` when absent, which
    /// the JSX entry points treat as VDOM.
    pub(crate) jsx_mode: Option<JsxMode>,
    /// JSX compatibility semantics (`compiler.jsxCompat`); `None` when absent,
    /// which the JSX entry points treat as `native`.
    pub(crate) jsx_compat: Option<JsxCompat>,
    /// Tag patterns that compile as custom elements instead of Vue components.
    pub(crate) custom_elements: Vec<crate::String>,
    pub(crate) compatibility: RawCompilerCompatibilityConfig,
}
