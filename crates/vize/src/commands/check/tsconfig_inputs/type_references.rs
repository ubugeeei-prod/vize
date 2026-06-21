//! `/// <reference types="..." />` package declaration resolution.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;
use vize_carton::FxHashSet;

use super::jsonc::parse_jsonc_value;

pub(crate) fn reference_type_packages(content: &str) -> Vec<std::string::String> {
    content
        .lines()
        .filter_map(reference_types_attribute)
        .map(std::string::String::from)
        .collect()
}

pub(crate) fn resolve_type_package_declaration_files(
    project_root: &Path,
    reference: &str,
) -> Vec<PathBuf> {
    resolve_type_reference_declaration_files(project_root, reference)
}

pub(crate) fn resolve_type_reference_declaration_files(
    search_start: &Path,
    reference: &str,
) -> Vec<PathBuf> {
    let Some(reference) = TypeReference::parse(reference) else {
        return Vec::new();
    };

    for package_root in type_package_roots(search_start, reference.package.as_str()) {
        if let Some(files) = package_declaration_files(&package_root, reference.subpath) {
            return files;
        }
    }

    Vec::new()
}

fn reference_types_attribute(line: &str) -> Option<&str> {
    let line = line.trim_start();
    if !line.starts_with("///") || !line.contains("<reference") {
        return None;
    }
    attribute_value(line, "types")
}

fn attribute_value<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("{name}=");
    let start = line.find(&needle)? + needle.len();
    let quote = line[start..].chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value_start = start + quote.len_utf8();
    let value_end = line[value_start..].find(quote)? + value_start;
    line.get(value_start..value_end)
}

struct TypeReference<'a> {
    package: std::string::String,
    subpath: Option<&'a str>,
}

impl<'a> TypeReference<'a> {
    fn parse(reference: &'a str) -> Option<Self> {
        let reference = reference.trim();
        if reference.is_empty() || reference.starts_with('.') || reference.starts_with('/') {
            return None;
        }

        let package_end = if let Some(scoped) = reference.strip_prefix('@') {
            let scope_end = scoped.find('/')?;
            let name_start = scope_end + 2;
            let name = reference.get(name_start..)?;
            name.find('/')
                .map_or(reference.len(), |name_end| name_start + name_end)
        } else {
            reference.find('/').unwrap_or(reference.len())
        };

        let package = reference.get(..package_end)?;
        if package.ends_with('/') || package.is_empty() {
            return None;
        }
        let subpath = reference
            .get(package_end + 1..)
            .filter(|subpath| !subpath.is_empty());

        Some(Self {
            package: std::string::String::from(package),
            subpath,
        })
    }
}

fn type_package_roots(search_start: &Path, package: &str) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let mut seen = FxHashSet::default();
    let mut current = if search_start.is_file() {
        search_start.parent()
    } else {
        Some(search_start)
    };

    while let Some(dir) = current {
        let node_modules = dir.join("node_modules");
        push_existing_package_root(&mut roots, &mut seen, node_modules.join(package));
        if let Some(types_package) = fallback_types_package_name(package) {
            push_existing_package_root(&mut roots, &mut seen, node_modules.join(types_package));
        }
        current = dir.parent();
    }

    roots
}

fn push_existing_package_root(
    roots: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
    package_root: PathBuf,
) {
    if package_root.is_dir() {
        let package_root = vize_carton::path::canonicalize_non_verbatim(&package_root);
        if seen.insert(package_root.clone()) {
            roots.push(package_root);
        }
    }
}

fn fallback_types_package_name(package: &str) -> Option<std::string::String> {
    if package.starts_with("@types/") {
        return None;
    }
    if let Some(scoped) = package.strip_prefix('@') {
        let mut parts = scoped.split('/');
        let scope = parts.next()?;
        let name = parts.next()?;
        return Some(format!("@types/{scope}__{name}"));
    }
    Some(format!("@types/{package}"))
}

fn package_declaration_files(package_root: &Path, subpath: Option<&str>) -> Option<Vec<PathBuf>> {
    for entry in package_declaration_entry_candidates(package_root, subpath) {
        if is_declaration_path(&entry) && entry.is_file() {
            return Some(collect_package_declaration_graph(&entry, package_root));
        }
    }

    None
}

