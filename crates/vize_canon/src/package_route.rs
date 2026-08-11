//! Importer-scoped package-route resolution shared by CLI and editor surfaces.
//!
//! This module resolves an authored package specifier to its physical source.
//! It never creates a global `paths` alias: the importer directory stays in the
//! cache key, so callers can later extend the identity with module-resolution
//! conditions without replacing this contract (#4000, #4002).

use std::path::{Component, Path, PathBuf};

use serde_json::Value;
use vize_carton::cstr;

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
use source::resolve_sources;

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

fn lookup_uncached(
    importer_dir: &Path,
    specifier: &str,
    options: PackageSourceOptions,
    search: &mut PackageSearchCache,
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
    let package_root = canonical_path(&package_link_root);
    let manifest_path = canonical_path(&package_root.join("package.json"));
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
    for candidate in &candidates {
        invalidation_paths.push(candidate.clone());
        let mut resolved = resolve_sources(candidate, options, invalidation_paths);
        if resolved.is_empty() && candidate.extension().is_some_and(|ext| ext == "vue") {
            resolved.push(source::ResolvedPackageSource {
                source_path: canonical_path(candidate),
                native_probe_path: canonical_path(&candidate.with_extension("d.vue.ts")),
            });
        }
        for source in resolved {
            let route_source = PackageRouteSource {
                target_path: canonical_path(candidate),
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
        workspace_source: !inside_node_modules(&package_root),
        nested_routes,
    })
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
