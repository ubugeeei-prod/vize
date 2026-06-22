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
    pub(crate) compatibility: RawCompilerCompatibilityConfig,
}
