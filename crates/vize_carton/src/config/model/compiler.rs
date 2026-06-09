//! Compiler config model.

use serde::Deserialize;

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

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct RawCompilerConfig {
    pub(crate) template_syntax: Option<RawTemplateSyntaxConfig>,
}
