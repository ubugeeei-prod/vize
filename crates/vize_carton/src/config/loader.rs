//! Config loading helpers.

use std::{
    io::{Error as IoError, ErrorKind},
    path::{Path, PathBuf},
    process::Command,
};

use pklrust::{Error as PklError, EvaluatorManager, EvaluatorOptions, ModuleSource};

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

        if let Some(config) = try_parse_candidate(&candidate) {
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

fn resolve_file_path(base: &Path) -> Option<PathBuf> {
    if base.is_file() {
        Some(base.to_path_buf())
    } else {
        None
    }
}

fn resolve_dir_path(base: &Path) -> Option<PathBuf> {
    if base.is_dir() {
        return Some(base.to_path_buf());
    }

    if base.extension().is_none() {
        return Some(base.to_path_buf());
    }

    None
}

fn try_parse_candidate(path: &Path) -> Option<RawVizeConfig> {
    try_parse_raw_candidate(path)
}

fn try_parse_raw_candidate(path: &Path) -> Option<RawVizeConfig> {
    match parse_raw_config_file(path) {
        Ok(config) => Some(config),
        Err(error) => {
            let should_try_next = should_try_next_config(path, error.as_ref());
            eprintln!(
                "\x1b[33mWarning:\x1b[0m Failed to parse {}: {}",
                path.display(),
                error
            );
            if should_try_next {
                None
            } else {
                Some(RawVizeConfig::default())
            }
        }
    }
}

fn should_try_next_config(path: &Path, error: &(dyn std::error::Error + 'static)) -> bool {
    if path.extension().and_then(|ext| ext.to_str()) != Some("pkl") {
        return true;
    }

    error
        .downcast_ref::<PklError>()
        .is_some_and(is_process_error)
}

fn parse_raw_config_file(path: &Path) -> Result<RawVizeConfig, Box<dyn std::error::Error>> {
    let config = match path.extension().and_then(|ext| ext.to_str()) {
        Some("pkl") => parse_pkl_config(path)?,
        Some("ts" | "js" | "mjs") => parse_js_config(path)?,
        Some("json") => {
            let content = std::fs::read_to_string(path)?;
            serde_json::from_str::<RawVizeConfig>(&content)?
        }
        _ => return Ok(RawVizeConfig::default()),
    };

    Ok(config)
}

fn parse_js_config(path: &Path) -> Result<RawVizeConfig, Box<dyn std::error::Error>> {
    let script = r#"
import { pathToFileURL } from "node:url";

const configPath = process.argv[1];
const module = await import(pathToFileURL(configPath).href);
const exported = module.default ?? module;
const config = typeof exported === "function"
  ? await exported({ mode: "development", command: "serve" })
  : exported;
process.stdout.write(JSON.stringify(config ?? {}));
"#;
    let output = Command::new("node")
        .arg("--input-type=module")
        .arg("-e")
        .arg(script)
        .arg(path)
        .current_dir(path.parent().unwrap_or_else(|| Path::new(".")))
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Box::new(IoError::new(
            ErrorKind::InvalidData,
            crate::cstr!("node failed to load config: {}", stderr.trim()).to_string(),
        )));
    }

    Ok(serde_json::from_slice::<RawVizeConfig>(&output.stdout)?)
}

fn parse_pkl_config(path: &Path) -> Result<RawVizeConfig, Box<dyn std::error::Error>> {
    let mut last_process_error = None;

    for command in pkl_command_candidates(path) {
        match parse_pkl_config_with_command(path, &command) {
            Ok(config) => return Ok(config),
            Err(error) if is_process_error(&error) => {
                last_process_error = Some(error);
            }
            Err(error) => return Err(Box::new(error)),
        }
    }

    Err(last_process_error
        .map(|error| Box::new(error) as Box<dyn std::error::Error>)
        .unwrap_or_else(|| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "failed to locate a usable pkl command",
            ))
        }))
}

fn parse_pkl_config_with_command(path: &Path, command: &Path) -> Result<RawVizeConfig, PklError> {
    let command = command.to_string_lossy();
    let mut manager = EvaluatorManager::with_command(command.as_ref())?;
    let options = pkl_evaluator_options(path);
    let evaluator = manager.new_evaluator(options)?;
    let result =
        manager.evaluate_module_typed::<RawVizeConfig>(&evaluator, ModuleSource::file(path));
    let _ = manager.close_evaluator(&evaluator);

    result
}

