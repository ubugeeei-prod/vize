use serde::{Deserialize, Serialize};

/// Spacing required inside Vue mustache interpolations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MustacheInterpolationSpacingStyle {
    /// Require one or more whitespace characters after `{{` and before `}}`.
    Always,
    /// Forbid whitespace after `{{` and before `}}`.
    Never,
}
