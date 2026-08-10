use super::{
    FxHashSet, Path, PathBuf, Value, normalize_input_path, parse_jsonc_value,
    push_tsconfig_candidates, tracked_read_to_string,
};

pub(super) fn collect_tsconfig_project_paths(tsconfig_path: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen = FxHashSet::default();
    collect_tsconfig_project_paths_inner(tsconfig_path, &mut seen, &mut paths);
    paths
}

fn collect_tsconfig_project_paths_inner(
    tsconfig_path: &Path,
    seen: &mut FxHashSet<PathBuf>,
    paths: &mut Vec<PathBuf>,
) {
    let resolved = normalize_input_path(tsconfig_path);
    if !seen.insert(resolved.clone()) {
        return;
    }
    paths.push(resolved.clone());

    let Ok(content) = tracked_read_to_string(&resolved) else {
        return;
    };
    let value = parse_jsonc_value(&content).unwrap_or(Value::Null);
    for reference in read_reference_entries(&value) {
        let Some(reference_path) = resolve_referenced_tsconfig(&resolved, &reference) else {
            continue;
        };
        collect_tsconfig_project_paths_inner(&reference_path, seen, paths);
    }
}

fn resolve_referenced_tsconfig(tsconfig_path: &Path, reference: &str) -> Option<PathBuf> {
    let base_dir = tsconfig_path.parent().unwrap_or(Path::new("."));
    let reference_path = Path::new(reference);
    let base = if reference_path.is_absolute() {
        reference_path.to_path_buf()
    } else {
        base_dir.join(reference_path)
    };
    let mut candidates = Vec::new();
    push_tsconfig_candidates(&mut candidates, base);
    candidates.into_iter().find(|candidate| candidate.is_file())
}

fn read_reference_entries(value: &Value) -> Vec<std::string::String> {
    value
        .get("references")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("path").and_then(Value::as_str))
        .map(std::string::String::from)
        .collect()
}
