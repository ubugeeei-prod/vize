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

use super::model::{ConfigFeatureFlags, LinterConfig, RawVizeConfig, VizeConfig};

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

/// Load configuration and linter settings from a directory or file path in one pass.
///
/// The lint/check CLIs call this on every invocation. Keeping the raw config
/// around long enough to derive both `VizeConfig` and `LinterConfig` avoids
/// parsing and normalizing the same config file twice.
pub fn load_config_and_linter_with_source(path: Option<&Path>) -> (LoadedConfig, LinterConfig) {
    let loaded = load_raw_config_with_source(path);
    let linter = loaded.config.linter.clone();
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
    let linter = loaded.config.linter.clone();
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
    load_raw_config_with_source(path).config.linter
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
