//! Package lookups whose filesystem walk does not hold the resolver lock.
//!
//! Independent specifiers can overlap. The resolution map is still updated
//! under the lock, with the same hit and eviction rules as the serial lookup.
//! Each miss borrows a search and path cache from the resolver, so manifests
//! and realpaths are not read again for every specifier.

use std::path::Path;

use super::super::search::PackageSearchCache;
use super::super::stamp::{stamp_paths, stamps_are_current, stamps_are_current_with_cache};
use super::super::{
    PackagePathCache, PackageResolutionContext, PackageResolutionMode, PackageSourceOptions,
    logical_absolute, lookup_uncached,
};
use super::lookup::PackageRouteLookup;
use super::{CachedPackageRouteLookup, PackageRouteResolver, ResolutionKey};

/// A search and path cache lent to one concurrent miss at a time.
#[derive(Default)]
pub(super) struct ConcurrentLookupCaches {
    search: PackageSearchCache,
    canonical_paths: PackagePathCache,
    epoch_generation: u64,
}

impl ConcurrentLookupCaches {
    /// Reset the same way the shared caches are on an epoch entry, and on
    /// every miss outside an epoch like the serial lookup.
    fn enter(&mut self, validation_epoch: u64, epoch_generation: u64) {
        if validation_epoch == 0 || self.epoch_generation != epoch_generation {
            self.canonical_paths.clear();
            self.search.enter_validation_epoch(validation_epoch);
            self.epoch_generation = epoch_generation;
        }
    }
}

impl PackageRouteResolver {
    /// Same context as [`Self::resolution_context`], without the resolver lock.
    /// A local manifest cache keeps the walk off the shared search map.
    pub fn resolution_context_concurrent(
        &self,
        importer: &Path,
        occurrence_mode: PackageResolutionMode,
        module_resolution: Option<&str>,
        module: Option<&str>,
        active_conditions: impl IntoIterator<Item = impl Into<super::String>>,
    ) -> (PackageResolutionContext, Vec<std::path::PathBuf>) {
        let mut search = PackageSearchCache::default();
        super::resolution_context_from_search(
            importer,
            occurrence_mode,
            module_resolution,
            module,
            active_conditions,
            &mut search,
        )
    }

    /// Cache-aware lookup. A hit stays on the caller; a miss walks the
    /// filesystem outside the resolver lock.
    pub fn lookup_with_context_concurrent(
        &self,
        importer_dir: &Path,
        specifier: &str,
        options: PackageSourceOptions,
        context: PackageResolutionContext,
    ) -> PackageRouteLookup {
        let logical_importer_dir = logical_absolute(importer_dir);
        let key: ResolutionKey = (
            logical_importer_dir.clone(),
            specifier.into(),
            options,
            context,
        );
        let mut caches = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(cached) = take_fresh_cached(&mut state, &key) {
                return cached;
            }
            state.cache_misses += 1;
            state.resolutions.remove(&key);
            if state.validation_epoch == 0 {
                state.canonical_paths.clear();
            }
            let mut caches = state.concurrent_caches.pop().unwrap_or_default();
            caches.enter(state.validation_epoch, state.epoch_generation);
            caches
        };

        let reads_before = caches.search.manifest_reads();
        let lookup = lookup_uncached(
            &logical_importer_dir,
            specifier,
            options,
            &mut caches.search,
            &mut caches.canonical_paths,
        );
        let manifest_reads = caches.search.manifest_reads() - reads_before;
        let stamps = stamp_paths(&lookup.invalidation_paths);

        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.resolutions.len() >= super::RESOLUTION_CACHE_CAPACITY {
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
        let validation_epoch = state.validation_epoch;
        state.search.add_manifest_reads(manifest_reads);
        state.concurrent_caches.push(caches);
        state.resolutions.insert(
            key,
            CachedPackageRouteLookup {
                lookup: lookup.clone(),
                stamps,
                last_used: clock,
                last_validated_epoch: validation_epoch,
            },
        );
        lookup
    }
}

fn take_fresh_cached(
    state: &mut super::PackageRouteResolverState,
    key: &ResolutionKey,
) -> Option<PackageRouteLookup> {
    let validation_epoch = state.validation_epoch;
    let cached_entry = state.resolutions.get(key).map(|cached| {
        (
            cached.last_validated_epoch,
            cached.stamps.clone(),
            cached.lookup.clone(),
        )
    });
    let cached = cached_entry.and_then(|(last_validated_epoch, stamps, lookup)| {
        let current = validation_epoch != 0 && last_validated_epoch == validation_epoch
            || if validation_epoch == 0 {
                stamps_are_current(&stamps)
            } else {
                stamps_are_current_with_cache(&stamps, &mut state.stamp_snapshots)
            };
        if current {
            if validation_epoch != 0
                && let Some(cached) = state.resolutions.get_mut(key)
            {
                cached.last_validated_epoch = validation_epoch;
            }
            Some(lookup)
        } else {
            None
        }
    })?;
    state.clock = state.clock.wrapping_add(1);
    let clock = state.clock;
    if let Some(entry) = state.resolutions.get_mut(key) {
        entry.last_used = clock;
    }
    state.cache_hits += 1;
    Some(cached)
}
