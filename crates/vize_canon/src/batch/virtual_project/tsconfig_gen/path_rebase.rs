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
    for name in [
        "rootDir",
        "outDir",
        "declarationDir",
        "tsBuildInfoFile",
        "mapRoot",
        "sourceRoot",
        "outFile",
    ] {
        let Some(raw) = compiler_options
            .get(name)
            .and_then(Value::as_str)
            .filter(|raw| !Path::new(raw).is_absolute())
            .filter(|raw| !matches!(name, "mapRoot" | "sourceRoot") || !is_url(raw))
        else {
            continue;
        };
        compiler_options.insert(
            name.into(),
            Value::String(normalize_tsconfig_path_target(base_dir, project_root, raw).into()),
        );
    }

    for name in ["rootDirs", "typeRoots"] {
        let Some(entries) = compiler_options.get_mut(name).and_then(Value::as_array_mut) else {
            continue;
        };
        for entry in entries {
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

    // `paths` targets are deliberately not rebased here: they anchor to the
    // effective `baseUrl` when one is declared anywhere in the chain, which is
    // only known after the whole chain merges. `paths_onto_project_root` runs
    // as a post-pass with the resolved anchor (#3886).
}

pub(super) fn is_url(value: &str) -> bool {
    let Some((scheme, _)) = value.split_once(':') else {
        return false;
    };
    scheme.len() > 1
        && scheme.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphabetic()
                || (index > 0 && (byte.is_ascii_digit() || matches!(byte, b'+' | b'-' | b'.')))
        })
}

/// Rebase relative `paths` targets from `anchor_dir` — the effective `baseUrl`
/// directory, or the winning map's declaring directory — onto the project root.
#[allow(clippy::disallowed_types)]
pub(super) fn paths_onto_project_root(
    paths: &mut Map<std::string::String, Value>,
    anchor_dir: &Path,
    project_root: &Path,
) {
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
                normalize_tsconfig_path_target(anchor_dir, project_root, raw_target).into(),
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
