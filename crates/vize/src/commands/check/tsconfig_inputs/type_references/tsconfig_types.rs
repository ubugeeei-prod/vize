use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_s0::FxHashSet;

use super::super::jsonc::parse_jsonc_value;
use super::super::loader::{
    read_extends_entries, resolve_extended_tsconfig, tracked_read_to_string,
};

pub(crate) fn collect_tsconfig_type_packages(
    tsconfig_path: Option<&Path>,
) -> Vec<std::string::String> {
    let Some(tsconfig_path) = tsconfig_path else {
        return Vec::new();
    };

    let mut seen = FxHashSet::default();
    load_tsconfig_type_packages(tsconfig_path, &mut seen).unwrap_or_default()
}

fn load_tsconfig_type_packages(
    tsconfig_path: &Path,
    seen: &mut FxHashSet<PathBuf>,
) -> Option<Vec<std::string::String>> {
    let resolved = vize_s0::path::canonicalize_non_verbatim(tsconfig_path);
    if !seen.insert(resolved.clone()) {
        return None;
    }

    let content = tracked_read_to_string(&resolved).ok()?;
    let value = parse_jsonc_value(&content).ok()?;

    let mut inherited = Vec::new();
    for extends in read_extends_entries(&value) {
        let Some(extends_path) = resolve_extended_tsconfig(&resolved, &extends) else {
            continue;
        };
        if let Some(parent_types) = load_tsconfig_type_packages(&extends_path, seen) {
            inherited.extend(parent_types);
        }
    }

    if let Some(types) = value
        .get("compilerOptions")
        .and_then(Value::as_object)
        .and_then(|compiler_options| compiler_options.get("types"))
        .and_then(Value::as_array)
    {
        return Some(
            types
                .iter()
                .filter_map(Value::as_str)
                .map(std::string::String::from)
                .collect(),
        );
    }

    (!inherited.is_empty()).then_some(inherited)
}
