//! Fail closed on invalid discovered configuration for command execution.
use std::path::{Path, PathBuf};

use super::{
    LoadedRawConfig,
    discovery::{CONFIG_FILE_NAMES, search_directories},
    parse::parse_raw_config_file,
    pkl, project_defaults,
};

pub(super) fn load_raw_config_checked(
    path: Option<&Path>,
) -> Result<LoadedRawConfig, std::string::String> {
    load_raw_checked(path, true)
}

/// An editor workspace directory can legitimately have no dedicated config.
pub(super) fn load_raw_editor_config_checked(
    path: Option<&Path>,
) -> Result<LoadedRawConfig, std::string::String> {
    load_raw_checked(path, false)
}

fn load_raw_checked(
    path: Option<&Path>,
    require_explicit_config: bool,
) -> Result<LoadedRawConfig, std::string::String> {
    let base = path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    if !base.exists() {
        return Err(crate::cstr!("config file not found: {}", base.display()).into());
    }
    if base.is_file() {
        return parse(&base);
    }
    for directory in search_directories(&base, path.is_none()) {
        for name in CONFIG_FILE_NAMES {
            let candidate = directory.join(name);
            if !candidate.exists() {
                continue;
            }
            match parse_raw_config_file(&candidate) {
                Ok(config) => {
                    return Ok(LoadedRawConfig {
                        config,
                        source_path: Some(candidate),
                    });
                }
                Err(error)
                    if path.is_none()
                        && candidate.extension().is_some_and(|ext| ext == "pkl")
                        && pkl::is_process_error_box(error.as_ref()) =>
                {
                    eprintln!(
                        "Warning: Failed to evaluate {}: {error}",
                        candidate.display()
                    );
                }
                Err(error) => {
                    return Err(
                        crate::cstr!("failed to parse {}: {error}", candidate.display()).into(),
                    );
                }
            }
        }
    }
    if path.is_some() && require_explicit_config {
        return Err(crate::cstr!("no vize config file found under {}", base.display()).into());
    }
    Ok(LoadedRawConfig {
        config: project_defaults(),
        source_path: None,
    })
}

fn parse(path: &Path) -> Result<LoadedRawConfig, std::string::String> {
    parse_raw_config_file(path)
        .map(|config| LoadedRawConfig {
            config,
            source_path: Some(path.to_path_buf()),
        })
        .map_err(|error| crate::cstr!("failed to parse {}: {error}", path.display()).into())
}
