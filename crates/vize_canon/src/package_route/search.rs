use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_carton::String;

pub(super) struct PackageRequest<'a> {
    pub(super) package: &'a str,
    pub(super) subpath: Option<&'a str>,
}

impl<'a> PackageRequest<'a> {
    pub(super) fn parse(specifier: &'a str) -> Option<Self> {
        if specifier.is_empty()
            || specifier.starts_with(['.', '/', '#'])
            || Path::new(specifier).is_absolute()
        {
            return None;
        }
        let package_end = if let Some(scoped) = specifier.strip_prefix('@') {
            let scope_length = scoped.find('/')?;
            if scope_length == 0 {
                return None;
            }
            let scope_end = scope_length + 1;
            let package_end = specifier[scope_end + 1..]
                .find('/')
                .map_or(specifier.len(), |end| scope_end + 1 + end);
            if package_end == scope_end + 1 {
                return None;
            }
            package_end
        } else {
            specifier.find('/').unwrap_or(specifier.len())
        };
        Some(Self {
            package: specifier.get(..package_end)?,
            subpath: specifier
                .get(package_end + 1..)
                .filter(|subpath| !subpath.is_empty()),
        })
    }
}

pub(super) fn find_package_root(
    importer_dir: &Path,
    package: &str,
    invalidation_paths: &mut Vec<PathBuf>,
) -> Option<PathBuf> {
    for dir in importer_dir.ancestors() {
        invalidation_paths.push(dir.join("package.json"));
        if package_name_at(dir).as_deref() == Some(package) {
            return Some(dir.to_path_buf());
        }
        let candidate = dir.join("node_modules").join(package);
        let manifest = candidate.join("package.json");
        invalidation_paths.push(candidate.clone());
        invalidation_paths.push(manifest.clone());
        if manifest.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub(super) fn nearest_package_manifest(
    start: &Path,
    invalidation_paths: &mut Vec<PathBuf>,
) -> Option<(PathBuf, Value)> {
    for dir in start.ancestors() {
        invalidation_paths.push(dir.join("package.json"));
        if let Some(manifest) = read_manifest(dir) {
            return Some((dir.to_path_buf(), manifest));
        }
    }
    None
}

pub(super) fn read_manifest(root: &Path) -> Option<Value> {
    serde_json::from_str(&std::fs::read_to_string(root.join("package.json")).ok()?).ok()
}

fn package_name_at(dir: &Path) -> Option<String> {
    read_manifest(dir)?.get("name")?.as_str().map(String::from)
}
