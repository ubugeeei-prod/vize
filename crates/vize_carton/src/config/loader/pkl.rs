//! PKL config evaluation.
//!
//! PKL support prefers project-local binaries so workspaces get reproducible
//! evaluation without requiring a globally installed `pkl`. Process startup
//! failures are reported separately from module evaluation failures because only
//! the former should let config discovery fall through to lower-priority files.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::model::RawVizeConfig;

#[derive(Debug)]
enum PklError {
    Process(std::io::Error),
    Eval(crate::String),
    Json(serde_json::Error),
}

impl fmt::Display for PklError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Process(error) => error.fmt(f),
            Self::Eval(error) => write!(f, "pkl eval failed: {error}"),
            Self::Json(error) => write!(f, "pkl eval produced invalid JSON: {error}"),
        }
    }
}

impl std::error::Error for PklError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Process(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Eval(_) => None,
        }
    }
}

/// Evaluate a PKL config and deserialize it into the raw config model.
pub(super) fn parse_pkl_config(path: &Path) -> Result<RawVizeConfig, Box<dyn std::error::Error>> {
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

/// Return true when a boxed parse error came from PKL process startup.
pub(super) fn is_process_error_box(error: &(dyn std::error::Error + 'static)) -> bool {
    error
        .downcast_ref::<PklError>()
        .is_some_and(is_process_error)
}

fn parse_pkl_config_with_command(path: &Path, command: &Path) -> Result<RawVizeConfig, PklError> {
    let mut process = Command::new(command);
    process.arg("eval").arg("-f").arg("json").arg(path);
    if let Some(parent) = path.parent() {
        process.current_dir(parent);
    }
    let output = process.output().map_err(PklError::Process)?;
    if !output.status.success() {
        return Err(PklError::Eval(format_pkl_failure(&output)));
    }
    serde_json::from_slice::<RawVizeConfig>(&output.stdout).map_err(PklError::Json)
}

fn is_process_error(error: &PklError) -> bool {
    matches!(error, PklError::Process(_))
}

fn format_pkl_failure(output: &std::process::Output) -> crate::String {
    let stderr = crate::cstr!("{}", String::from_utf8_lossy(&output.stderr).trim());
    if stderr.is_empty() {
        let stdout = crate::cstr!("{}", String::from_utf8_lossy(&output.stdout).trim());
        if stdout.is_empty() {
            output
                .status
                .code()
                .map(|code| crate::cstr!("exit code {code}"))
                .unwrap_or_else(|| "terminated by signal".into())
        } else {
            stdout
        }
    } else {
        stderr
    }
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
