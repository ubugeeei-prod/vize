//! Config loading helpers.
//!
//! This module owns discovery and high-level loading while the format-specific
//! readers live in sibling modules. The split keeps the public contract visible
//! here and avoids letting JavaScript, PKL, and path-search details crowd the
//! main flow.

mod discovery;
mod js;
mod parse;
mod pkl;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use discovery::{resolve_dir_path, resolve_file_path};
use parse::{parse_raw_config_file, try_parse_raw_candidate};

use super::model::{
    ConfigEntryFiles, ConfigEntryIgnore, ConfigFeatureFlags, LinterConfig, RawVizeConfig,
    VizeConfig,
};

const CONFIG_FILE_NAMES: [&str; 5] = [
    "vize.config.pkl",
    "vize.config.ts",
    "vize.config.js",
    "vize.config.mjs",
    "vize.config.json",
];

/// Loaded config and its source path.
#[derive(Debug, Clone)]
pub struct LoadedConfig {
    /// Effective configuration with defaults applied.
    pub config: VizeConfig,
    /// Path of the config file that was used, if any.
    pub source_path: Option<PathBuf>,
}

/// Loaded config with auxiliary feature flags.
#[derive(Debug, Clone)]
pub struct LoadedConfigWithFeatures {
    /// Effective configuration with defaults applied.
    pub config: VizeConfig,
    /// Path of the config file that was used, if any.
    pub source_path: Option<PathBuf>,
    /// Auxiliary feature flags parsed from config keys.
    pub features: ConfigFeatureFlags,
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
    config: RawVizeConfig,
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

/// Load the configured `compiler.templateSyntax` value from a directory or file path.
pub fn load_compiler_template_syntax(path: Option<&Path>) -> Option<&'static str> {
    load_raw_config_with_source(path)
        .config
        .compiler
        .template_syntax
        .map(|template_syntax| template_syntax.as_str())
}

/// Load the configured `vue.version` dialect from a directory or file path.
///
/// Returns `None` when the key is absent (modern Vue 3). Unknown or ambiguous
/// values fail config parsing earlier, so a returned value always names a valid
/// dialect. The build runner threads this into the per-file compile options so
/// it reaches the parser/transform layer.
pub fn load_compiler_vue_version(path: Option<&Path>) -> Option<crate::config::VueVersion> {
    let loaded = load_raw_config_with_source(path);
    let (_, features) = loaded.config.into_config_and_features();
    features.vue_version
}

/// Load the configured `compiler.jsxMode` default output mode (#1496).
///
/// Returns `None` when the key is absent (treated as VDOM by the JSX entry
/// points). The build runner and plugins thread this into the native
/// `compileJsx` mode-selection logic, where a per-component `"use vue:*"`
/// directive can still override it.
pub fn load_compiler_jsx_mode(path: Option<&Path>) -> Option<crate::config::JsxMode> {
    let loaded = load_raw_config_with_source(path);
    let (_, features) = loaded.config.into_config_and_features();
    features.jsx_mode
}

/// Load configuration and linter settings from a directory or file path in one pass.
///
/// The lint/check CLIs call this on every invocation. Keeping the raw config
/// around long enough to derive both `VizeConfig` and `LinterConfig` avoids
/// parsing and normalizing the same config file twice.
pub fn load_config_and_linter_with_source(path: Option<&Path>) -> (LoadedConfig, LinterConfig) {
    let loaded = load_raw_config_with_source(path);
    let linter = load_linter_from_raw_config(&loaded.config);
    let (config, _) = loaded.config.into_config_and_features();
    (
        LoadedConfig {
            config,
            source_path: loaded.source_path,
        },
        linter,
    )
}

/// Load configuration, auxiliary feature flags, and linter settings in one pass.
///
/// This is the LSP/native variant of the same optimization: a single raw parse
/// feeds stable config, feature flags, and lint settings.
pub fn load_config_and_linter_with_features_and_source(
    path: Option<&Path>,
) -> (LoadedConfigWithFeatures, LinterConfig) {
    let loaded = load_raw_config_with_source(path);
    let linter = load_linter_from_raw_config(&loaded.config);
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
    load_linter_from_raw_config(&loaded.config)
}

pub fn load_config_entry_ignores_with_source(path: Option<&Path>) -> LoadedConfigEntryIgnores {
    let loaded = load_raw_config_with_source(path);
    let top_level_ignores = loaded
        .config
        .ignores
        .as_deref()
        .unwrap_or_default()
        .iter()
        .cloned()
        .map(|pattern| ConfigEntryIgnore {
            base_path: None,
            pattern,
        });
    let entry_ignores = loaded
        .config
        .entries
        .as_deref()
        .unwrap_or_default()
        .iter()
        .flat_map(|entry| {
            entry
                .ignores
                .as_deref()
                .unwrap_or_default()
                .iter()
                .cloned()
                .map(|pattern| ConfigEntryIgnore {
                    base_path: entry.base_path.clone(),
                    pattern,
                })
        })
        .collect::<Vec<_>>();
    let ignores = top_level_ignores.chain(entry_ignores).collect();
    LoadedConfigEntryIgnores {
        ignores,
        source_path: loaded.source_path,
    }
}

pub fn load_config_entry_files_with_source(path: Option<&Path>) -> LoadedConfigEntryFiles {
    let loaded = load_raw_config_with_source(path);
    let mut entries = Vec::new();
    if let Some(files) = loaded.config.files.filter(|files| !files.is_empty()) {
        entries.push(ConfigEntryFiles {
            base_path: loaded.config.base_path,
            files,
        });
    }
    entries.extend(
        loaded
            .config
            .entries
            .unwrap_or_default()
            .into_iter()
            .filter_map(|entry| {
                entry
                    .files
                    .filter(|files| !files.is_empty())
                    .map(|files| ConfigEntryFiles {
                        base_path: entry.base_path,
                        files,
                    })
            }),
    );
    LoadedConfigEntryFiles {
        entries,
        source_path: loaded.source_path,
    }
}

fn load_linter_from_raw_config(config: &RawVizeConfig) -> LinterConfig {
    let mut linter = LinterConfig::from(config.linter.clone());
    if linter.preset.is_none() {
        linter.preset = common_entry_linter_preset(config);
    }
    linter
}

fn common_entry_linter_preset(config: &RawVizeConfig) -> Option<crate::String> {
    let mut common_preset: Option<crate::String> = None;
    for entry in config.entries.as_deref().unwrap_or_default() {
        let entry_linter = LinterConfig::from(entry.linter.clone());
        let Some(entry_preset) = entry_linter.preset else {
            continue;
        };
        if common_preset
            .as_ref()
            .is_some_and(|preset| preset.as_str() != entry_preset.as_str())
        {
            return None;
        }
        common_preset = Some(entry_preset);
    }
    common_preset
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
            config: RawVizeConfig::default(),
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
        config: RawVizeConfig::default(),
        source_path: None,
    }
}
