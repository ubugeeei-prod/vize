//! Public and crate-local entry points for dependency reachability walks.

use std::path::PathBuf;

use vize_carton::{FxHashMap, String as CompactString};

use crate::batch::error::CorsaResult;
use crate::batch::import_rewriter::ModuleSpecifierFactsCache;

use super::{PackageResolver, VirtualProject};

impl VirtualProject {
    /// Register every reachable first-party dependency, to a fixpoint.
    pub fn register_reachable_dependencies(&mut self) -> CorsaResult<()> {
        vize_carton::profile!(
            "canon.project.reachable_dependencies",
            self.register_reachable_dependencies_with_overlays(&FxHashMap::default())
        )
    }

    /// Reconcile only dependencies reachable from sources rebuilt by one
    /// persistent patch. Existing edge ownership covers the untouched graph.
    pub(crate) fn register_reachable_dependencies_from(
        &mut self,
        sources: &[PathBuf],
    ) -> CorsaResult<()> {
        self.register_reachable_dependencies_inner(
            Some(sources),
            &FxHashMap::default(),
            &[],
            None,
            None,
        )
    }

    /// Run the same walk with unsaved editor buffers standing in for their
    /// on-disk contents.
    ///
    /// An editor session must see an import while it is still unsaved so the
    /// reachable mirror exists before the buffer is written to disk (#3900).
    pub(crate) fn register_reachable_dependencies_with_overlays(
        &mut self,
        overlays: &FxHashMap<PathBuf, &str>,
    ) -> CorsaResult<()> {
        self.register_reachable_dependencies_inner(None, overlays, &[], None, None)
    }

    #[cfg(test)]
    pub(crate) fn register_reachable_dependencies_with_package_resolver(
        &mut self,
        overlays: &FxHashMap<PathBuf, &str>,
        workspace_package_specifiers: &[CompactString],
        package_resolver: PackageResolver<'_>,
    ) -> CorsaResult<()> {
        self.register_reachable_dependencies_inner(
            None,
            overlays,
            workspace_package_specifiers,
            Some(package_resolver),
            None,
        )
    }

    /// Share only immutable syntax facts across one preparation's walks.
    /// Resolution, registration and edge replacement still run on every walk.
    pub(crate) fn register_reachable_dependencies_with_import_facts(
        &mut self,
        overlays: &FxHashMap<PathBuf, &str>,
        workspace_package_specifiers: &[CompactString],
        package_resolver: PackageResolver<'_>,
        facts: &mut ModuleSpecifierFactsCache,
    ) -> CorsaResult<()> {
        self.register_reachable_dependencies_inner(
            None,
            overlays,
            workspace_package_specifiers,
            Some(package_resolver),
            Some(facts),
        )
    }
}
