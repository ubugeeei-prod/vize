//! Resolve bare imports only far enough to identify authored workspace sources.
//! TypeScript remains the authority for the actual program resolution.

use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_carton::{FxHashMap, String, cstr};

use super::{ImportFileOptions, is_node_modules_path, resolve_import_base};
use crate::commands::check::path_cache::CanonicalPathCache;

type ResolutionKey = (PathBuf, String, ImportFileOptions);

#[derive(Default)]
pub(super) struct PackageImportResolver {
    resolutions: FxHashMap<ResolutionKey, Option<PathBuf>>,
}

impl PackageImportResolver {
    pub(super) fn resolve(
        &mut self,
        importer_dir: &Path,
        specifier: &str,
        canonical_paths: &mut CanonicalPathCache,
        options: ImportFileOptions,
    ) -> Option<PathBuf> {
        let key = (
            canonical_paths.canonicalize(importer_dir),
            specifier.into(),
            options,
        );
        if let Some(cached) = self.resolutions.get(&key) {
            return cached.clone();
        }
        let resolved = resolve_uncached(importer_dir, specifier, canonical_paths, options);
        self.resolutions.insert(key, resolved.clone());
        resolved
    }
}

fn resolve_uncached(
    importer_dir: &Path,
    specifier: &str,
    canonical_paths: &mut CanonicalPathCache,
    options: ImportFileOptions,
) -> Option<PathBuf> {
    if specifier.starts_with('#') {
        return resolve_internal_import(importer_dir, specifier, canonical_paths, options);
    }
    let request = PackageRequest::parse(specifier)?;
    let package_root = find_package_root(importer_dir, request.package)?;
    let package_root = canonical_paths.canonicalize(&package_root);
    // Installed dependencies remain TypeScript's responsibility. Only a
    // symlinked/self-referenced package whose real source lives outside
    // `node_modules` can contribute authored diagnostics here.
    if is_node_modules_path(&package_root) {
        return None;
    }
    let package_json = std::fs::read_to_string(package_root.join("package.json"))
        .ok()
        .and_then(|content| serde_json::from_str::<Value>(&content).ok());
    let exports_declared = package_json
        .as_ref()
        .is_some_and(|package| package.get("exports").is_some());
    let mut candidates = Vec::new();
    if let Some(package_json) = package_json.as_ref() {
        collect_manifest_candidates(
            package_json,
            &package_root,
            request.subpath,
            &mut candidates,
        );
    }
    if candidates.is_empty() {
        if exports_declared {
            return None;
        }
        candidates.push(match request.subpath {
            Some(subpath) => package_root.join(subpath),
            None => package_root.join("index"),
        });
    }
    candidates
        .into_iter()
        .find_map(|candidate| resolve_import_base(&candidate, canonical_paths, options))
}

fn resolve_internal_import(
    importer_dir: &Path,
    specifier: &str,
    canonical_paths: &mut CanonicalPathCache,
    options: ImportFileOptions,
) -> Option<PathBuf> {
    let (package_root, package_json) = nearest_package_manifest(importer_dir)?;
    let imports = package_json.get("imports")?;
    let mut candidates = Vec::new();
    collect_map_request_targets(imports, specifier, &package_root, &mut candidates);
    candidates
        .into_iter()
        .find_map(|candidate| resolve_import_base(&candidate, canonical_paths, options))
}

fn nearest_package_manifest(start: &Path) -> Option<(PathBuf, Value)> {
    for dir in start.ancestors() {
        let Ok(manifest) = std::fs::read_to_string(dir.join("package.json")) else {
            continue;
        };
        let Ok(package) = serde_json::from_str::<Value>(&manifest) else {
            return None;
        };
        return Some((dir.to_path_buf(), package));
    }
    None
}

struct PackageRequest<'a> {
    package: &'a str,
    subpath: Option<&'a str>,
}

impl<'a> PackageRequest<'a> {
    fn parse(specifier: &'a str) -> Option<Self> {
        if specifier.is_empty()
            || specifier.starts_with(['.', '/', '#'])
            || Path::new(specifier).is_absolute()
        {
            return None;
        }
        let package_end = if let Some(scoped) = specifier.strip_prefix('@') {
            let slash = scoped.find('/')? + 1;
            specifier[slash + 1..]
                .find('/')
                .map_or(specifier.len(), |end| slash + 1 + end)
        } else {
            specifier.find('/').unwrap_or(specifier.len())
        };
        let package = specifier.get(..package_end)?;
        let subpath = specifier
            .get(package_end + 1..)
            .filter(|subpath| !subpath.is_empty());
        Some(Self { package, subpath })
    }
}

