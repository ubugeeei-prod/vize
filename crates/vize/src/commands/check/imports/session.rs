use std::path::PathBuf;

use vize_canon::PackageRouteResolver;
use vize_l0::{FxHashMap, String};

use super::super::imports_aliases::PathAliasResolver;
use super::ImportFileOptions;
use super::cache::{PackageLookupCache, ResolutionContextCache};
use super::registration::VirtualRegistrationCache;

/// The inputs a `paths` target depends on: the alias configuration and the
/// import options of the walk that resolved it. Walks sharing one session may
/// differ in either, so each scope keeps its own targets.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct AliasScope {
    aliases: u64,
    options: ImportFileOptions,
}

impl AliasScope {
    pub(super) fn new(aliases: Option<&PathAliasResolver>, options: ImportFileOptions) -> Self {
        Self {
            aliases: aliases.map_or(0, PathAliasResolver::fingerprint),
            options,
        }
    }
}

/// Memo state shared by all import walks for one check program.
#[derive(Default)]
pub(in crate::commands::check) struct LocalImportSession {
    pub(super) registration_cache: VirtualRegistrationCache,
    pub(super) resolution_contexts: ResolutionContextCache,
    pub(super) package_lookups: PackageLookupCache,
    /// `paths` alias targets per [`AliasScope`], shared by the import walk and
    /// every registration closure it scans under the same scope.
    alias_targets_by_scope: FxHashMap<AliasScope, FxHashMap<String, Option<PathBuf>>>,
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

    /// The `paths` targets resolved under `scope`, if any walk filled them.
    pub(super) fn alias_targets(
        &self,
        scope: AliasScope,
    ) -> Option<&FxHashMap<String, Option<PathBuf>>> {
        self.alias_targets_by_scope.get(&scope)
    }

    pub(super) fn alias_targets_mut(
        &mut self,
        scope: AliasScope,
    ) -> &mut FxHashMap<String, Option<PathBuf>> {
        self.alias_targets_by_scope.entry(scope).or_default()
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
