use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_carton::FxHashMap;

use super::stamp::{InputStamp, manifest_path};

#[derive(Default)]
pub(super) struct PackageSearchCache {
    manifests: FxHashMap<PathBuf, CachedManifest>,
}

struct CachedManifest {
    stamp: InputStamp,
    value: Option<Value>,
}

impl PackageSearchCache {
    pub(super) fn clear(&mut self) {
        self.manifests.clear();
    }

    fn read_manifest(&mut self, root: &Path) -> Option<Value> {
        let path = manifest_path(root);
        if let Some(cached) = self.manifests.get(&path)
            && cached.stamp.is_current()
        {
            return cached.value.clone();
        }
        let stamp = InputStamp::capture(&path);
        let value = std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok());
        self.manifests.insert(
            path,
            CachedManifest {
                stamp,
                value: value.clone(),
            },
        );
        value
    }
}

pub(super) struct PackageRequest<'a> {
    pub(super) package: &'a str,
    pub(super) subpath: Option<&'a str>,
}

impl<'a> PackageRequest<'a> {
    pub(super) fn parse(specifier: &'a str) -> Option<Self> {
        if specifier.is_empty()
            || specifier.starts_with(['.', '/', '#'])
            || is_absolute_specifier(specifier)
        {
            return None;
        }
        let package_end = if let Some(scoped) = specifier.strip_prefix('@') {
            let scope_end = scoped.find('/')?;
            if scope_end == 0 {
                return None;
            }
            let scope_end = scope_end + 1;
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

fn is_absolute_specifier(specifier: &str) -> bool {
    if Path::new(specifier).is_absolute() || specifier.starts_with("\\\\") {
        return true;
    }
    let bytes = specifier.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\')
}

pub(super) fn find_package_root(
    importer_dir: &Path,
    package: &str,
    invalidation_paths: &mut Vec<PathBuf>,
    cache: &mut PackageSearchCache,
) -> Option<PathBuf> {
    // Node resolves a package self-reference from the nearest package scope
    // before walking `node_modules`. Do that as a separate cached pass so a
    // same-named nested install cannot shadow the authored package itself.
    for dir in importer_dir.ancestors() {
        invalidation_paths.push(dir.join("package.json"));
        if let Some(manifest) = read_manifest(dir, cache) {
            if manifest.get("name").and_then(Value::as_str) == Some(package) {
                return Some(dir.to_path_buf());
            }
            break;
        }
    }
    for dir in importer_dir.ancestors() {
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
    cache: &mut PackageSearchCache,
) -> Option<(PathBuf, Value)> {
    for dir in start.ancestors() {
        invalidation_paths.push(dir.join("package.json"));
        if let Some(manifest) = read_manifest(dir, cache) {
            return Some((dir.to_path_buf(), manifest));
        }
    }
    None
}

pub(super) fn read_manifest(root: &Path, cache: &mut PackageSearchCache) -> Option<Value> {
    cache.read_manifest(root)
}
