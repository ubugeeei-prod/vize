//! One evaluation of the workspace config for an editor session.

use std::path::{Path, PathBuf};

use crate::config::{
    ConfigFeatureFlags, ConfigLintRuleOptions, LanguageServerUnstableFlags, LinterConfig,
    VizeConfig,
};

use super::load_raw_config_with_source;

/// All LSP config values derived from one raw config evaluation.
#[derive(Debug, Clone)]
pub struct LoadedLspConfig {
    pub config: VizeConfig,
    pub source_path: Option<PathBuf>,
    pub features: ConfigFeatureFlags,
    pub linter: LinterConfig,
    pub lint_rule_options: ConfigLintRuleOptions,
    pub language_server_unstable_flags: LanguageServerUnstableFlags,
    pub request_timeout_ms: u64,
}

/// Evaluate JS/TS/PKL only once so the editor timeout and type checker
/// settings always come from the same config snapshot.
pub fn load_lsp_config_snapshot(path: Option<&Path>) -> LoadedLspConfig {
    let loaded = load_raw_config_with_source(path);
    let linter = loaded.config.linter();
    let lint_rule_options = loaded.config.lint_rule_options().clone();
    let language_server_unstable_flags = loaded.config.language_server_unstable_flags();
    let request_timeout_ms = loaded.config.lsp_request_timeout_ms();
    let (config, features) = loaded.config.into_config_and_features();

    LoadedLspConfig {
        config,
        source_path: loaded.source_path,
        features,
        linter,
        lint_rule_options,
        language_server_unstable_flags,
        request_timeout_ms,
    }
}
