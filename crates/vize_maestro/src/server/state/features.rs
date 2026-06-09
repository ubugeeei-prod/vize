//! LSP feature flags and config-section parsing.

use serde::Deserialize;
use vize_carton::config::LanguageServerConfig;

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct LspConfigSection {
    enabled: Option<bool>,
    /// Legacy diagnostics switch. Kept as a lint diagnostics alias for older configs.
    diagnostics: Option<bool>,
    lint: Option<bool>,
    typecheck: Option<bool>,
    editor: Option<bool>,
    ecosystem: Option<bool>,
    options_api: Option<bool>,
    legacy_vue2: Option<bool>,
    completion: Option<bool>,
    hover: Option<bool>,
    definition: Option<bool>,
    references: Option<bool>,
    document_symbols: Option<bool>,
    workspace_symbols: Option<bool>,
    code_actions: Option<bool>,
    rename: Option<bool>,
    formatting: Option<bool>,
    code_lens: Option<bool>,
    semantic_tokens: Option<bool>,
    document_links: Option<bool>,
    folding_ranges: Option<bool>,
    inlay_hints: Option<bool>,
    file_rename: Option<bool>,
    corsa: Option<bool>,
    tsgo: Option<bool>,
    cross_file: Option<bool>,
}

impl LspConfigSection {
    pub(super) fn apply_to(self, features: &mut LspFeatureConfig) {
        if self.enabled == Some(false) {
            *features = LspFeatureConfig::disabled();
            return;
        }

        if let Some(enabled) = self.diagnostics {
            features.lint = enabled;
            if !enabled {
                features.typecheck = false;
            }
        }

        if let Some(enabled) = self.lint {
            features.lint = enabled;
        }

        if let Some(enabled) = self.typecheck {
            features.typecheck = enabled;
        }
        if self.corsa == Some(true) || self.tsgo == Some(true) {
            features.typecheck = true;
        }

        if let Some(enabled) = self.editor {
            features.apply_editor_bundle(enabled);
        }

        if let Some(enabled) = self.ecosystem {
            features.ecosystem = enabled;
        }

        if let Some(enabled) = self.options_api {
            features.options_api = enabled;
        }

        if let Some(enabled) = self.legacy_vue2 {
            features.legacy_vue2 = enabled;
        }

        if let Some(enabled) = self.completion {
            features.completion = enabled;
        }
        if let Some(enabled) = self.hover {
            features.hover = enabled;
        }
        if let Some(enabled) = self.definition {
            features.definition = enabled;
        }
        if let Some(enabled) = self.references {
            features.references = enabled;
        }
        if let Some(enabled) = self.document_symbols {
            features.document_symbols = enabled;
        }
        if let Some(enabled) = self.workspace_symbols {
            features.workspace_symbols = enabled;
        }
        if let Some(enabled) = self.code_actions {
            features.code_actions = enabled;
        }
        if let Some(enabled) = self.rename {
            features.rename = enabled;
        }
        if let Some(enabled) = self.formatting {
            features.formatting = enabled;
        }
        if let Some(enabled) = self.code_lens {
            features.code_lens = enabled;
        }
        if let Some(enabled) = self.semantic_tokens {
            features.semantic_tokens = enabled;
        }
        if let Some(enabled) = self.document_links {
            features.document_links = enabled;
        }
        if let Some(enabled) = self.folding_ranges {
            features.folding_ranges = enabled;
        }
        if let Some(enabled) = self.inlay_hints {
            features.inlay_hints = enabled;
        }
        if let Some(enabled) = self.file_rename {
            features.file_rename = enabled;
        }
        if let Some(enabled) = self.cross_file {
            features.cross_file = enabled;
        }
    }
}

