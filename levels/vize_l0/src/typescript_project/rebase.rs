//! Rebasing of path-ish compiler options, first onto the project root while the
//! `extends` chain is flattened, then (for `rootDir`) onto the virtual mirror.
//!
//! A configured `rootDir` decides the emitted declaration layout, so declaration
//! emit honors it rather than inferring one. Inferring the common source
//! directory agrees with an explicit `rootDir` only while every emitted file sits
//! under it; as soon as the program also contains a file outside it, the common
//! directory collapses toward the project root and every declaration keeps its
//! source directory prefix (#3355).

use std::path::Path;

use serde_json::{Map, Value};

use super::normalize_tsconfig_path_target;

/// Rebase the relative path-ish options of a single tsconfig onto the project
/// root. They resolve against the tsconfig that declares them, so rebasing keeps
/// the declaring config's meaning once the `extends` chain is flattened into one
/// option set.
#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
pub fn onto_project_root(
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

pub fn is_url(value: &str) -> bool {
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
#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
pub fn paths_onto_project_root(
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
