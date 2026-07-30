//! Rebasing of path-ish compiler options, first onto the project root while the
//! `extends` chain is flattened, then (for `rootDir`) onto the virtual mirror.
//!
//! A configured `rootDir` decides the emitted declaration layout, so declaration
//! emit honors it rather than inferring one. Inferring the common source
//! directory agrees with an explicit `rootDir` only while every emitted file sits
//! under it; as soon as the program also contains a file outside it, the common
//! directory collapses toward the project root and every declaration keeps its
//! source directory prefix (#3355).

use std::path::{Component, Path, PathBuf};

use serde_json::{Map, Value};

use super::super::tsconfig_paths::normalize_tsconfig_path_target;

/// Rebase the relative path-ish options of a single tsconfig onto the project
/// root. They resolve against the tsconfig that declares them, so rebasing keeps
/// the declaring config's meaning once the `extends` chain is flattened into one
/// option set.
#[allow(clippy::disallowed_types)]
pub(super) fn onto_project_root(
    compiler_options: &mut Map<std::string::String, Value>,
    base_dir: &Path,
    project_root: &Path,
) {
    let relative_root_dir = compiler_options
        .get("rootDir")
        .and_then(Value::as_str)
        .filter(|root_dir| !Path::new(root_dir).is_absolute())
        .map(|root_dir| normalize_tsconfig_path_target(base_dir, project_root, root_dir));
    if let Some(root_dir) = relative_root_dir {
        compiler_options.insert("rootDir".into(), Value::String(root_dir.into()));
    }

    if let Some(type_roots) = compiler_options
        .get_mut("typeRoots")
        .and_then(Value::as_array_mut)
    {
        for entry in type_roots {
            let Some(raw_entry) = entry.as_str() else {
                continue;
            };
            if Path::new(raw_entry).is_absolute() {
                continue;
            }
            *entry = Value::String(
                normalize_tsconfig_path_target(base_dir, project_root, raw_entry).into(),
            );
        }
    }

    let Some(paths) = compiler_options
        .get_mut("paths")
        .and_then(Value::as_object_mut)
    else {
        return;
    };

    for targets in paths.values_mut() {
        let Some(targets) = targets.as_array_mut() else {
            continue;
        };
        for target in targets {
            let Some(raw_target) = target.as_str() else {
                continue;
            };
            if Path::new(raw_target).is_absolute() {
                continue;
            }
            *target = Value::String(
                normalize_tsconfig_path_target(base_dir, project_root, raw_target).into(),
            );
        }
    }
}

/// Rebase a configured `rootDir` onto the virtual mirror.
///
/// The mirror reproduces paths relative to the project root, so a `rootDir` of
/// `./lib` becomes `<virtual_root>/lib`. Relative values are already rebased onto
/// the project root by [`onto_project_root`], which resolves them against the
/// tsconfig that declares them, so an inherited `rootDir` keeps the base config's
/// meaning. Returns `None` when nothing is configured, or when the configured
/// directory falls outside the project root and therefore has no mirror
/// counterpart — in each case the caller keeps inferring the layout.
pub(super) fn root_dir_into_mirror(
    project_root: &Path,
    virtual_root: &Path,
    configured: Option<&str>,
) -> Option<PathBuf> {
    let configured = Path::new(configured?);
    let absolute = if configured.is_absolute() {
        configured.to_path_buf()
    } else {
        project_root.join(configured)
    };
    let absolute = vize_carton::path::canonicalize_non_verbatim(&absolute);
    let relative = absolute.strip_prefix(project_root).ok()?;
    // `canonicalize` returns the input untouched for a directory that does not
    // exist, so `..` can survive and `strip_prefix` still matches lexically
    // (`<root>/../outside` starts with `<root>`). Such a path has no mirror
    // counterpart.
    if relative
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return None;
    }
    Some(virtual_root.join(relative))
}
