use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_carton::FxHashSet;

use super::super::compiler_inputs::{CompilerInputs, read_jsonc};
use super::normalize_path_lexically;

pub(super) fn collect_project_paths(tsconfig_path: &Path) -> Vec<PathBuf> {
    collect_project_paths_with_inputs(tsconfig_path, None)
}

pub(in super::super::super) fn collect_project_paths_with_inputs(
    tsconfig_path: &Path,
    inputs: Option<&CompilerInputs>,
) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen = FxHashSet::default();
    collect_project_paths_inner(tsconfig_path, &mut seen, &mut paths, inputs);
    paths
}

fn collect_project_paths_inner(
    tsconfig_path: &Path,
    seen: &mut FxHashSet<PathBuf>,
    paths: &mut Vec<PathBuf>,
    inputs: Option<&CompilerInputs>,
) {
    if let Some(inputs) = inputs {
        inputs.observe(tsconfig_path);
    }
    let resolved = normalize_path(tsconfig_path);
    if !seen.insert(resolved.clone()) {
        return;
    }
    paths.push(resolved.clone());

    let Ok(value) = read_jsonc(&resolved, inputs) else {
        return;
    };
    for reference in value
        .get("references")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.get("path").and_then(Value::as_str))
    {
        let Some(referenced) = resolve_reference(&resolved, reference, inputs) else {
            continue;
        };
        collect_project_paths_inner(&referenced, seen, paths, inputs);
    }
}

fn resolve_reference(
    tsconfig_path: &Path,
    reference: &str,
    inputs: Option<&CompilerInputs>,
) -> Option<PathBuf> {
    if let Some(inputs) = inputs {
        return inputs.resolve("reference", tsconfig_path, reference, || {
            resolve_reference_uncached(tsconfig_path, reference, Some(inputs))
        });
    }
    resolve_reference_uncached(tsconfig_path, reference, None)
}

fn resolve_reference_uncached(
    tsconfig_path: &Path,
    reference: &str,
    inputs: Option<&CompilerInputs>,
) -> Option<PathBuf> {
    let base_dir = tsconfig_path.parent().unwrap_or(Path::new("."));
    let reference = Path::new(reference);
    let base = if reference.is_absolute() {
        reference.to_path_buf()
    } else {
        base_dir.join(reference)
    };
    config_candidates(base)
        .into_iter()
        .map(|candidate| normalize_path_lexically(&candidate))
        .find(|candidate| {
            if let Some(inputs) = inputs {
                inputs.observe(candidate);
            }
            candidate.is_file()
        })
        .map(|candidate| normalize_path(&candidate))
}

fn config_candidates(base: PathBuf) -> Vec<PathBuf> {
    if base.extension().is_some() {
        return vec![base];
    }
    vec![
        base.clone(),
        base.with_extension("json"),
        base.join("tsconfig.json"),
    ]
}

pub(super) fn normalize_path(path: &Path) -> PathBuf {
    vize_carton::path::canonicalize_non_verbatim(&normalize_path_lexically(path))
}
