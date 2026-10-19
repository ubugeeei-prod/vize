//! Stamp-validated reachability scans shared across editor preparations.
//!
//! An editor edit rebuilds its alias context, and every rebuild used to scan
//! each package route's source graph again. The scan only depends on the files
//! it consulted, so a result stays reusable while those inputs keep their
//! stamps, exactly like a cached route lookup.

use std::path::PathBuf;

use vize_carton::String;

use super::super::PackageResolutionContext;
use super::super::stamp::{
    InputStamp, stamp_paths, stamp_paths_with_cache, stamps_are_current,
    stamps_are_current_with_cache,
};
use super::PackageRouteResolver;
use crate::batch::virtual_project::PackageRouteReachability;

/// `(resolution scope, importer, manifest, specifier, context, budget)`. The
/// scope identifies the aliases and compiler options the scan resolved with.
pub(crate) type ReachabilityCacheKey =
    (u64, PathBuf, PathBuf, String, PackageResolutionContext, u8);

const REACHABILITY_CACHE_CAPACITY: usize = 512;

pub(super) struct CachedReachability {
    reachability: PackageRouteReachability,
    stamps: Vec<InputStamp>,
    last_used: u64,
    last_validated_epoch: u64,
}

impl PackageRouteResolver {
    /// A previous scan whose consulted inputs are unchanged.
    pub(crate) fn cached_reachability(
        &self,
        key: &ReachabilityCacheKey,
    ) -> Option<PackageRouteReachability> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let epoch = state.validation_epoch;
        let (validated, stamps) = state
            .reachability
            .get(key)
            .map(|cached| (cached.last_validated_epoch, cached.stamps.clone()))?;
        let current = (epoch != 0 && validated == epoch)
            || if epoch == 0 {
                stamps_are_current(&stamps)
            } else {
                stamps_are_current_with_cache(&stamps, &mut state.stamp_snapshots)
            };
        if !current {
            state.reachability.remove(key);
            return None;
        }
        state.clock = state.clock.wrapping_add(1);
        let clock = state.clock;
        let cached = state.reachability.get_mut(key)?;
        cached.last_used = clock;
        cached.last_validated_epoch = epoch;
        Some(cached.reachability.clone())
    }

    pub(crate) fn store_reachability(
        &self,
        key: ReachabilityCacheKey,
        reachability: &PackageRouteReachability,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let stamps = if state.validation_epoch == 0 {
            stamp_paths(&reachability.inputs)
        } else {
            stamp_paths_with_cache(&reachability.inputs, &mut state.stamp_snapshots)
        };
        if state.reachability.len() >= REACHABILITY_CACHE_CAPACITY
            && !state.reachability.contains_key(&key)
        {
            let oldest = state
                .reachability
                .iter()
                .min_by_key(|(_, cached)| cached.last_used)
                .map(|(key, _)| key.clone());
            if let Some(oldest) = oldest {
                state.reachability.remove(&oldest);
            }
        }
        state.clock = state.clock.wrapping_add(1);
        let entry = CachedReachability {
            reachability: reachability.clone(),
            stamps,
            last_used: state.clock,
            last_validated_epoch: state.validation_epoch,
        };
        state.reachability.insert(key, entry);
    }
}
