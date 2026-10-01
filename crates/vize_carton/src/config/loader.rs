//! Config loading helpers.
//!
//! This module owns discovery and high-level loading while the format-specific
//! readers live in sibling modules. The split keeps the public contract visible
//! here and avoids letting JavaScript, PKL, and path-search details crowd the
//! main flow.

mod checked;
mod compiler_keys;
mod formatter;
pub use formatter::load_config_with_formatter_options_and_source;
mod discovery;
#[cfg(test)]
mod experimental_tests;
mod js;
mod jsx;
mod language_server;
#[cfg(test)]
mod legacy_dialect_tests;
mod library;
mod lint_features;
mod lsp_snapshot;
mod parse;
mod pkl;
#[cfg(test)]
mod tests;
mod vapor;

use std::path::{Path, PathBuf};

use checked::load_raw_config_checked;
use discovery::{CONFIG_FILE_NAMES, resolve_dir_path, resolve_file_path};
use parse::{parse_raw_config_file, try_parse_raw_candidate};

use super::{
    ConfigDocument, ConfigEntryFiles, ConfigEntryIgnore, ConfigExperimentalVueFlags,
    ConfigFeatureFlags, LinterConfig, VizeConfig,
};
pub use compiler_keys::*;
pub use library::{LoadedLibConfig, load_lib_config_with_source};
pub use lsp_snapshot::{LoadedLspConfig, load_lsp_config_snapshot};
pub use {jsx::load_compiler_jsx_compat, vapor::load_compiler_vapor};
pub use {language_server::*, lint_features::*};

