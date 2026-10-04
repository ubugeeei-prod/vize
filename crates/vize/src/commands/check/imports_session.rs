use vize_canon::PackageRouteResolver;

use super::cache::{PackageLookupCache, ResolutionContextCache};
use super::registration::VirtualRegistrationCache;

#[path = "imports_source_cache.rs"]
mod source_cache;
pub(super) use source_cache::SourceOccurrenceCache;

/// Memo state shared by all import walks for one check program.
#[derive(Default)]
pub(in crate::commands::check) struct LocalImportSession {
    pub(super) registration_cache: VirtualRegistrationCache,
    pub(super) resolution_contexts: ResolutionContextCache,
    pub(super) package_lookups: PackageLookupCache,
    pub(super) source_occurrences: SourceOccurrenceCache,
}

impl LocalImportSession {
    pub(in crate::commands::check) fn new(packages: &mut PackageRouteResolver) -> Self {
        packages.begin_validation_epoch();
        Self::default()
    }

    /// Default program preparation walks roots again after adding ambient
    /// declarations. One-pass explicit scopes do not retain source bytes.
    pub(in crate::commands::check) fn with_source_reuse(
        packages: &mut PackageRouteResolver,
    ) -> Self {
        let mut session = Self::new(packages);
        session.source_occurrences = SourceOccurrenceCache::with_reuse();
        session
    }

    #[cfg(test)]
    pub(super) fn package_lookup_entries(&self) -> usize {
        self.package_lookups.len()
    }
}
