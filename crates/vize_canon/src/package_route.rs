//! Importer-scoped package-route resolution shared by CLI and editor surfaces.
//!
//! This module resolves an authored package specifier to its physical source.
//! It never creates a global `paths` alias: the importer directory stays in the
//! cache key, so callers can later extend the identity with module-resolution
//! conditions without replacing this contract (#4000, #4002).

use std::path::{Component, Path, PathBuf};

use serde_json::Value;
use vize_carton::{FxHashMap, cstr};

#[path = "package_route/cache.rs"]
mod cache;
#[path = "package_route/candidates.rs"]
mod candidates;
#[path = "package_route/graph_inputs.rs"]
mod graph_inputs;
#[path = "package_route/model.rs"]
mod model;
#[path = "package_route/search.rs"]
mod search;
#[path = "package_route/shadow_mapping.rs"]
mod shadow_mapping;
#[path = "package_route/source.rs"]
mod source;
#[path = "package_route/stamp.rs"]
pub(crate) mod stamp;
#[cfg(test)]
use candidates::collect_targets;
use candidates::{
    collect_external_import_targets, collect_legacy_candidates, collect_request_targets,
    collect_types_version_candidates,
};
use search::{
    PackageRequest, PackageSearchCache, find_package_root, nearest_package_manifest, read_manifest,
};
use source::{resolve_sources, resolve_workspace_source_fallbacks};

pub use cache::{PackageRouteLookup, PackageRouteResolver};
#[cfg(feature = "native")]
pub(crate) use model::PackageRouteKey;
pub use model::{
    PackageResolutionContext, PackageResolutionMode, PackageRoute, PackageRouteBinding,
    PackageRouteMetrics, PackageRouteSource,
};

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

#[derive(Default)]
pub(super) struct PackagePathCache {
    paths: FxHashMap<PathBuf, PathBuf>,
}

impl PackagePathCache {
    fn clear(&mut self) {
        self.paths.clear();
    }

    fn canonicalize(&mut self, path: &Path) -> PathBuf {
        if let Some(cached) = self.paths.get(path) {
            return cached.clone();
        }
        let canonical = vize_carton::path::canonicalize_non_verbatim(path);
        self.paths.insert(path.to_path_buf(), canonical.clone());
        canonical
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.paths.len()
    }
}

fn lookup_uncached(
    importer_dir: &Path,
    specifier: &str,
    options: PackageSourceOptions,
    search: &mut PackageSearchCache,
    canonical_paths: &mut PackagePathCache,
) -> PackageRouteLookup {
    let mut invalidation_paths = Vec::new();
    let mut watchable_negative = false;
    let route = resolve_uncached(
        importer_dir,
        specifier,
        options,
        &mut invalidation_paths,
        search,
        &mut watchable_negative,
        canonical_paths,
    );
    if let Some(route) = route.as_ref() {
        invalidation_paths.extend(route.invalidation_paths());
        graph_inputs::collect(&route.package_link_root, &mut invalidation_paths);
    }
    graph_inputs::collect(importer_dir, &mut invalidation_paths);
    invalidation_paths.sort();
    invalidation_paths.dedup();
    cache::PackageRouteLookup {
        route,
        invalidation_paths,
        watchable_negative,
    }
}

