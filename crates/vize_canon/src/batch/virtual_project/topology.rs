//! Deterministic project-topology counters for one scanned batch project.
//!
//! These are exact set cardinalities, not timings: a performance gate can pin
//! them on a shared, noisy machine and still fail closed when the project
//! materializes or type-checks the same authored file more than once (#4153).

use super::VirtualProject;

/// Exact per-phase membership of one scanned batch project.
///
/// Every field is a set cardinality taken after `scan_paths`, so two runs over
/// the same tree with the same inputs produce identical values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BatchTopologyMetrics {
    /// Caller-selected source roots eligible for diagnostics and declarations.
    pub scan_roots: usize,
    /// Registered generated/rewritten sources (the discovery + materialization
    /// phase result reported as "Running Corsa diagnostics for N files").
    pub virtual_files: usize,
    /// Non-TS modules copied verbatim for module resolution.
    pub passthrough_files: usize,
    /// Importer-scoped package identities retained by the project.
    pub package_route_bindings: usize,
    /// Bindings that resolved to a physical package.
    pub resolved_package_routes: usize,
    /// Distinct materialized package shadow scopes (one manifest each).
    pub package_shadow_scopes: usize,
    /// Materialized package shadow source copies.
    pub package_shadow_files: usize,
    /// Resolved importer -> dependency edges owning inferred sources.
    pub dependency_edges: usize,
    /// Files the materialize pass expects to exist in the mirror.
    pub materialized_files: usize,
    /// Root files listed in the generated program's `include`.
    pub native_program_files: usize,
}

impl BatchTopologyMetrics {
    /// Materialized shadow copies per registered virtual file.
    ///
    /// One shadow scope per physical package keeps this bounded by the number
    /// of packages; one scope per importing directory does not.
    pub fn shadow_copies_per_virtual_file(&self) -> f64 {
        if self.virtual_files == 0 {
            return 0.0;
        }
        self.package_shadow_files as f64 / self.virtual_files as f64
    }

    /// Program root files per registered virtual file. A program that lists
    /// each authored file a bounded number of times keeps this near its
    /// generated-companion constant; importer-scoped duplication scales it with
    /// the number of importing directories.
    pub fn program_files_per_virtual_file(&self) -> f64 {
        if self.virtual_files == 0 {
            return 0.0;
        }
        self.native_program_files as f64 / self.virtual_files as f64
    }
}

impl VirtualProject {
    pub(crate) fn topology_metrics(&self) -> BatchTopologyMetrics {
        BatchTopologyMetrics {
            scan_roots: self
                .declaration_roots
                .as_ref()
                .map_or(0, vize_carton::FxHashSet::len),
            virtual_files: self.virtual_files.len(),
            passthrough_files: self.passthrough_files.len(),
            package_route_bindings: self.package_routes.len(),
            resolved_package_routes: self
                .package_routes
                .values()
                .filter(|binding| binding.route.is_some())
                .count(),
            package_shadow_scopes: self.package_shadow_manifests.len(),
            package_shadow_files: self.package_shadow_files.len(),
            dependency_edges: self
                .dependency_edges
                .values()
                .map(|targets| targets.len())
                .sum(),
            materialized_files: self.expected_materialized_files().len(),
            native_program_files: self
                .include_paths(None, self.source_file_policy().allows_javascript())
                .len(),
        }
    }

    /// Materialized package shadow paths, sorted. Tests use this to pin *where*
    /// a scope landed, not only how many there are.
    #[cfg(test)]
    pub(crate) fn topology_shadow_paths(&self) -> Vec<std::path::PathBuf> {
        let mut paths = self
            .package_shadow_files
            .keys()
            .chain(self.package_shadow_manifests.keys())
            .cloned()
            .collect::<Vec<_>>();
        paths.sort();
        paths
    }

    /// Shadow paths relative to the mirror root, sorted, for exact assertions.
    #[cfg(test)]
    pub(crate) fn topology_shadow_manifest_scopes(&self) -> Vec<vize_carton::String> {
        let mut scopes = self
            .package_shadow_manifests
            .keys()
            .filter_map(|path| path.parent())
            .filter_map(|path| path.strip_prefix(&self.virtual_root).ok())
            .map(|path| vize_carton::ToCompactString::to_compact_string(&path.display()))
            .collect::<Vec<_>>();
        scopes.sort();
        scopes
    }

    /// Program root files relative to the mirror root, sorted.
    #[cfg(test)]
    pub(crate) fn topology_program_files(&self) -> Vec<vize_carton::String> {
        let mut files = self.include_paths(None, self.source_file_policy().allows_javascript());
        files.sort();
        files
    }
}
