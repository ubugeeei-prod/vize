//! Self-contained snapshots of effective TypeScript compiler options.

use std::path::Path;

use serde_json::{Map, Value};

use super::super::VirtualProject;
use super::compiler_options::FlattenedCompilerOptions;
use super::path_rebase::is_url;
use crate::batch::error::CorsaResult;

/// Snapshot the effective compiler options of a tsconfig chain into one
/// self-contained map. The returned options never depend on the directory of a
/// generated config: every path-bearing option is absolute. Options Canon
/// deliberately removes from the executable project remain in this snapshot
/// so the input-less option probe can still report authored config errors.
///
/// This keeps package `extends`, array `extends`, cycle handling, and
/// diamond-graph memoization in the same authority used by batch checking.
#[allow(clippy::disallowed_types)]
pub fn snapshot_tsconfig_compiler_options(
    project_root: &Path,
    tsconfig_path: &Path,
) -> CorsaResult<Map<std::string::String, Value>> {
    let project = VirtualProject::new(project_root)?;
    let FlattenedCompilerOptions {
        mut options,
        base_url,
        ..
    } = project.load_compiler_options_flattened(Some(tsconfig_path))?;

    absolutize_string_option(&mut options, "rootDir", &project.project_root);
    absolutize_string_option(&mut options, "outDir", &project.project_root);
    absolutize_string_option(&mut options, "declarationDir", &project.project_root);
    absolutize_string_option(&mut options, "tsBuildInfoFile", &project.project_root);
    absolutize_string_option(&mut options, "mapRoot", &project.project_root);
    absolutize_string_option(&mut options, "sourceRoot", &project.project_root);
    absolutize_string_option(&mut options, "outFile", &project.project_root);
    absolutize_array_option(&mut options, "rootDirs", &project.project_root);
    absolutize_array_option(&mut options, "typeRoots", &project.project_root);
    absolutize_paths(&mut options, &project.project_root);
    if let Some(base_url) = base_url {
        options.insert(
            "baseUrl".into(),
            Value::String(absolute_project_target(&project.project_root, &base_url).into()),
        );
    }
    Ok(options)
}

#[allow(clippy::disallowed_types)]
fn absolutize_string_option(
    options: &mut Map<std::string::String, Value>,
    name: &str,
    project_root: &Path,
) {
    let Some(raw) = options.get(name).and_then(Value::as_str) else {
        return;
    };
    if matches!(name, "mapRoot" | "sourceRoot") && is_url(raw) {
        return;
    }
    options.insert(
        name.into(),
        Value::String(absolute_project_target(project_root, raw).into()),
    );
}

#[allow(clippy::disallowed_types)]
fn absolutize_array_option(
    options: &mut Map<std::string::String, Value>,
    name: &str,
    project_root: &Path,
) {
    let Some(entries) = options.get_mut(name).and_then(Value::as_array_mut) else {
        return;
    };
    for entry in entries {
        let Some(raw) = entry.as_str() else {
            continue;
        };
        *entry = Value::String(absolute_project_target(project_root, raw).into());
    }
}

#[allow(clippy::disallowed_types)]
fn absolutize_paths(options: &mut Map<std::string::String, Value>, project_root: &Path) {
    let Some(paths) = options.get_mut("paths").and_then(Value::as_object_mut) else {
        return;
    };
    for targets in paths.values_mut().filter_map(Value::as_array_mut) {
        for target in targets {
            let Some(raw) = target.as_str() else {
                continue;
            };
            *target = Value::String(absolute_project_target(project_root, raw).into());
        }
    }
}

fn absolute_project_target(project_root: &Path, raw: &str) -> vize_carton::String {
    let raw = Path::new(raw);
    let absolute = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        project_root.join(raw)
    };
    super::super::tsconfig_paths::normalize_path_lexically(&absolute)
        .to_string_lossy()
        .replace('\\', "/")
        .into()
}