fn package_declaration_entry_candidates(
    package_root: &Path,
    subpath: Option<&str>,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let package_json_path = package_root.join("package.json");
    if let Ok(content) = fs::read_to_string(&package_json_path)
        && let Ok(package_json) = parse_jsonc_value(&content)
    {
        if let Some(subpath) = subpath {
            collect_subpath_export_type_entries(
                &package_json,
                package_root,
                subpath,
                &mut candidates,
            );
        } else {
            collect_root_package_type_entries(&package_json, package_root, &mut candidates);
        }
    }

    match subpath {
        Some(subpath) => {
            push_declaration_entry_candidate(&mut candidates, package_root.join(subpath))
        }
        None => push_declaration_entry_candidate(&mut candidates, package_root.join("index.d.ts")),
    }
    candidates
}

fn collect_root_package_type_entries(
    package_json: &Value,
    package_root: &Path,
    candidates: &mut Vec<PathBuf>,
) {
    for field in ["types", "typings"] {
        if let Some(types) = package_json.get(field).and_then(Value::as_str) {
            push_declaration_entry_candidate(candidates, package_root.join(types));
        }
    }

    if let Some(exports) = package_json.get("exports") {
        let root_export = exports.get(".").unwrap_or(exports);
        collect_export_type_entries(root_export, package_root, candidates);
    }
}

fn collect_subpath_export_type_entries(
    package_json: &Value,
    package_root: &Path,
    subpath: &str,
    candidates: &mut Vec<PathBuf>,
) {
    let Some(exports) = package_json.get("exports") else {
        return;
    };
    let key = format!("./{subpath}");
    if let Some(export) = exports.get(&key) {
        collect_export_type_entries(export, package_root, candidates);
    }
}

fn collect_export_type_entries(value: &Value, package_root: &Path, candidates: &mut Vec<PathBuf>) {
    match value {
        Value::String(path) => {
            push_declaration_entry_candidate(candidates, package_root.join(path))
        }
        Value::Array(values) => {
            for value in values {
                collect_export_type_entries(value, package_root, candidates);
            }
        }
        Value::Object(map) => {
            if let Some(types) = map.get("types").and_then(Value::as_str) {
                push_declaration_entry_candidate(candidates, package_root.join(types));
            }
            for value in map.values() {
                collect_export_type_entries(value, package_root, candidates);
            }
        }
        _ => {}
    }
}

fn push_declaration_entry_candidate(candidates: &mut Vec<PathBuf>, path: PathBuf) {
    push_unique_candidate(candidates, path.clone());
    if path.extension().is_none() {
        push_unique_candidate(candidates, path.with_extension("d.ts"));
        push_unique_candidate(candidates, path.join("index.d.ts"));
    }
}

fn push_unique_candidate(candidates: &mut Vec<PathBuf>, path: PathBuf) {
    if !candidates.contains(&path) {
        candidates.push(path);
    }
}

fn collect_package_declaration_graph(entry: &Path, package_root: &Path) -> Vec<PathBuf> {
    let package_root = vize_carton::path::canonicalize_non_verbatim(package_root);
    let mut files = Vec::new();
    let mut seen = FxHashSet::default();
    let mut queue = vec![entry.to_path_buf()];

    while let Some(path) = queue.pop() {
        let path = vize_carton::path::canonicalize_non_verbatim(&path);
        if !seen.insert(path.clone()) {
            continue;
        }
        files.push(path.clone());

        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let Some(base_dir) = path.parent() else {
            continue;
        };
        for reference in content.lines().filter_map(reference_path_attribute) {
            let referenced =
                vize_carton::path::canonicalize_non_verbatim(&base_dir.join(reference));
            if referenced.starts_with(&package_root)
                && is_declaration_path(&referenced)
                && referenced.is_file()
            {
                queue.push(referenced);
            }
        }
    }

    files
}

fn reference_path_attribute(line: &str) -> Option<&str> {
    let line = line.trim_start();
    if !line.starts_with("///") || !line.contains("<reference") {
        return None;
    }
    attribute_value(line, "path")
}

fn is_declaration_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
}

#[cfg(test)]
mod tests;