fn pkl_evaluator_options(path: &Path) -> EvaluatorOptions {
    let Some(root_dir) = path.parent() else {
        return EvaluatorOptions::preconfigured();
    };

    let root_dir = root_dir.to_string_lossy();
    EvaluatorOptions::preconfigured().root_dir(root_dir.as_ref())
}

fn is_process_error(error: &PklError) -> bool {
    matches!(error, PklError::Io(_) | PklError::Process(_))
}

fn pkl_command_candidates(path: &Path) -> Vec<PathBuf> {
    let mut commands = Vec::with_capacity(9);

    push_pkl_command_candidates(&mut commands, path);

    if let Ok(current_dir) = std::env::current_dir() {
        push_pkl_command_candidates(&mut commands, &current_dir);
    }

    commands.push(PathBuf::from("pkl"));
    commands
}

fn push_pkl_command_candidates(commands: &mut Vec<PathBuf>, path: &Path) {
    let search_root = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };

    for ancestor in search_root.ancestors() {
        for binary in local_pkl_candidates(ancestor) {
            if binary.exists() && !commands.iter().any(|command| command == &binary) {
                commands.push(binary);
            }
        }
    }
}

fn local_pkl_candidates(base: &Path) -> [PathBuf; 6] {
    [
        base.join("node_modules/.bin/pkl"),
        base.join("node_modules/.bin/pkl.cmd"),
        base.join("node_modules/.pnpm/node_modules/.bin/pkl"),
        base.join("node_modules/.pnpm/node_modules/.bin/pkl.cmd"),
        base.join("node_modules/@pkl-community/pkl/pkl"),
        base.join("node_modules/@pkl-community/pkl/pkl.exe"),
    ]
}
#[cfg(test)]
mod tests {
    use super::{
        load_config_and_linter_with_source, load_config_with_source, load_linter_config,
        validate_explicit_config_path,
    };

    #[test]
    fn validate_explicit_config_path_missing_errors() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("does-not-exist.toml");

        let result = validate_explicit_config_path(&missing);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("config file not found"));
    }

    #[test]
    fn validate_explicit_config_path_malformed_errors() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("vize.config.json");
        std::fs::write(&config_path, "this is { not valid json ===").unwrap();

        let result = validate_explicit_config_path(&config_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("failed to parse"));
    }

    #[test]
    fn validate_explicit_config_path_valid_ok() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("vize.config.json");
        std::fs::write(&config_path, r#"{ "formatter": { "singleQuote": true } }"#).unwrap();

        assert!(validate_explicit_config_path(&config_path).is_ok());
    }

    #[test]
    fn validate_explicit_config_path_dir_with_config_ok() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("vize.config.json"),
            r#"{ "formatter": {} }"#,
        )
        .unwrap();

        assert!(validate_explicit_config_path(dir.path()).is_ok());
    }

    #[test]
    fn validate_explicit_config_path_empty_dir_errors() {
        let dir = tempfile::tempdir().unwrap();
        let result = validate_explicit_config_path(dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no vize config file found"));
    }

    #[test]
    fn load_config_uses_explicit_file_path() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("custom.json");
        std::fs::write(&config_path, r#"{ "formatter": { "singleQuote": true } }"#).unwrap();

        let loaded = load_config_with_source(Some(&config_path));
        assert_eq!(loaded.source_path.as_deref(), Some(config_path.as_path()));
        assert!(loaded.config.formatter.single_quote);
    }

    #[test]
    fn load_config_uses_typescript_config() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("vize.config.ts");
        std::fs::write(
            &config_path,
            r#"
export default {
  linter: {
    rules: {
      "vue/prop-name-casing": "off",
      "script/no-options-api": "error",
    },
  },
}
"#,
        )
        .unwrap();

        let (loaded, linter_from_loaded_config) =
            load_config_and_linter_with_source(Some(dir.path()));
        let linter = load_linter_config(Some(dir.path()));

        assert_eq!(loaded.source_path.as_deref(), Some(config_path.as_path()));
        assert_eq!(
            linter_from_loaded_config.disabled_rules(),
            ["vue/prop-name-casing"]
        );
        assert_eq!(
            linter_from_loaded_config.enabled_rules(),
            ["script/no-options-api"]
        );
        assert_eq!(linter.disabled_rules(), ["vue/prop-name-casing"]);
        assert_eq!(linter.enabled_rules(), ["script/no-options-api"]);
    }
}
