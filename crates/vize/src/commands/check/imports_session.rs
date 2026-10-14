use std::path::PathBuf;

use vize_canon::PackageRouteResolver;
use vize_l0::FxHashMap;

use super::cache::{PackageLookupCache, ResolutionContextCache};
use super::registration::VirtualRegistrationCache;
use super::specifiers::ModuleSpecifierOccurrence;

/// Memo state shared by all import walks for one check program.
#[derive(Default)]
pub(in crate::commands::check) struct LocalImportSession {
    pub(super) registration_cache: VirtualRegistrationCache,
    pub(super) resolution_contexts: ResolutionContextCache,
    pub(super) package_lookups: PackageLookupCache,
    /// `None` records an unreadable file.
    pub(super) module_specifiers: FxHashMap<PathBuf, Option<Vec<ModuleSpecifierOccurrence>>>,
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
        self.module_specifiers.len()
    }
}