#[derive(Debug, Clone)]
pub struct LoadedConfig {
    pub config: VizeConfig,
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct LoadedConfigWithFeatures {
    pub config: VizeConfig,
    pub source_path: Option<PathBuf>,
    pub features: ConfigFeatureFlags,
}

#[derive(Debug, Clone, Default)]
pub struct LoadedConfigExperimentalVueFlags {
    pub source_path: Option<PathBuf>,
    pub flags: ConfigExperimentalVueFlags,
}

#[derive(Debug, Clone)]
pub struct LoadedConfigEntryIgnores {
    pub ignores: Vec<ConfigEntryIgnore>,
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct LoadedConfigEntryFiles {
    pub entries: Vec<ConfigEntryFiles>,
    pub source_path: Option<PathBuf>,
}

struct LoadedRawConfig {
    config: ConfigDocument,
    source_path: Option<PathBuf>,
}

/// Validate that an explicitly-provided `--config` path exists and parses.
///
/// Auto-discovery silently falls back to defaults when no config is found
/// or a candidate fails to parse, but an explicit `--config <path>` must
/// hard-error so CI/scripts don't silently run with the wrong rules (#970).
///
/// Returns `Ok(())` if the path resolves to a parseable config file (or to
/// a directory that contains one). Returns `Err(message)` if the path is
/// missing, points at a non-config file with no parseable form, or fails
/// to parse.
pub fn validate_explicit_config_path(path: &Path) -> Result<(), std::string::String> {
    let display = path.display();
    if !path.exists() {
        return Err(crate::cstr!("config file not found: {display}").into());
    }

    if path.is_file() {
        return parse_raw_config_file(path)
            .map(|_| ())
            .map_err(|error| crate::cstr!("failed to parse {display}: {error}").into());
    }

    if path.is_dir() {
        for file_name in CONFIG_FILE_NAMES {
            let candidate = path.join(file_name);
            if candidate.exists() {
                let candidate_display = candidate.display();
                return parse_raw_config_file(&candidate)
                    .map(|_| ())
                    .map_err(|error| {
                        crate::cstr!("failed to parse {candidate_display}: {error}").into()
                    });
            }
        }
        return Err(crate::cstr!("no vize config file found under {display}").into());
    }

    Err(crate::cstr!("config path is neither a file nor a directory: {display}").into())
}

/// Load configuration from a directory or file path.
pub fn load_config(path: Option<&Path>) -> VizeConfig {
    load_config_with_source(path).config
}

/// Load configuration from a directory or file path and return its source path.
pub fn load_config_with_source(path: Option<&Path>) -> LoadedConfig {
    let loaded = load_raw_config_with_source(path);
    let (config, _) = loaded.config.into_config_and_features();
    LoadedConfig {
        config,
        source_path: loaded.source_path,
    }
}

/// Load configuration and auxiliary feature flags from a directory or file path.
pub fn load_config_with_features_and_source(path: Option<&Path>) -> LoadedConfigWithFeatures {
    let loaded = load_raw_config_with_source(path);
    let (config, features) = loaded.config.into_config_and_features();
    LoadedConfigWithFeatures {
        config,
        source_path: loaded.source_path,
        features,
    }
}

/// Load only the opt-in Vue RFC experimental flags from configuration.
pub fn load_config_experimental_vue_flags_with_source(
    path: Option<&Path>,
) -> LoadedConfigExperimentalVueFlags {
    let loaded = load_raw_config_with_source(path);
    LoadedConfigExperimentalVueFlags {
        flags: loaded.config.experimental_vue_flags(),
        source_path: loaded.source_path,
    }
}

/// Load configuration and linter settings in one pass (one raw parse derives
/// both `VizeConfig` and `LinterConfig`, avoiding a double parse on every CLI run).
pub fn load_config_and_linter_with_source(path: Option<&Path>) -> (LoadedConfig, LinterConfig) {
    let loaded = load_raw_config_with_source(path);
    let linter = loaded.config.linter();
    let (config, _) = loaded.config.into_config_and_features();
    (
        LoadedConfig {
            config,
            source_path: loaded.source_path,
        },
        linter,
    )
}

/// Load config, feature flags, and linter settings in one raw parse (LSP/native variant).
pub fn load_config_and_linter_with_features_and_source(
    path: Option<&Path>,
) -> (LoadedConfigWithFeatures, LinterConfig) {
    let loaded = load_raw_config_with_source(path);
    let linter = loaded.config.linter();
    let (config, features) = loaded.config.into_config_and_features();
    (
        LoadedConfigWithFeatures {
            config,
            source_path: loaded.source_path,
            features,
        },
        linter,
    )
}

/// Load linter-specific configuration from a directory or file path.
pub fn load_linter_config(path: Option<&Path>) -> LinterConfig {
    let loaded = load_raw_config_with_source(path);
    loaded.config.linter()
}

/// Load the typed per-rule lint options (`linter.ruleOptions`); defaults when unset.
pub fn load_config_lint_rule_options(path: Option<&Path>) -> crate::config::ConfigLintRuleOptions {
    let loaded = load_raw_config_with_source(path);
    loaded.config.lint_rule_options().clone()
}

/// Load the stable per-rule lint options subset; defaults when unset.
pub fn load_linter_rule_options(path: Option<&Path>) -> crate::config::LintRuleOptions {
    let loaded = load_raw_config_with_source(path);
    loaded.config.lint_rule_options().stable_options().clone()
}

pub fn load_config_entry_ignores_with_source(path: Option<&Path>) -> LoadedConfigEntryIgnores {
    let loaded = load_raw_config_with_source(path);
    let ignores = loaded.config.entry_ignores();
    LoadedConfigEntryIgnores {
        ignores,
        source_path: loaded.source_path,
    }
}

pub fn load_config_entry_files_with_source(path: Option<&Path>) -> LoadedConfigEntryFiles {
    let loaded = load_raw_config_with_source(path);
    let entries = loaded.config.into_entry_files();
    LoadedConfigEntryFiles {
        entries,
        source_path: loaded.source_path,
    }
}

fn load_raw_config_with_source(path: Option<&Path>) -> LoadedRawConfig {
    let base = path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    if let Some(file_path) = resolve_file_path(&base) {
        return LoadedRawConfig {
            config: parse_raw_config_file(&file_path).unwrap_or_default(),
            source_path: Some(file_path),
        };
    }

    let Some(dir_path) = resolve_dir_path(&base) else {
        return LoadedRawConfig {
            config: ConfigDocument::default(),
            source_path: None,
        };
    };

    for file_name in CONFIG_FILE_NAMES {
        let candidate = dir_path.join(file_name);
        if !candidate.exists() {
            continue;
        }

        if let Some(config) = try_parse_raw_candidate(&candidate) {
            return LoadedRawConfig {
                config,
                source_path: Some(candidate),
            };
        }
    }

    LoadedRawConfig {
        config: ConfigDocument::default(),
        source_path: None,
    }
}
