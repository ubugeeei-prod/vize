//! Importer-scoped package-route resolution shared by CLI and editor surfaces.
//!
//! This module resolves an authored package specifier to its physical source.
//! It never creates a global `paths` alias: the importer directory stays in the
//! cache key, so callers can later extend the identity with module-resolution
//! conditions without replacing this contract (#4000, #4002).

use std::path::{Component, Path, PathBuf};

use serde_json::Value;
use vize_carton::{FxHashMap, String, cstr};

#[path = "package_route/search.rs"]
mod search;
#[path = "package_route/source.rs"]
mod source;
#[path = "package_route/stamp.rs"]
mod stamp;
use search::{
    PackageRequest, PackageSearchCache, find_package_root, nearest_package_manifest, read_manifest,
};
use source::resolve_source;
use stamp::{InputStamp, stamp_paths, stamps_are_current};

type ResolutionKey = (PathBuf, String, PackageSourceOptions);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct PackageSourceOptions {
    include_javascript: bool,
    include_jsx: bool,
}

impl PackageSourceOptions {
    pub const fn new(include_javascript: bool, include_jsx: bool) -> Self {
        Self {
            include_javascript,
            include_jsx,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageRoute {
    pub source_path: PathBuf,
    pub package_root: PathBuf,
    pub package_link_root: PathBuf,
    pub manifest_path: PathBuf,
    pub workspace_source: bool,
}

impl PackageRoute {
    pub fn invalidation_paths(&self) -> Vec<PathBuf> {
        let mut paths = vec![
            self.source_path.clone(),
            self.manifest_path.clone(),
            self.package_link_root.clone(),
            self.package_link_root.join("package.json"),
        ];
        paths.sort();
        paths.dedup();
        paths
    }
}

#[derive(Default)]
pub struct PackageRouteResolver {
    resolutions: FxHashMap<ResolutionKey, CachedPackageRouteLookup>,
    search: PackageSearchCache,
}

struct CachedPackageRouteLookup {
    lookup: PackageRouteLookup,
    stamps: Vec<InputStamp>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PackageRouteLookup {
    route: Option<PackageRoute>,
    invalidation_paths: Vec<PathBuf>,
}

impl PackageRouteLookup {
    pub fn into_parts(self) -> (Option<PackageRoute>, Vec<PathBuf>) {
        (self.route, self.invalidation_paths)
    }
}

impl PackageRouteResolver {
    pub fn resolve(
        &mut self,
        importer_dir: &Path,
        specifier: &str,
        options: PackageSourceOptions,
    ) -> Option<PackageRoute> {
        self.lookup(importer_dir, specifier, options).route
    }

    /// Resolve a route while retaining every positive or negative filesystem
    /// input consulted. Editor caches and reverse indexes use these paths to
    /// observe a package link or manifest created after an unresolved open.
    pub fn lookup(
        &mut self,
        importer_dir: &Path,
        specifier: &str,
        options: PackageSourceOptions,
    ) -> PackageRouteLookup {
        // Key on the logical importer directory: resolution walks the logical
        // ancestors, so two importer directories that share a canonical path
        // through a package symlink still see different `node_modules` chains.
        let logical_importer_dir = logical_absolute(importer_dir);
        let key = (logical_importer_dir.clone(), specifier.into(), options);
        if let Some(cached) = self.resolutions.get(&key)
            && stamps_are_current(&cached.stamps)
        {
            return cached.lookup.clone();
        }
        self.resolutions.remove(&key);
        let lookup = lookup_uncached(&logical_importer_dir, specifier, options, &mut self.search);
        let stamps = stamp_paths(&lookup.invalidation_paths);
        self.resolutions.insert(
            key,
            CachedPackageRouteLookup {
                lookup: lookup.clone(),
                stamps,
            },
        );
        lookup
    }

    pub fn clear(&mut self) {
        self.resolutions.clear();
        self.search.clear();
    }
}

fn lookup_uncached(
    importer_dir: &Path,
    specifier: &str,
    options: PackageSourceOptions,
    search: &mut PackageSearchCache,
) -> PackageRouteLookup {
    let mut invalidation_paths = Vec::new();
    let route = resolve_uncached(
        importer_dir,
        specifier,
        options,
        &mut invalidation_paths,
        search,
    );
    if let Some(route) = route.as_ref() {
        invalidation_paths.extend(route.invalidation_paths());
    }
    invalidation_paths.sort();
    invalidation_paths.dedup();
    PackageRouteLookup {
        route,
        invalidation_paths,
    }
}

fn resolve_uncached(
    importer_dir: &Path,
    specifier: &str,
    options: PackageSourceOptions,
    invalidation_paths: &mut Vec<PathBuf>,
    search: &mut PackageSearchCache,
) -> Option<PackageRoute> {
    let (package_link_root, manifest, request) = if specifier.starts_with('#') {
        let (root, manifest) = nearest_package_manifest(importer_dir, invalidation_paths, search)?;
        (root, manifest, specifier.to_owned())
    } else {
        let request = PackageRequest::parse(specifier)?;
        let root = find_package_root(importer_dir, request.package, invalidation_paths, search)?;
        let manifest = read_manifest(&root, search)?;
        let request = request
            .subpath
            .map_or_else(|| ".".to_owned(), |subpath| cstr!("./{subpath}").into());
        (root, manifest, request)
    };
    let package_root = canonical_path(&package_link_root);
    let manifest_path = canonical_path(&package_root.join("package.json"));
    let mappings = if specifier.starts_with('#') {
        manifest.get("imports")
    } else {
        manifest.get("exports")
    };
    let exports_declared = !specifier.starts_with('#') && mappings.is_some();
    let mut candidates = Vec::new();
    if let Some(mappings) = mappings {
        collect_request_targets(mappings, &request, &package_root, &mut candidates);
    }
    if candidates.is_empty() && !specifier.starts_with('#') && !exports_declared {
        collect_legacy_candidates(&manifest, &request, &package_root, &mut candidates);
    }
    let source_path = candidates
        .iter()
        .find_map(|candidate| resolve_source(candidate, options))
        .or_else(|| {
            candidates
                .iter()
                .find(|candidate| candidate.extension().is_some_and(|ext| ext == "vue"))
                .map(|candidate| canonical_path(candidate))
        })?;
    Some(PackageRoute {
        source_path,
        package_root: package_root.clone(),
        package_link_root: logical_absolute(&package_link_root),
        manifest_path,
        workspace_source: !inside_node_modules(&package_root),
    })
}

fn collect_legacy_candidates(
    manifest: &Value,
    request: &str,
    root: &Path,
    candidates: &mut Vec<PathBuf>,
) {
    if request != "." {
        candidates.push(root.join(request.trim_start_matches("./")));
        return;
    }
    for field in ["types", "typings", "module", "main"] {
        if let Some(target) = manifest.get(field).and_then(Value::as_str) {
            candidates.push(root.join(target.trim_start_matches("./")));
        }
    }
    candidates.push(root.join("index"));
}

fn collect_request_targets(value: &Value, request: &str, root: &Path, out: &mut Vec<PathBuf>) {
    if let Some(mappings) = value.as_object()
        && mappings.keys().any(|key| key.starts_with(['.', '#']))
    {
        if let Some(target) = mappings.get(request) {
            collect_targets(target, root, None, out);
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
        if let Some((_, capture, target)) = best {
            collect_targets(target, root, Some(capture), out);
        }
        return;
    }
    if request == "." {
        collect_targets(value, root, None, out);
    }
}

fn collect_targets(value: &Value, root: &Path, wildcard: Option<&str>, out: &mut Vec<PathBuf>) {
    match value {
        Value::String(target) => {
            let target =
                wildcard.map_or_else(|| target.clone(), |value| target.replace('*', value));
            let Some(relative) = target.strip_prefix("./") else {
                return;
            };
            let relative = Path::new(relative);
            if !relative.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            }) {
                out.push(root.join(relative));
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_targets(value, root, wildcard, out);
            }
        }
        Value::Object(conditions) => {
            for condition in ["types", "import", "module", "default", "require"] {
                if let Some(value) = conditions.get(condition) {
                    collect_targets(value, root, wildcard, out);
                    break;
                }
            }
        }
        _ => {}
    }
}

fn inside_node_modules(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::Normal(part) if part == "node_modules"))
}

fn canonical_path(path: &Path) -> PathBuf {
    vize_carton::path::canonicalize_non_verbatim(path)
}

/// Preserve the logical package spelling, including a workspace symlink.
/// The physical root above is canonicalized separately for source identity.
fn logical_absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    }
}

#[cfg(test)]
#[path = "package_route_tests.rs"]
mod tests;
