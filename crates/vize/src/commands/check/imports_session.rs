use vize_canon::PackageRouteResolver;

use super::cache::{PackageLookupCache, ResolutionContextCache};
use super::registration::VirtualRegistrationCache;

/// Memo state shared by all import walks for one check program.
#[derive(Default)]
pub(in crate::commands::check) struct LocalImportSession {
    pub(super) registration_cache: VirtualRegistrationCache,
    pub(super) resolution_contexts: ResolutionContextCache,
    pub(super) package_lookups: PackageLookupCache,
}

impl LocalImportSession {
    pub(in crate::commands::check) fn new(packages: &mut PackageRouteResolver) -> Self {
        packages.begin_validation_epoch();
        Self::default()
    }

    #[cfg(test)]
    pub(super) fn package_lookup_entries(&self) -> usize {
        self.package_lookups.len()
    }
}
