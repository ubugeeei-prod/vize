//! Config loading helpers.

use std::path::{Path, PathBuf};

use pklrust::{Error as PklError, EvaluatorManager, EvaluatorOptions, ModuleSource};

use super::model::{RawVizeConfig, VizeConfig};

const CONFIG_FILE_NAMES: [&str; 2] = ["vize.config.pkl", "vize.config.json"];

/// Loaded config and its source path.
#[derive(Debug, Clone)]
pub struct LoadedConfig {
    /// Effective configuration with defaults applied.
    pub config: VizeConfig,
    /// Path of the config file that was used, if any.
    pub source_path: Option<PathBuf>,
}

/// Load configuration from a directory or file path.
pub fn load_config(path: Option<&Path>) -> VizeConfig {
    load_config_with_source(path).config
}

/// Load configuration from a directory or file path and return its source path.
pub fn load_config_with_source(path: Option<&Path>) -> LoadedConfig {
    let base = path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    if let Some(file_path) = resolve_file_path(&base) {
        return LoadedConfig {
            config: parse_config_file(&file_path).unwrap_or_default(),
            source_path: Some(file_path),
        };
    }

    let Some(dir_path) = resolve_dir_path(&base) else {
        return LoadedConfig {
            config: VizeConfig::default(),
            source_path: None,
        };
    };

    for file_name in CONFIG_FILE_NAMES {
        let candidate = dir_path.join(file_name);
        if !candidate.exists() {
            continue;
        }

        if let Some(config) = try_parse_candidate(&candidate) {
            return LoadedConfig {
                config,
                source_path: Some(candidate),
            };
        }
    }

    LoadedConfig {
        config: VizeConfig::default(),
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

fn try_parse_candidate(path: &Path) -> Option<VizeConfig> {
    match parse_config_file(path) {
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
                Some(VizeConfig::default())
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

fn parse_config_file(path: &Path) -> Result<VizeConfig, Box<dyn std::error::Error>> {
    let config = match path.extension().and_then(|ext| ext.to_str()) {
        Some("pkl") => parse_pkl_config(path)?,
        Some("json") => {
            let content = std::fs::read_to_string(path)?;
            serde_json::from_str::<RawVizeConfig>(&content)?.into()
        }
        _ => return Ok(VizeConfig::default()),
    };

    Ok(config)
}

fn parse_pkl_config(path: &Path) -> Result<VizeConfig, Box<dyn std::error::Error>> {
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

fn parse_pkl_config_with_command(path: &Path, command: &Path) -> Result<VizeConfig, PklError> {
    let command = command.to_string_lossy();
    let mut manager = EvaluatorManager::with_command(command.as_ref())?;
    let options = pkl_evaluator_options(path);
    let evaluator = manager.new_evaluator(options)?;
    let result =
        manager.evaluate_module_typed::<RawVizeConfig>(&evaluator, ModuleSource::file(path));
    let _ = manager.close_evaluator(&evaluator);

    result.map(Into::into)
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
    use super::load_config_with_source;

    #[test]
    fn load_config_uses_explicit_file_path() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("custom.json");
        std::fs::write(&config_path, r#"{ "formatter": { "singleQuote": true } }"#).unwrap();

        let loaded = load_config_with_source(Some(&config_path));
        assert_eq!(loaded.source_path.as_deref(), Some(config_path.as_path()));
        assert!(loaded.config.formatter.single_quote);
    }
}
