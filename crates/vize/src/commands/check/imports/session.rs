use super::cache::{PackageLookupCache, ResolutionContextCache};
use super::registration::VirtualRegistrationCache;
use vize_canon::PackageRouteResolver;

/// Memo state shared by all import walks for one check program.
#[derive(Default)]
pub(in crate::commands::check) struct LocalImportSession {
    pub(super) registration_cache: VirtualRegistrationCache,
    pub(super) resolution_contexts: ResolutionContextCache,
    pub(super) package_lookups: PackageLookupCache,
    /// Fires once per uncached package lookup. Tests use it to show two
    /// independent specifiers overlap instead of taking one serial turn.
    #[cfg(test)]
    pub(super) package_lookup_probe: Option<fn()>,
    /// Fires once per uncached `paths` lookup.
    #[cfg(test)]
    pub(super) path_lookup_probe: Option<fn()>,
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

    #[cfg(test)]
    pub(super) fn module_specifier_entries(&self) -> usize {
        self.registration_cache.module_specifiers.len()
    }
}
