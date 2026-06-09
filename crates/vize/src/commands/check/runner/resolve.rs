//! Project-root, `tsconfig`, declaration-output, and miscellaneous path
//! resolution helpers for the `check` runner.

use std::path::{Path, PathBuf};

use vize_canon::DeclarationEmitOptions;
use vize_carton::String;

use crate::commands::check::tsconfig_inputs::{
    TsconfigDeclarationOptions, load_tsconfig_declaration_options,
};

pub(super) fn resolve_declaration_emit_options(
    declaration_dir: Option<&Path>,
    tsconfig_path: Option<&Path>,
    project_root: &Path,
) -> DeclarationEmitOptions {
    let tsconfig_options = tsconfig_path
        .map(load_tsconfig_declaration_options)
        .unwrap_or_default();
    let out_dir = resolve_declaration_dir(declaration_dir, &tsconfig_options, project_root);

    DeclarationEmitOptions::new(out_dir)
        .with_declaration_map(tsconfig_options.declaration_map.unwrap_or(false))
}

pub(super) fn resolve_declaration_dir(
    declaration_dir: Option<&Path>,
    tsconfig_options: &TsconfigDeclarationOptions,
    project_root: &Path,
) -> PathBuf {
    declaration_dir
        .map(|path| {
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                project_root.join(path)
            }
        })
        .or_else(|| tsconfig_options.output_dir().map(Path::to_path_buf))
        .unwrap_or_else(|| project_root.join("dist").join("types"))
}

pub(super) fn resolve_project_root(
    explicit_tsconfig: Option<&Path>,
    cwd: &Path,
    files: &[PathBuf],
) -> PathBuf {
    if let Some(tsconfig) = explicit_tsconfig {
        let tsconfig_path = if tsconfig.is_absolute() {
            tsconfig.to_path_buf()
        } else {
            cwd.join(tsconfig)
        };
        let tsconfig_dir = tsconfig_path
            .canonicalize()
            .unwrap_or(tsconfig_path)
            .parent()
            .map(|parent| parent.to_path_buf())
            .unwrap_or_else(|| cwd.to_path_buf());
        if files.is_empty() {
            return tsconfig_dir;
        }

        return common_project_root(tsconfig_dir, files);
    }

    if let Some(root) = resolve_project_root_from_files(files) {
        return root;
    }

    if let Some(root) = find_nearest_tsconfig_dir(cwd) {
        return root;
    }

    cwd.to_path_buf()
}

pub(super) fn resolve_tsconfig_path(
    explicit_tsconfig: Option<&Path>,
    cwd: &Path,
    project_root: &Path,
    files: &[PathBuf],
) -> Option<PathBuf> {
    if let Some(tsconfig) = explicit_tsconfig {
        let tsconfig_path = if tsconfig.is_absolute() {
            tsconfig.to_path_buf()
        } else {
            cwd.join(tsconfig)
        };
        return Some(tsconfig_path.canonicalize().unwrap_or(tsconfig_path));
    }

    let candidate = project_root.join("tsconfig.json");
    if candidate.exists() {
        return Some(candidate);
    }

    for file in files {
        let Some(root) = find_nearest_tsconfig_dir(file) else {
            continue;
        };
        let candidate = root.join("tsconfig.json");
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

pub(super) fn find_nearest_tsconfig_dir(path: &Path) -> Option<PathBuf> {
    let mut current = if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    };

    while let Some(dir) = current {
        if dir.join("tsconfig.json").exists() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }

    None
}

fn resolve_project_root_from_files(files: &[PathBuf]) -> Option<PathBuf> {
    let common = common_file_parent(files)?;
    Some(find_nearest_tsconfig_dir(&common).unwrap_or(common))
}

fn common_file_parent(files: &[PathBuf]) -> Option<PathBuf> {
    let mut common = files
        .first()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf)?;

    for file in &files[1..] {
        let parent = file.parent().unwrap_or(file.as_path());
        while !parent.starts_with(&common) {
            if !common.pop() {
                return None;
            }
        }
    }

    Some(common)
}

fn common_project_root(mut common: PathBuf, files: &[PathBuf]) -> PathBuf {
    for file in files {
        let parent = file.parent().unwrap_or(file.as_path());
        while !parent.starts_with(&common) {
            if !common.pop() {
                return common;
            }
        }
    }

    common
}

pub(super) fn display_path(base: &Path, path: &Path) -> vize_carton::String {
    use vize_carton::cstr;

    path.strip_prefix(base)
        .map(|relative| cstr!("{}", relative.display()))
        .unwrap_or_else(|_| cstr!("{}", path.display()))
}

pub(super) fn resolve_from_config_dir(config_dir: &Path, candidate: &str) -> PathBuf {
    let path = Path::new(candidate);
    if path.is_absolute() {
        return path.to_path_buf();
    }

    config_dir.join(path)
}

pub(super) fn validate_corsa_server_count(servers: Option<usize>) -> Result<(), String> {
    let Some(servers) = servers else {
        return Ok(());
    };
    if servers == 0 {
        return Err("typeChecker.servers must be at least 1.".into());
    }
    if servers > 1 {
        return Err(format!(
            "typeChecker.servers={servers} is not supported by the direct Corsa project-session runner yet; use 1 or omit the option."
        )
        .into());
    }
    Ok(())
}
