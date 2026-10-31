//! Caller-owned syntax facts for otherwise unchanged dependency walks.

use super::{
    CompactString, CorsaResult, FxHashMap, ModuleSpecifierFactsCache, PackageResolver, PathBuf,
    VirtualProject,
};

impl VirtualProject {
    pub(crate) fn register_reachable_dependencies_with_facts(
        &mut self,
        facts: &mut ModuleSpecifierFactsCache,
    ) -> CorsaResult<()> {
        vize_carton::profile!(
            "canon.project.reachable_dependencies",
            self.register_reachable_dependencies_inner(
                None,
                &FxHashMap::default(),
                &[],
                None,
                Some(facts),
            )
        )
    }

    /// Share immutable syntax only. Resolution, registration and edge
    /// replacement still run on every walk with their current caller inputs.
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
