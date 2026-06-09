//! Loading, parsing, and `extends`/`references` resolution for `tsconfig.json`.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;
use vize_carton::{FxHashSet, profile, profiler::global_profiler};

use super::glob::normalize_input_path;
use super::jsonc::parse_jsonc_value;
use super::spec::{GlobSpec, RelativePathSpec, TsconfigDeclarationOptions, TsconfigInputSpec};

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

pub(super) fn load_tsconfig_inputs(tsconfig_path: &Path) -> Option<TsconfigInputSpec> {
    let mut seen = FxHashSet::default();
    load_tsconfig_inputs_inner(tsconfig_path, &mut seen).ok()
}

fn load_tsconfig_inputs_inner(
    tsconfig_path: &Path,
    seen: &mut FxHashSet<PathBuf>,
) -> Result<TsconfigInputSpec, std::io::Error> {
    let resolved = normalize_input_path(tsconfig_path);
    if !seen.insert(resolved.clone()) {
        return Ok(TsconfigInputSpec::default());
    }

    let content = tracked_read_to_string(&resolved)?;
    let value = parse_jsonc_value(&content).unwrap_or(Value::Null);
    let dir = resolved.parent().unwrap_or(Path::new("."));

    let mut merged = TsconfigInputSpec::default();
    for extends in read_extends_entries(&value) {
        let Some(extends_path) = resolve_extended_tsconfig(&resolved, &extends) else {
            continue;
        };
        let extended = load_tsconfig_inputs_inner(&extends_path, seen)?;
        merged.apply_extended(extended);
    }

    if let Some(files) = read_string_array(&value, "files") {
        merged.has_files = true;
        merged.files = files
            .into_iter()
            .map(|value| RelativePathSpec::new(dir, &value))
            .collect();
    }

    if let Some(includes) = read_string_array(&value, "include") {
        merged.has_includes = true;
        merged.includes = includes
            .into_iter()
            .filter_map(|value| GlobSpec::new(dir, &value))
            .collect();
    }

    if let Some(excludes) = read_string_array(&value, "exclude") {
        merged.has_excludes = true;
        merged.excludes = excludes
            .into_iter()
            .filter_map(|value| GlobSpec::new(dir, &value))
            .collect();
    }

    Ok(merged)
}

pub(crate) fn load_tsconfig_declaration_options(
    tsconfig_path: &Path,
) -> TsconfigDeclarationOptions {
    let mut seen = FxHashSet::default();
    load_tsconfig_declaration_options_inner(tsconfig_path, &mut seen).unwrap_or_default()
}

fn load_tsconfig_declaration_options_inner(
    tsconfig_path: &Path,
    seen: &mut FxHashSet<PathBuf>,
) -> Result<TsconfigDeclarationOptions, std::io::Error> {
    let resolved = normalize_input_path(tsconfig_path);
    if !seen.insert(resolved.clone()) {
        return Ok(TsconfigDeclarationOptions::default());
    }

    let content = tracked_read_to_string(&resolved)?;
    let value = parse_jsonc_value(&content).unwrap_or(Value::Null);
    let dir = resolved.parent().unwrap_or(Path::new("."));

    let mut merged = TsconfigDeclarationOptions::default();
    for extends in read_extends_entries(&value) {
        let Some(extends_path) = resolve_extended_tsconfig(&resolved, &extends) else {
            continue;
        };
        let extended = load_tsconfig_declaration_options_inner(&extends_path, seen)?;
        merged.apply_extended(extended);
    }

    let Some(compiler_options) = value.get("compilerOptions").and_then(Value::as_object) else {
        return Ok(merged);
    };

    if let Some(declaration_dir) = compiler_options
        .get("declarationDir")
        .and_then(Value::as_str)
    {
        merged.declaration_dir = Some(resolve_tsconfig_path_option(dir, declaration_dir));
    }
    if let Some(out_dir) = compiler_options.get("outDir").and_then(Value::as_str) {
        merged.out_dir = Some(resolve_tsconfig_path_option(dir, out_dir));
    }
    if let Some(declaration_map) = compiler_options
        .get("declarationMap")
        .and_then(Value::as_bool)
    {
        merged.declaration_map = Some(declaration_map);
    }

    Ok(merged)
}

