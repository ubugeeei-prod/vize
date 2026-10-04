//! Preserve real in-package install scopes inside workspace shadow copies.

use std::path::{Path, PathBuf};

use vize_carton::FxHashSet;

use super::{PackageShadowTopology, VirtualProject};

impl VirtualProject {
    pub(super) fn private_dependencies_for(&self, manifest: &Path) -> Vec<crate::PackageRoute> {
        let mut routes = self
            .package_route_manifests
            .get(manifest)
            .into_iter()
            .flatten()
            .filter_map(|key| self.package_routes.get(key))
            .filter(|binding| binding.specifier.starts_with('#'))
            .filter_map(|binding| binding.route.as_ref())
            .flat_map(|route| route.nested_routes.iter().cloned())
            .collect::<Vec<_>>();
        routes.sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));
        routes.dedup();
        routes
    }

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
        for source in sources.keys() {
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
                let Ok(relative) = dependency.package_link_root.strip_prefix(&root) else {
                    // A root/shared install already uses the common shadow
                    // scope. Inventing deeper copies splits class identity.
                    continue;
                };
                if !relative.ends_with(Path::new("node_modules").join(name)) {
                    continue;
                }
                dependencies.push((shadow_root.join(relative), dependency));
            }
        }
        dependencies.sort_by(|(left, left_route), (right, right_route)| {
            (left, &left_route.manifest_path).cmp(&(right, &right_route.manifest_path))
        });
        dependencies.dedup();
        for (shadow, dependency) in dependencies {
            // A real per-package node_modules link must not outrank the bound
            // materialized dependency and lead native probing to raw Vue files.
            // Rebase the resolver's selected real link, preserving one shared
            // namespace for importers that use the same installation scope.
            self.collect_route_shadow_topology(dependency, &shadow, ancestors, topology);
        }
    }
}