fn find_package_root(importer_dir: &Path, package: &str) -> Option<PathBuf> {
    for dir in importer_dir.ancestors() {
        if package_name_at(dir).as_deref() == Some(package) {
            return Some(dir.to_path_buf());
        }
        let candidate = dir.join("node_modules").join(package);
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
}

fn package_name_at(dir: &Path) -> Option<String> {
    let manifest = std::fs::read_to_string(dir.join("package.json")).ok()?;
    let package = serde_json::from_str::<Value>(&manifest).ok()?;
    package.get("name")?.as_str().map(String::from)
}

fn collect_manifest_candidates(
    package_json: &Value,
    package_root: &Path,
    subpath: Option<&str>,
    candidates: &mut Vec<PathBuf>,
) {
    if let Some(exports) = package_json.get("exports") {
        if let Some(subpath) = subpath {
            collect_subpath_export_targets(exports, subpath, package_root, candidates);
        } else if let Some(export) = exports.get(".").or(Some(exports)) {
            collect_export_targets(export, package_root, None, candidates);
        }
        return;
    }
    if let Some(subpath) = subpath {
        candidates.push(package_root.join(subpath));
        return;
    }
    for field in ["types", "typings", "module", "main"] {
        if let Some(target) = package_json.get(field).and_then(Value::as_str) {
            candidates.push(package_root.join(target));
        }
    }
    candidates.push(package_root.join("index"));
}

fn collect_subpath_export_targets(
    exports: &Value,
    subpath: &str,
    root: &Path,
    candidates: &mut Vec<PathBuf>,
) {
    collect_map_request_targets(exports, &cstr!("./{subpath}"), root, candidates);
}

fn collect_map_request_targets(
    mappings: &Value,
    request: &str,
    root: &Path,
    candidates: &mut Vec<PathBuf>,
) {
    let Some(mappings) = mappings.as_object() else {
        return;
    };
    if let Some(value) = mappings.get(request) {
        collect_export_targets(value, root, None, candidates);
        return;
    }
    let best = mappings
        .iter()
        .filter_map(|(pattern, value)| {
            let (prefix, suffix) = pattern.split_once('*')?;
            let capture = request.strip_prefix(prefix)?.strip_suffix(suffix)?;
            Some(((prefix.len(), suffix.len()), capture, value))
        })
        .max_by_key(|(specificity, _, _)| *specificity);
    if let Some((_, capture, value)) = best {
        collect_export_targets(value, root, Some(capture), candidates);
    }
}

fn collect_export_targets(
    value: &Value,
    root: &Path,
    wildcard: Option<&str>,
    candidates: &mut Vec<PathBuf>,
) {
    match value {
        Value::String(target) => {
            let target = match wildcard {
                Some(wildcard) => target.replace('*', wildcard),
                None => target.clone(),
            };
            let Some(relative) = target.strip_prefix("./") else {
                return;
            };
            let relative = Path::new(relative);
            if relative.components().any(|component| {
                matches!(
                    component,
                    std::path::Component::ParentDir
                        | std::path::Component::RootDir
                        | std::path::Component::Prefix(_)
                )
            }) {
                return;
            }
            candidates.push(root.join(relative));
        }
        Value::Array(values) => {
            for value in values {
                collect_export_targets(value, root, wildcard, candidates);
            }
        }
        Value::Object(_) => collect_condition_targets(value, root, wildcard, candidates),
        _ => {}
    }
}

fn collect_condition_targets(
    value: &Value,
    root: &Path,
    wildcard: Option<&str>,
    candidates: &mut Vec<PathBuf>,
) {
    let Some(conditions) = value.as_object() else {
        return;
    };
    for condition in ["types", "import", "module", "default", "require"] {
        if let Some(value) = conditions.get(condition) {
            collect_export_targets(value, root, wildcard, candidates);
        }
    }
}

#[cfg(test)]
#[path = "imports_packages_tests.rs"]
mod tests;
