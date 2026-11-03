use crate::String;
use serde::{Deserialize, Serialize};

/// Options for `script/no-restricted-globals`.
///
/// When `globals` is non-empty it **replaces** the rule's built-in deny list;
/// otherwise the built-in defaults (`process`, `localStorage`, `sessionStorage`)
/// apply.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NoRestrictedGlobalsOptions {
    /// Restricted global identifier references.
    pub globals: Vec<RestrictedGlobal>,
}

/// A single restricted global entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RestrictedGlobal {
    /// Identifier name to forbid (e.g. `process`).
    pub name: String,
    /// Optional advisory message shown in the diagnostic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Options for `script/no-restricted-members`.
///
/// The rule is off unless `members` is configured; there is no built-in default
/// list. This is the project-local-rule mechanism: each entry flags an
/// `<object>.<property>` member access.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NoRestrictedMembersOptions {
    /// Restricted `<object>.<property>` member accesses.
    pub members: Vec<RestrictedMember>,
}

/// A single restricted member-access entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RestrictedMember {
    /// Object identifier (e.g. `window`).
    pub object: String,
    /// Property name accessed on the object (e.g. `localStorage`).
    pub property: String,
    /// Optional advisory message shown in the diagnostic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Options for `musea/prefer-design-tokens`.
///
/// The rule is off unless tokens are configured and the rule is enabled by
/// severity or implicitly by this non-empty token list. Each token maps one
/// hardcoded CSS value to a design-token path.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct MuseaPreferDesignTokensOptions {
    /// Design tokens that should replace matching hardcoded CSS values.
    pub tokens: Vec<MuseaDesignToken>,
}

/// A design token recognized by `musea/prefer-design-tokens`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MuseaDesignToken {
    /// Token path (e.g. `color.primary`).
    pub path: String,
    /// Primitive CSS value to match (e.g. `#3b82f6`).
    pub value: String,
    /// Token tier shown in diagnostics. Defaults to `primitive`.
    #[serde(default = "default_musea_design_token_tier")]
    pub tier: String,
}

fn default_musea_design_token_tier() -> String {
    "primitive".into()
}
