//! Coherent package-resolution snapshots scoped to one import collection.

#![expect(
    clippy::disallowed_types,
    reason = "a lookup remains live while recursive registration mutates its session caches"
)]

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;

use vize_canon::{
    PackageResolutionContext, PackageResolutionMode, PackageRoute, PackageRouteLookup,
    PackageSourceOptions,
};
use vize_l0::{FxHashMap, String};

/// `(resolution scope, importer directory, importer extension, mode)`: programs
/// whose module options agree share contexts.
type ResolutionContextKey = (u64, PathBuf, Option<OsString>, PackageResolutionMode);
pub(super) type ResolutionContextCache =
    FxHashMap<ResolutionContextKey, (PackageResolutionContext, Vec<PathBuf>)>;

type PackageLookupKey = (
    PathBuf,
    String,
    PackageSourceOptions,
    PackageResolutionContext,
);
pub(super) type PackageLookupCache = FxHashMap<PackageLookupKey, CachedPackageLookup>;

/// The resolver answer is immutable for the session's existing cache key. A
/// reader only needs an owned route when it publishes a materialized binding;
/// cloning every nested route before that decision copies discarded payloads.
#[derive(Clone)]
pub(super) struct CachedPackageLookup {
    route: Option<Arc<PackageRoute>>,
    invalidation_paths: Arc<[PathBuf]>,
    watchable_negative: bool,
}

impl From<PackageRouteLookup> for CachedPackageLookup {
    fn from(lookup: PackageRouteLookup) -> Self {
        let watchable_negative = lookup.is_watchable_negative();
        let (route, invalidation_paths) = lookup.into_parts();
        Self {
            route: route.map(Arc::new),
            invalidation_paths: invalidation_paths.into(),
            watchable_negative,
        }
    }
}

impl CachedPackageLookup {
    pub(super) fn route(&self) -> Option<&PackageRoute> {
        self.route.as_deref()
    }

    pub(super) fn is_watchable_negative(&self) -> bool {
        self.watchable_negative
    }

    /// Each consumer still owns its ordered invalidation paths: callers add
    /// context and importer inputs without changing the cached resolver answer.
    pub(super) fn into_parts(self) -> (Option<Arc<PackageRoute>>, Vec<PathBuf>) {
        (self.route, self.invalidation_paths.to_vec())
    }
}

#[cfg(test)]
mod tests;
