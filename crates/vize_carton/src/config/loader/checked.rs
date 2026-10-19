//! Fail closed on invalid discovered configuration for command execution.
use std::path::{Path, PathBuf};

use super::{
    BeforeConfigRead, ConfigLoadFailureIdentity, LoadedRawConfig,
    discovery::{
        CONFIG_FILE_NAMES, CONFIG_MIGRATION, retired_config_paths, search_directories_observed,
    },
    project_defaults,
};

pub(super) fn load_raw_config_checked(
    path: Option<&Path>,
) -> Result<LoadedRawConfig, std::string::String> {
    load_raw_checked(
        path,
        true,
        &mut Vec::new(),
        &mut true,
        &mut None,
        &mut None,
        None,
    )
}

/// An editor workspace directory can legitimately have no dedicated config.
pub(super) fn load_raw_editor_config_checked(
    path: Option<&Path>,
) -> Result<LoadedRawConfig, std::string::String> {
    load_raw_editor_config_attempt(path).outcome
}

/// Failed resolver observations are repair watches, never configuration values.
pub(super) struct EditorConfigAttempt {
    pub outcome: Result<LoadedRawConfig, std::string::String>,
    pub dependencies: Vec<PathBuf>,
    pub dependencies_complete: bool,
    pub failure_identity: Option<ConfigLoadFailureIdentity>,
}

pub(super) fn load_raw_editor_config_attempt(path: Option<&Path>) -> EditorConfigAttempt {
    load_raw_editor_config_attempt_observed(path, None)
}

pub(super) fn load_raw_editor_config_attempt_observed(
    path: Option<&Path>,
    before_read: BeforeConfigRead<'_>,
) -> EditorConfigAttempt {
    load_raw_editor_config_attempt_with_compile_cache(path, before_read, None)
}

pub(super) fn load_raw_editor_config_attempt_with_compile_cache(
    path: Option<&Path>,
    mut before_read: BeforeConfigRead<'_>,
    compile_cache: Option<&Path>,
) -> EditorConfigAttempt {
    let mut dependencies = Vec::new();
    let mut dependencies_complete = true;
    let mut failure_identity = None;
    let outcome = load_raw_checked(
        path,
        false,
        &mut dependencies,
        &mut dependencies_complete,
        &mut before_read,
        &mut failure_identity,
        compile_cache,
    );
    dependencies.sort();
    dependencies.dedup();
    EditorConfigAttempt {
        outcome,
        dependencies,
        dependencies_complete,
        failure_identity,
    }
}

fn load_raw_checked(
    path: Option<&Path>,
    require_explicit_config: bool,
    dependencies: &mut Vec<PathBuf>,
    complete: &mut bool,
    before_read: &mut BeforeConfigRead<'_>,
    failure_identity: &mut Option<ConfigLoadFailureIdentity>,
    compile_cache: Option<&Path>,
) -> Result<LoadedRawConfig, std::string::String> {
    let base = path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    if !base.is_dir()
        && let Some(observe) = before_read.as_deref_mut()
    {
        observe(std::slice::from_ref(&base));
    }
    dependencies.push(base.clone());
    if !base.exists() {
        return Err(crate::cstr!("config file not found: {}", base.display()).into());
    }
    if base.is_file() {
        return parse(
            &base,
            dependencies,
            complete,
            before_read,
            failure_identity,
            compile_cache,
        );
    }
    // The directory's children are the actual discovery inputs. Keeping its
    // mtime would turn unrelated source-file creation into config changes.
    dependencies.pop();
    for directory in search_directories_observed(&base, path.is_none(), before_read) {
        for name in CONFIG_FILE_NAMES {
            let candidate = directory.join(name);
            if let Some(observe) = before_read.as_deref_mut() {
                observe(std::slice::from_ref(&candidate));
            }
            dependencies.push(candidate.clone());
            if !candidate.exists() {
                continue;
            }
            match super::parse::parse_raw_config_file_with_compile_cache(
                &candidate,
                dependencies,
                complete,
                before_read,
                compile_cache,
            ) {
                Ok(config) => {
                    return Ok(LoadedRawConfig {
                        config,
                        source_path: Some(candidate),
                    });
                }
                Err(error) => {
                    *failure_identity = super::failure_identity::from_error(error.as_ref());
                    return Err(
                        crate::cstr!("failed to parse {}: {error}", candidate.display()).into(),
                    );
                }
            }
        }
        if let Some(observe) = before_read.as_deref_mut() {
            let retired_candidates = ["pkl", "ts", "mts", "cts", "js", "mjs", "cjs", "json"]
                .map(|extension| directory.join(crate::cstr!("vize.config.{extension}").as_str()));
            observe(&retired_candidates);
        }
        let retired = retired_config_paths(&directory);
        if !retired.is_empty() {
            dependencies.extend(retired);
            return Err(CONFIG_MIGRATION.into());
        }
    }
    if path.is_some() && require_explicit_config {
        return Err(crate::cstr!("no Vite config file found under {}", base.display()).into());
    }
    Ok(LoadedRawConfig {
        config: project_defaults(),
        source_path: None,
    })
}

fn parse(
    path: &Path,
    dependencies: &mut Vec<PathBuf>,
    complete: &mut bool,
    before_read: &mut BeforeConfigRead<'_>,
    failure_identity: &mut Option<ConfigLoadFailureIdentity>,
    compile_cache: Option<&Path>,
) -> Result<LoadedRawConfig, std::string::String> {
    super::parse::parse_raw_config_file_with_compile_cache(
        path,
        dependencies,
        complete,
        before_read,
        compile_cache,
    )
    .map(|config| LoadedRawConfig {
        config,
        source_path: Some(path.to_path_buf()),
    })
    .map_err(|error| {
        *failure_identity = super::failure_identity::from_error(error.as_ref());
        crate::cstr!("failed to parse {}: {error}", path.display()).into()
    })
}
