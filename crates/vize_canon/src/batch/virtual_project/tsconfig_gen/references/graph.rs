use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_carton::FxHashSet;

use super::{normalize_path_lexically, parse_jsonc_value};

pub(super) fn collect_project_paths(tsconfig_path: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen = FxHashSet::default();
    collect_project_paths_inner(tsconfig_path, &mut seen, &mut paths);
    paths
}

fn collect_project_paths_inner(
    tsconfig_path: &Path,
    seen: &mut FxHashSet<PathBuf>,
    paths: &mut Vec<PathBuf>,
) {
    let resolved = normalize_path(tsconfig_path);
    if !seen.insert(resolved.clone()) {
        return;
    }
    paths.push(resolved.clone());

    let Ok(content) = std::fs::read_to_string(&resolved) else {
        return;
    };
    let Ok(value) = parse_jsonc_value(&content) else {
        return;
    };
    for reference in value
        .get("references")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.get("path").and_then(Value::as_str))
    {
        let Some(referenced) = resolve_reference(&resolved, reference) else {
            continue;
        };
        collect_project_paths_inner(&referenced, seen, paths);
    }
}

fn resolve_reference(tsconfig_path: &Path, reference: &str) -> Option<PathBuf> {
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
        .find(|candidate| candidate.is_file())
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
