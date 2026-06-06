//! PKL config evaluation.
//!
//! PKL support prefers project-local binaries so workspaces get reproducible
//! evaluation without requiring a globally installed `pkl`. Process startup
//! failures are reported separately from module evaluation failures because only
//! the former should let config discovery fall through to lower-priority files.

use std::path::{Path, PathBuf};

use pklrust::{Error as PklError, EvaluatorManager, EvaluatorOptions, ModuleSource};

use crate::config::model::RawVizeConfig;

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
