//! Preserve authored importer scopes inside a package's shadow copies.

use std::path::{Path, PathBuf};

use vize_carton::FxHashSet;

use super::{PackageShadowTopology, VirtualProject};

impl VirtualProject {
    pub(super) fn collect_workspace_dependency_shadows(
        &self,
        route: &crate::PackageRoute,
        shadow_root: &Path,
        ancestors: &mut FxHashSet<PathBuf>,
        topology: &mut PackageShadowTopology,
    ) {
        if !route.workspace_source {
            return;
        }
        let root = vize_carton::path::canonicalize_non_verbatim(&route.package_root);
        let Some(sources) = self.package_source_index.get(&root) else {
            return;
        };
        let mut dependencies = Vec::new();
        for (source, (relative, _)) in sources {
            let Some(keys) = self.package_route_importers.get(source) else {
                continue;
            };
            for key in keys {
                let Some(binding) = self.package_routes.get(key) else {
                    continue;
                };
                // Private imports already retain their manifest-keyed topology.
                if binding.specifier.starts_with('#') {
                    continue;
                }
                let Some(dependency) = binding.route.as_ref() else {
                    continue;
                };
                let Some(name) = dependency.package_name.as_deref() else {
                    continue;
                };
                let importer_dir = shadow_root.join(relative.parent().unwrap_or(Path::new("")));
                dependencies.push((importer_dir.join("node_modules").join(name), dependency));
            }
        }
        dependencies.sort_by(|(left, left_route), (right, right_route)| {
            (left, &left_route.manifest_path).cmp(&(right, &right_route.manifest_path))
        });
        for (shadow, dependency) in dependencies {
            // A real per-package node_modules link must not outrank the bound
            // materialized dependency and lead native probing to raw Vue files.
            // Placing it at this exact authored importer's rebased directory
            // preserves divergent package identities in different source dirs.
            self.collect_route_shadow_topology(dependency, &shadow, ancestors, topology);
        }
    }
}
