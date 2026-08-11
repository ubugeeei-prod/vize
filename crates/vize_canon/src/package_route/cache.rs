use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, String};

use super::search::PackageSearchCache;
use super::stamp::{InputStamp, stamp_paths, stamps_are_current};
use super::{
    PackageResolutionContext, PackageRoute, PackageRouteMetrics, PackageSourceOptions,
    logical_absolute, lookup_uncached,
};

type ResolutionKey = (
    PathBuf,
    String,
    PackageSourceOptions,
    PackageResolutionContext,
);

const RESOLUTION_CACHE_CAPACITY: usize = 2_048;

#[derive(Clone, Default)]
pub struct PackageRouteResolver {
    // One cache identity is deliberately shared by CLI, Corsa, Maestro, and
    // persistent check-server consumers. A scoped reference cannot outlive
    // those independently owned sessions.
    #[allow(clippy::disallowed_types)]
    state: std::sync::Arc<std::sync::Mutex<PackageRouteResolverState>>,
}

#[derive(Default)]
struct PackageRouteResolverState {
    resolutions: FxHashMap<ResolutionKey, CachedPackageRouteLookup>,
    search: PackageSearchCache,
    cache_hits: u64,
    cache_misses: u64,
    refresh_considered_routes: u64,
    refresh_affected_routes: u64,
    last_refresh_considered_routes: u64,
    last_refresh_affected_routes: u64,
    last_refresh_total_routes: u64,
    clock: u64,
    resolution_evictions: u64,
}

struct CachedPackageRouteLookup {
    lookup: PackageRouteLookup,
    stamps: Vec<InputStamp>,
    last_used: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PackageRouteLookup {
    pub(super) route: Option<PackageRoute>,
    pub(super) invalidation_paths: Vec<PathBuf>,
    pub(super) watchable_negative: bool,
}

impl PackageRouteLookup {
    pub fn into_parts(self) -> (Option<PackageRoute>, Vec<PathBuf>) {
        (self.route, self.invalidation_paths)
    }

    pub fn is_watchable_negative(&self) -> bool {
        self.route.is_none() && self.watchable_negative
    }
}

impl PackageRouteResolver {
    /// Derive one importer occurrence context through the resolver-owned
    /// manifest cache. The returned inputs join the binding's reverse index.
    pub fn resolution_context(
        &mut self,
        importer: &Path,
        occurrence_mode: super::PackageResolutionMode,
        module_resolution: Option<&str>,
        module: Option<&str>,
        active_conditions: impl IntoIterator<Item = impl Into<String>>,
    ) -> (PackageResolutionContext, Vec<PathBuf>) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut inputs = Vec::new();
        let start = importer.parent().unwrap_or(importer);
        let nearest_scope =
            super::search::nearest_package_manifest(start, &mut inputs, &mut state.search);
        let package_is_module = nearest_scope.as_ref().map(|(_, manifest)| {
            manifest.get("type").and_then(serde_json::Value::as_str) == Some("module")
        });
        let mut context = PackageResolutionContext::for_importer(
            importer,
            occurrence_mode,
            module_resolution,
            module,
            active_conditions,
            package_is_module,
        );
        context.scope_manifest_path = nearest_scope.map(|(root, _)| root.join("package.json"));
        (context, inputs)
    }

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
        self.lookup_with_context(importer_dir, specifier, options, Default::default())
    }

    pub fn lookup_with_context(
        &mut self,
        importer_dir: &Path,
        specifier: &str,
        options: PackageSourceOptions,
        context: PackageResolutionContext,
    ) -> PackageRouteLookup {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Key on the logical importer directory: resolution walks the logical
        // ancestors, so two importer directories that share a canonical path
        // through a package symlink still see different `node_modules` chains.
        let logical_importer_dir = logical_absolute(importer_dir);
        let key = (
            logical_importer_dir.clone(),
            specifier.into(),
            options,
            context,
        );
        let cached = state
            .resolutions
            .get(&key)
            .filter(|cached| stamps_are_current(&cached.stamps))
            .map(|cached| cached.lookup.clone());
        if let Some(cached) = cached {
            state.clock = state.clock.wrapping_add(1);
            let clock = state.clock;
            if let Some(entry) = state.resolutions.get_mut(&key) {
                entry.last_used = clock;
            }
            state.cache_hits += 1;
            return cached;
        }
        state.cache_misses += 1;
        state.resolutions.remove(&key);
        let lookup = lookup_uncached(&logical_importer_dir, specifier, options, &mut state.search);
        let stamps = stamp_paths(&lookup.invalidation_paths);
        if state.resolutions.len() >= RESOLUTION_CACHE_CAPACITY {
            let oldest = state
                .resolutions
                .iter()
                .min_by_key(|(_, cached)| cached.last_used)
                .map(|(key, _)| key.clone());
            if let Some(oldest) = oldest {
                state.resolutions.remove(&oldest);
                state.resolution_evictions += 1;
            }
        }
        state.clock = state.clock.wrapping_add(1);
        let clock = state.clock;
        state.resolutions.insert(
            key,
            CachedPackageRouteLookup {
                lookup: lookup.clone(),
                stamps,
                last_used: clock,
            },
        );
        lookup
    }

    pub fn clear(&mut self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.resolutions.clear();
        state.search.clear();
        state.cache_hits = 0;
        state.cache_misses = 0;
        state.refresh_considered_routes = 0;
        state.refresh_affected_routes = 0;
        state.last_refresh_considered_routes = 0;
        state.last_refresh_affected_routes = 0;
        state.last_refresh_total_routes = 0;
        state.clock = 0;
        state.resolution_evictions = 0;
    }

    #[cfg(feature = "native")]
    pub(crate) fn record_refresh_scope(
        &mut self,
        total: usize,
        considered: usize,
        affected: usize,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.refresh_considered_routes += considered as u64;
        state.refresh_affected_routes += affected as u64;
        state.last_refresh_considered_routes = considered as u64;
        state.last_refresh_affected_routes = affected as u64;
        state.last_refresh_total_routes = total as u64;
    }

    pub fn metrics(&self) -> PackageRouteMetrics {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        PackageRouteMetrics {
            cache_hits: state.cache_hits,
            cache_misses: state.cache_misses,
            manifest_reads: state.search.manifest_reads(),
            refresh_considered_routes: state.refresh_considered_routes,
            refresh_affected_routes: state.refresh_affected_routes,
            last_refresh_considered_routes: state.last_refresh_considered_routes,
            last_refresh_affected_routes: state.last_refresh_affected_routes,
            last_refresh_total_routes: state.last_refresh_total_routes,
            resolution_cache_entries: state.resolutions.len() as u64,
            manifest_cache_entries: state.search.len() as u64,
            resolution_cache_evictions: state.resolution_evictions,
            manifest_cache_evictions: state.search.evictions(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PackageRouteResolver, RESOLUTION_CACHE_CAPACITY};

    #[test]
    fn route_cache_has_a_measured_hard_bound() {
        let root = tempfile::tempdir().unwrap();
        let importer = root.path().join("src");
        std::fs::create_dir_all(&importer).unwrap();
        let mut resolver = PackageRouteResolver::default();
        for index in 0..=RESOLUTION_CACHE_CAPACITY {
            let _ = resolver.lookup(
                &importer,
                &format!("package-{index}"),
                crate::PackageSourceOptions::new(false, false),
            );
        }

        let metrics = resolver.metrics();
        assert_eq!(
            metrics.resolution_cache_entries,
            RESOLUTION_CACHE_CAPACITY as u64
        );
        assert_eq!(metrics.resolution_cache_evictions, 1);
    }
}