impl From<LanguageServerConfig> for LspConfigSection {
    fn from(config: LanguageServerConfig) -> Self {
        Self {
            enabled: config.enabled,
            diagnostics: config.diagnostics,
            lint: config.lint,
            typecheck: config.typecheck,
            editor: config.editor,
            ecosystem: config.ecosystem,
            options_api: None,
            legacy_vue2: None,
            completion: config.completion,
            hover: config.hover,
            definition: config.definition,
            references: config.references,
            document_symbols: config.document_symbols,
            workspace_symbols: config.workspace_symbols,
            code_actions: config.code_actions,
            rename: config.rename,
            formatting: config.formatting,
            code_lens: config.code_lens,
            semantic_tokens: config.semantic_tokens,
            document_links: config.document_links,
            folding_ranges: config.folding_ranges,
            inlay_hints: config.inlay_hints,
            file_rename: config.file_rename,
            corsa: config.corsa,
            tsgo: config.tsgo,
            cross_file: config.cross_file,
        }
    }
}

/// Feature switches for Maestro LSP capabilities.
///
/// Non-opinionated diagnostics and editor features default to on. Formatting stays
/// opt-in because it encodes style preferences and can overlap with project formatters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspFeatureConfig {
    pub(crate) lint: bool,
    pub(crate) typecheck: bool,
    pub(crate) ecosystem: bool,
    pub(crate) options_api: bool,
    pub(crate) legacy_vue2: bool,
    pub(crate) completion: bool,
    pub(crate) hover: bool,
    pub(crate) definition: bool,
    pub(crate) references: bool,
    pub(crate) document_symbols: bool,
    pub(crate) workspace_symbols: bool,
    pub(crate) code_actions: bool,
    pub(crate) rename: bool,
    pub(crate) formatting: bool,
    pub(crate) code_lens: bool,
    pub(crate) semantic_tokens: bool,
    pub(crate) document_links: bool,
    pub(crate) folding_ranges: bool,
    pub(crate) inlay_hints: bool,
    pub(crate) file_rename: bool,
    pub(crate) cross_file: bool,
}

impl LspFeatureConfig {
    fn disabled() -> Self {
        Self {
            lint: false,
            typecheck: false,
            ecosystem: false,
            options_api: false,
            legacy_vue2: false,
            completion: false,
            hover: false,
            definition: false,
            references: false,
            document_symbols: false,
            workspace_symbols: false,
            code_actions: false,
            rename: false,
            formatting: false,
            code_lens: false,
            semantic_tokens: false,
            document_links: false,
            folding_ranges: false,
            inlay_hints: false,
            file_rename: false,
            cross_file: false,
        }
    }

    pub(crate) fn has_diagnostics(self) -> bool {
        self.lint || self.typecheck || self.ecosystem
    }

    fn apply_editor_bundle(&mut self, enabled: bool) {
        self.ecosystem = enabled;
        self.completion = enabled;
        self.hover = enabled;
        self.definition = enabled;
        self.references = enabled;
        self.document_symbols = enabled;
        self.workspace_symbols = enabled;
        self.rename = enabled;
        self.code_lens = enabled;
        self.semantic_tokens = enabled;
        self.document_links = enabled;
        self.folding_ranges = enabled;
        self.inlay_hints = enabled;
        self.file_rename = enabled;
    }
}

impl Default for LspFeatureConfig {
    fn default() -> Self {
        Self {
            lint: true,
            typecheck: true,
            ecosystem: true,
            options_api: false,
            legacy_vue2: false,
            completion: true,
            hover: true,
            definition: true,
            references: true,
            document_symbols: true,
            workspace_symbols: true,
            code_actions: true,
            rename: true,
            formatting: false,
            code_lens: true,
            semantic_tokens: true,
            document_links: true,
            folding_ranges: true,
            inlay_hints: true,
            file_rename: true,
            // Cross-file diagnostic groups are off by default — they scan
            // every Vue file in the workspace, which is too slow for the
            // default editor experience. Opt in via
            // `languageServer.crossFile = true`.
            cross_file: false,
        }
    }
}
