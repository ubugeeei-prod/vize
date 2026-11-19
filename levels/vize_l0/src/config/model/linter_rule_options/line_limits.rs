//! Options for `max-lines` and `vue/max-lines-per-block`.

use serde::{Deserialize, Serialize};

use super::ConfigLintRuleOptions;
use crate::String;

/// Object form of `linter.ruleOptions["max-lines"]`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct MaxLinesObject {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_comments: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_blank_lines: Option<bool>,
}

/// `linter.ruleOptions["max-lines"]`: a bare maximum or the object form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaxLinesOptions {
    Max(u32),
    Object(MaxLinesObject),
}

/// `linter.ruleOptions["vue/max-lines-per-block"]`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct MaxLinesPerBlockOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_blank_lines: Option<bool>,
}

impl ConfigLintRuleOptions {
    /// Configured whole-file line limit.
    pub fn max_lines(&self) -> Option<&MaxLinesOptions> {
        self.max_lines.as_ref()
    }

    /// Configured per-block line limits.
    pub fn max_lines_per_block(&self) -> Option<&MaxLinesPerBlockOptions> {
        self.max_lines_per_block.as_ref()
    }

    /// Configured design-token primitive values for
    /// `musea/prefer-design-tokens` as `(value, path, tier)` tuples. Empty
    /// when unconfigured.
    pub fn musea_design_tokens(&self) -> Vec<(String, String, String)> {
        self.musea_prefer_design_tokens
            .as_ref()
            .map(|options| {
                options
                    .tokens
                    .iter()
                    .map(|token| (token.value.clone(), token.path.clone(), token.tier.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }
}