pub(crate) fn resolve_extended_tsconfig(tsconfig_path: &Path, extends: &str) -> Option<PathBuf> {
    let base_dir = tsconfig_path.parent().unwrap_or(Path::new("."));
    let mut candidates = Vec::new();

    if Path::new(extends).is_absolute() || extends.starts_with('.') {
        push_tsconfig_candidates(
            &mut candidates,
            if Path::new(extends).is_absolute() {
                PathBuf::from(extends)
            } else {
                base_dir.join(extends)
            },
        );
    } else {
        push_node_modules_tsconfig_candidates(&mut candidates, base_dir, extends);
    }

    candidates.into_iter().find(|candidate| candidate.is_file())
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

fn resolve_tsconfig_path_option(base_dir: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    }
}

fn push_node_modules_tsconfig_candidates(
    candidates: &mut Vec<PathBuf>,
    base_dir: &Path,
    extends: &str,
) {
    let mut current = Some(base_dir);
    while let Some(dir) = current {
        let node_modules = dir.join("node_modules");
        if let Some((package, subpath)) = split_package_specifier(extends) {
            let package_root = node_modules.join(package);
            if let Some(subpath) = subpath {
                push_tsconfig_candidates(candidates, package_root.join(subpath));
            } else {
                push_package_json_tsconfig_candidates(candidates, &package_root);
                candidates.push(package_root.join("tsconfig.json"));
            }
        } else {
            push_tsconfig_candidates(candidates, node_modules.join(extends));
        }
        current = dir.parent();
    }
}

fn split_package_specifier(extends: &str) -> Option<(&str, Option<&str>)> {
    let mut parts = extends.split('/');
    let first = parts.next()?;
    if first.is_empty() {
        return None;
    }

    if first.starts_with('@') {
        let name = parts.next()?;
        if name.is_empty() {
            return None;
        }
        let package_len = first.len() + 1 + name.len();
        let subpath = extends
            .get(package_len + 1..)
            .filter(|value| !value.is_empty());
        return Some((&extends[..package_len], subpath));
    }

    let subpath = extends
        .get(first.len() + 1..)
        .filter(|value| !value.is_empty());
    Some((first, subpath))
}

fn push_package_json_tsconfig_candidates(candidates: &mut Vec<PathBuf>, package_root: &Path) {
    let package_json_path = package_root.join("package.json");
    let Some(tsconfig) = tracked_read_to_string(&package_json_path)
        .ok()
        .and_then(|content| parse_jsonc_value(&content).ok())
        .and_then(|value| {
            value
                .get("tsconfig")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
    else {
        return;
    };

    push_tsconfig_candidates(candidates, package_root.join(tsconfig));
}

pub(super) fn tracked_read_to_string(path: &Path) -> Result<std::string::String, std::io::Error> {
    match profile!("cli.check.tsconfig.read", fs::read_to_string(path)) {
        Ok(content) => {
            global_profiler().record_fs_read_to_string(content.len());
            Ok(content)
        }
        Err(error) => {
            global_profiler().record_fs_read_to_string_failure();
            Err(error)
        }
    }
}

fn push_tsconfig_candidates(candidates: &mut Vec<PathBuf>, base: PathBuf) {
    candidates.push(base.clone());
    if base.extension().is_none() {
        candidates.push(base.with_extension("json"));
        candidates.push(base.join("tsconfig.json"));
    }
}

fn read_string_array(value: &Value, key: &str) -> Option<Vec<std::string::String>> {
    value.get(key).and_then(Value::as_array).map(|items| {
        items
            .iter()
            .filter_map(|item| item.as_str().map(std::string::String::from))
            .collect()
    })
}

pub(crate) fn read_extends_entries(value: &Value) -> Vec<std::string::String> {
    match value.get("extends") {
        Some(Value::String(extends)) => vec![extends.clone()],
        Some(Value::Array(extends)) => extends
            .iter()
            .filter_map(|item| item.as_str().map(std::string::String::from))
            .collect(),
        _ => Vec::new(),
    }
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
