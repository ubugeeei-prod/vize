//! Language server feature flags.

use serde::{Deserialize, Serialize};

/// IDE language server settings.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LanguageServerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Legacy diagnostics switch. Kept as a lint diagnostics alias for older configs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lint: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typecheck: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ecosystem: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_symbols: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_symbols: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_actions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatting: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_lens: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_tokens: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_links: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folding_ranges: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inlay_hints: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_rename: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corsa: Option<bool>,
    /// Deprecated alias for `corsa`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tsgo: Option<bool>,
    /// Opt into cross-file diagnostic groups (provide/inject matching,
    /// reactivity tracking, race conditions, etc.) from the language server.
    /// Defaults to off because these groups need a workspace-wide scan and
    /// can be slower than single-file lint. See `vize lint --cross-file` for
    /// the same machinery on the CLI side.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_file: Option<bool>,
}

impl LanguageServerConfig {
    /// Returns true when the config matches the built-in defaults.
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

pub type LspConfig = LanguageServerConfig;
