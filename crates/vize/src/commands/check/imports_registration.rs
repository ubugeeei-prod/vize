use std::path::{Path, PathBuf};

use vize_l0::{FxHashMap, FxHashSet};

use super::super::imports_aliases::PathAliasResolver;
use super::super::path_cache::CanonicalPathCache;
use super::{
    ImportFileOptions, extract_module_specifier_occurrences, is_declaration_file,
    is_relative_specifier, resolve_import_base, resolve_import_base_with_inputs,
    resolve_relative_import,
};

#[path = "imports_registration/scan.rs"]
mod scan;

use scan::source_needs_virtual_registration;

#[derive(Clone, Default)]
pub(super) struct VirtualRegistrationDiscovery {
    pub(super) package_routes: Vec<vize_canon::PackageRouteBinding>,
    pub(super) package_sources: Vec<PathBuf>,
}

#[derive(Clone)]
pub(super) struct CachedVirtualRegistration {
    needs_registration: bool,
    discovery: VirtualRegistrationDiscovery,
    package_routes_reported: bool,
}

/// Reachability of one package route never depends on the importer that
/// reached it: the bounded scan resolves nested specifiers from the package's
/// own files. Keying on the manifest, spelling, and resolution context keeps
/// the scan to once per distinct route instead of once per occurrence.
type PackageReachabilityKey = (
    PathBuf,
    vize_l0::String,
    vize_canon::PackageResolutionContext,
    u8,
);

#[derive(Default)]
pub(super) struct VirtualRegistrationCache {
    registrations: FxHashMap<(PathBuf, bool), CachedVirtualRegistration>,
    reachability: FxHashMap<PackageReachabilityKey, vize_canon::batch::PackageRouteReachability>,
    negative_alias_sources: FxHashSet<PathBuf>,
    #[cfg(test)]
    source_reads: usize,
}

impl VirtualRegistrationCache {
    #[cfg(test)]
    pub(super) fn registration_entries(&self) -> usize {
        self.registrations.len()
    }

    #[cfg(test)]
    pub(super) fn source_reads(&self) -> usize {
        self.source_reads
    }
}

/// One walk of the sources reachable from a single registration candidate.
#[derive(Default)]
struct RegistrationFrontier {
    visited: FxHashSet<PathBuf>,
    queue: Vec<PathBuf>,
    #[cfg(test)]
    source_reads: usize,
}

/// A file inside a package that imports that package's own name pulls the
/// package entry, not an internal alias such as `#/composables/file`.
pub(super) fn importer_is_inside_own_package_entry(
    importer: &Path,
    route: &vize_canon::PackageRoute,
    specifier: &str,
) -> bool {
    let Some(name) = route.package_name.as_deref() else {
        return false;
    };
    let own_entry = specifier == name
        || specifier
            .strip_prefix(name)
            .is_some_and(|rest| rest.starts_with('/'));
    own_entry && importer.starts_with(&route.package_root)
}

pub(super) fn non_relative_import_needs_virtual_registration(
    path: &Path,
    canonical_paths: &mut CanonicalPathCache,
    options: ImportFileOptions,
    aliases: Option<&PathAliasResolver>,
    packages: Option<&mut vize_canon::PackageRouteResolver>,
    cache: &mut VirtualRegistrationCache,
    discovery: &mut VirtualRegistrationDiscovery,
) -> bool {
    let pure_alias = packages.is_none();
    if pure_alias && cache.negative_alias_sources.contains(path) {
        return false;
    }
    let cache_key = (path.to_path_buf(), packages.is_some());
    if let Some(cached) = cache.registrations.get_mut(&cache_key) {
        // Route bindings are collection-global facts; sources are caller-local
        // invalidation inputs for the outer route that reached this memo.
        if !cached.package_routes_reported {
            discovery
                .package_routes
                .extend(cached.discovery.package_routes.iter().cloned());
            cached.package_routes_reported = true;
        }
        discovery
            .package_sources
            .extend(cached.discovery.package_sources.iter().cloned());
        return cached.needs_registration;
    }

    let mut frontier = RegistrationFrontier {
        queue: vec![path.to_path_buf()],
        ..Default::default()
    };
    let mut discovered = Vec::new();
    let needs_registration = source_needs_virtual_registration(
        &mut frontier,
        canonical_paths,
        options,
        aliases,
        packages,
        &mut cache.reachability,
        &mut discovered,
        &cache.negative_alias_sources,
    );
    #[cfg(test)]
    {
        cache.source_reads += frontier.source_reads;
    }
    // A completely scanned Vue-free closure proves every member negative.
    // Package walks retain their full caller-specific invalidation inputs.
    if pure_alias && !needs_registration {
        cache
            .negative_alias_sources
            .extend(frontier.visited.iter().cloned());
    }
    let resolved_discovery = VirtualRegistrationDiscovery {
        package_routes: discovered,
        package_sources: if needs_registration {
            frontier.visited.into_iter().collect()
        } else {
            Vec::new()
        },
    };
    discovery
        .package_routes
        .extend(resolved_discovery.package_routes.iter().cloned());
    discovery
        .package_sources
        .extend(resolved_discovery.package_sources.iter().cloned());
    cache.registrations.insert(
        cache_key,
        CachedVirtualRegistration {
            needs_registration,
            discovery: resolved_discovery,
            package_routes_reported: true,
        },
    );
    needs_registration
}
