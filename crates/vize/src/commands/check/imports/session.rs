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
    resolution: u64,
    options: ImportFileOptions,
}

/// The inputs a relative or package edge depends on besides its importer:
/// the import options and the compiler options package contexts derive from.
/// `paths` and `baseUrl` stay out, so programs differing only there share
/// their edges.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct EdgeScope {
    resolution: u64,
    options: ImportFileOptions,
}

impl AliasScope {
    pub(super) fn new(aliases: Option<&PathAliasResolver>, options: ImportFileOptions) -> Self {
        Self {
            aliases: aliases.map_or(0, PathAliasResolver::fingerprint),
            resolution: PathAliasResolver::resolution_fingerprint(aliases),
            options,
        }
    }

    /// Keys package resolution contexts, which ignore `paths` and options.
    pub(super) fn resolution(self) -> u64 {
        self.resolution
    }

    pub(super) fn edges(self) -> EdgeScope {
        EdgeScope {
            resolution: self.resolution,
            options: self.options,
        }
    }

    /// One value per alias configuration, for caches keyed by an integer.
    pub(super) fn token(self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::hash::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

/// Memo state shared by all import walks of one check invocation. Facts that
/// depend only on their keys outlive a program; the rest starts over in
/// [`Self::begin_program`].
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
        let mut session = Self::default();
        session.begin_program(packages);
        session
    }

    /// Start the walks of another program. Referenced programs of one shell
    /// reach mostly the same packages, so parsed sources, package lookups,
    /// contexts and reachability scans carry over; alias-dependent
    /// registrations do not.
    pub(in crate::commands::check) fn begin_program(
        &mut self,
        packages: &mut PackageRouteResolver,
    ) {
        packages.begin_validation_epoch();
        self.registration_cache.begin_program();
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