fn resolve_uncached(
    importer_dir: &Path,
    specifier: &str,
    options: PackageSourceOptions,
    invalidation_paths: &mut Vec<PathBuf>,
    search: &mut PackageSearchCache,
    watchable_negative: &mut bool,
    canonical_paths: &mut PackagePathCache,
) -> Option<PackageRoute> {
    let (package_link_root, manifest, request, package_name) = if specifier.starts_with('#') {
        let (root, manifest) = nearest_package_manifest(importer_dir, invalidation_paths, search)?;
        let package_name = manifest.get("name").and_then(Value::as_str).map(Into::into);
        (root, manifest, specifier.to_owned(), package_name)
    } else {
        if specifier.starts_with("node:") {
            return None;
        }
        let request = PackageRequest::parse(specifier)?;
        let package_name = request.package.into();
        let Some(root) =
            find_package_root(importer_dir, request.package, invalidation_paths, search)
        else {
            *watchable_negative = true;
            return None;
        };
        let Some(manifest) = read_manifest(&root, search) else {
            *watchable_negative = true;
            return None;
        };
        let request = request
            .subpath
            .map_or_else(|| ".".to_owned(), |subpath| cstr!("./{subpath}").into());
        (root, manifest, request, Some(package_name))
    };
    let package_root = canonical_path(&package_link_root, canonical_paths);
    let manifest_path = canonical_path(&package_root.join("package.json"), canonical_paths);
    let mappings = if specifier.starts_with('#') {
        manifest.get("imports")
    } else {
        manifest.get("exports")
    };
    let mut candidates = Vec::new();
    if let Some(mappings) = mappings {
        collect_request_targets(mappings, &request, &package_root, &mut candidates);
    }
    let mut nested_routes = Vec::new();
    if specifier.starts_with('#')
        && let Some(mappings) = mappings
    {
        let mut external_targets = Vec::new();
        collect_external_import_targets(mappings, &request, &mut external_targets);
        external_targets.sort();
        external_targets.dedup();
        for target in external_targets {
            if let Some(route) = resolve_uncached(
                &package_root,
                &target,
                options,
                invalidation_paths,
                search,
                watchable_negative,
                canonical_paths,
            ) {
                nested_routes.push(route);
            }
        }
    }
    if !specifier.starts_with('#') {
        // Keep legacy/type-version topology alongside exports. Native
        // TypeScript decides which family is active for Node10/Classic versus
        // Node16/NodeNext/Bundler; extra shadow candidates do not select one.
        collect_legacy_candidates(&manifest, &request, &package_root, &mut candidates);
        collect_types_version_candidates(&manifest, &request, &package_root, &mut candidates);
    }
    let mut source_targets = Vec::new();
    let workspace_source = !inside_node_modules(&package_root);
    for candidate in &candidates {
        invalidation_paths.push(candidate.clone());
        let mut resolved = resolve_sources(candidate, options, invalidation_paths, canonical_paths);
        if resolved.is_empty() && workspace_source {
            resolved = resolve_workspace_source_fallbacks(
                candidate,
                &package_root,
                options,
                invalidation_paths,
                canonical_paths,
            );
        }
        if resolved.is_empty() && candidate.extension().is_some_and(|ext| ext == "vue") {
            resolved.push(source::ResolvedPackageSource {
                source_path: canonical_path(candidate, canonical_paths),
                native_probe_path: canonical_path(
                    &candidate.with_extension("d.vue.ts"),
                    canonical_paths,
                ),
            });
        }
        for source in resolved {
            let route_source = PackageRouteSource {
                target_path: canonical_path(candidate, canonical_paths),
                source_path: source.source_path,
                native_probe_path: source.native_probe_path,
            };
            if !source_targets.contains(&route_source) {
                source_targets.push(route_source);
            }
        }
    }
    let mut source_paths = source_targets
        .iter()
        .map(|source| source.source_path.clone())
        .collect::<Vec<_>>();
    source_paths.sort();
    source_paths.dedup();
    if source_paths.is_empty() && nested_routes.is_empty() {
        *watchable_negative |= specifier.starts_with('#')
            || !inside_node_modules(&package_root)
            || candidates.iter().any(|candidate| {
                candidate
                    .extension()
                    .is_some_and(|extension| extension == "vue")
            });
        return None;
    }
    Some(PackageRoute {
        source_paths,
        dependency_paths: Vec::new(),
        source_targets,
        package_root: package_root.clone(),
        package_link_root: logical_absolute(&package_link_root),
        manifest_path,
        package_name,
        workspace_source,
        nested_routes,
    })
}

fn inside_node_modules(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::Normal(part) if part == "node_modules"))
}

fn canonical_path(path: &Path, cache: &mut PackagePathCache) -> PathBuf {
    cache.canonicalize(path)
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
