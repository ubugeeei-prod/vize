//! Importer-scoped package-route resolution shared by CLI and editor surfaces.
//!
//! This module resolves an authored package specifier to its physical source.
//! It never creates a global `paths` alias: the importer directory stays in the
//! cache key, so callers can later extend the identity with module-resolution
//! conditions without replacing this contract (#4000, #4002).

use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};

use serde_json::Value;
use vize_carton::{FxHashMap, FxHashSet, cstr};

mod cache;
mod candidates;
mod graph_inputs;
mod model;
mod search;
mod shadow_mapping;
mod source;
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

pub use cache::{PackageRouteLookup, PackageRouteResolver, PackageRouteValidationScope};
#[cfg(feature = "native")]
pub(crate) use model::PackageRouteKey;
pub use model::{
    PackageResolutionContext, PackageResolutionMode, PackageRoute, PackageRouteBinding,
    PackageRouteMetrics, PackageRouteSource,
};

/// Read an installed package's manifest using the same importer-scoped lookup
/// as package routes, including workspace self references and pnpm links.
#[cfg(feature = "native")]
pub(crate) fn installed_package_manifest(
    importer_dir: &Path,
    package: &str,
) -> Option<(PathBuf, Value)> {
    let mut cache = PackageSearchCache::default();
    let root = find_package_root(importer_dir, package, &mut Vec::new(), &mut cache)?;
    let manifest = read_manifest(&root, &mut cache)?;
    Some((root.join("package.json"), manifest))
}

/// Reuse package lookup semantics with an embedding capture's input reader.
pub(crate) fn installed_package_manifest_with_reader(
    importer_dir: &Path,
    package: &str,
    mut read: impl FnMut(&Path) -> Option<Value>,
    observe: impl Fn(&Path),
) -> Option<(PathBuf, Value)> {
    let root = search::find_package_root_with_reader(
        importer_dir,
        package,
        &mut Vec::new(),
        &mut |directory| read(&directory.join("package.json")),
        &observe,
    )?;
    let path = root.join("package.json");
    Some((path.clone(), read(&path)?))
}

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

/// One directory listing: the exact names, and the ASCII-lowercased names a
/// case-insensitive filesystem would still match.
struct DirectoryListing {
    names: FxHashSet<OsString>,
    lowercase: FxHashSet<Vec<u8>>,
    /// Some entry is not ASCII, so case folding can reach an ASCII probe from
    /// outside ASCII (`k.ts` matches a Kelvin-sign `K.ts` on APFS).
    non_ascii: bool,
}

impl DirectoryListing {
    /// `None` when the directory cannot be read in full: an entry error would
    /// leave the listing partial, and a partial listing cannot prove absence.
    fn read(dir: &Path) -> Option<Self> {
        let mut names = FxHashSet::default();
        let mut lowercase = FxHashSet::default();
        let mut non_ascii = false;
        for entry in std::fs::read_dir(dir).ok()? {
            let name = entry.ok()?.file_name();
            let bytes = name.as_encoded_bytes();
            non_ascii |= !bytes.is_ascii();
            lowercase.insert(bytes.to_ascii_lowercase());
            names.insert(name);
        }
        Some(Self {
            names,
            lowercase,
            non_ascii,
        })
    }

    /// Whether no entry can match `name` on any filesystem: neither its exact
    /// spelling nor, for an ASCII name among ASCII entries, one that differs
    /// only in case. Everything else keeps its stat: a non-ASCII name, since a
    /// normalization-insensitive filesystem matches more than case; an ASCII
    /// name beside non-ASCII entries, since case folding can match it to one
    /// of them; and on Windows a spelling that can open a listed entry under
    /// another name.
    fn lacks(&self, name: &OsStr) -> bool {
        if self.names.contains(name) {
            return false;
        }
        let bytes = name.as_encoded_bytes();
        if !bytes.is_ascii() || self.non_ascii || (cfg!(windows) && may_alias_on_windows(bytes)) {
            return false;
        }
        !self.lowercase.contains(&bytes.to_ascii_lowercase())
    }
}

/// Whether Windows could open a listed entry through this spelling: an 8.3
/// alias (`LONGFI~1.TS` for `Longfilename1.ts`), a name with trailing dots or
/// spaces, which Win32 strips before the lookup, or a stream suffix after `:`.
fn may_alias_on_windows(name: &[u8]) -> bool {
    name.contains(&b'~')
        || name.contains(&b':')
        || name.last().is_some_and(|last| matches!(last, b'.' | b' '))
}

#[derive(Default)]
pub(super) struct PackagePathCache {
    paths: FxHashMap<PathBuf, PathBuf>,
    /// Listed directories, so a probe for an absent entry costs no syscall.
    /// `None` is a directory that could not be read; its probes stat.
    listings: FxHashMap<PathBuf, Option<DirectoryListing>>,
    #[cfg(test)]
    canonicalizations: usize,
}

impl PackagePathCache {
    fn clear(&mut self) {
        self.paths.clear();
        self.listings.clear();
    }

    /// `path.is_file()`, answered from the parent listing when the entry is
    /// absent. A listed entry is still checked on disk.
    pub(super) fn is_file(&mut self, path: &Path) -> bool {
        !self.absent(path) && path.is_file()
    }

    /// Whether a listed ancestor proves that `path` cannot exist.
    fn absent(&mut self, path: &Path) -> bool {
        let mut child = path;
        for ancestor in path.ancestors().skip(1) {
            let Some(name) = child.file_name() else {
                return false;
            };
            match self.listing(ancestor) {
                Some(listing) => return listing.lacks(name),
                None => child = ancestor,
            }
        }
        false
    }

    fn listing(&mut self, dir: &Path) -> Option<&DirectoryListing> {
        if !self.listings.contains_key(dir) {
            let listing = DirectoryListing::read(dir);
            self.listings.insert(dir.to_path_buf(), listing);
        }
        self.listings.get(dir).and_then(Option::as_ref)
    }

    fn canonicalize(&mut self, path: &Path) -> PathBuf {
        if let Some(cached) = self.paths.get(path) {
            return cached.clone();
        }
        // An entry its parent does not list fails realpath; skip the syscall.
        if self.absent(path) {
            let normalized = vize_carton::path::normalize_windows_verbatim_path(path.to_path_buf());
            self.paths.insert(path.to_path_buf(), normalized.clone());
            return normalized;
        }
        #[cfg(test)]
        {
            self.canonicalizations += 1;
        }
        let canonical = match path.canonicalize() {
            Ok(physical) => {
                let physical = vize_carton::path::normalize_windows_verbatim_path(physical);
                // A successful realpath also establishes its physical spelling.
                // Package targets use that spelling after resolving a pnpm link;
                // querying it again must not repeat the same filesystem walk.
                if physical != path {
                    self.paths
                        .entry(physical.clone())
                        .or_insert_with(|| physical.clone());
                }
                physical
            }
            Err(_) => vize_carton::path::normalize_windows_verbatim_path(path.to_path_buf()),
        };
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
    vize_carton::path::sort_dedup_paths(&mut invalidation_paths);
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
mod path_cache_tests;
#[cfg(test)]
mod tests;
