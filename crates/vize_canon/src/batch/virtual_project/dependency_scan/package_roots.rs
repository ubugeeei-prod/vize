//! Package-local ownership from the existing authoritative route-root index.

use std::path::{Path, PathBuf};

use super::VirtualProject;

impl VirtualProject {
    pub(super) fn raw_package_roots_are_indexed(&self) -> (bool, usize) {
        let mut checked_roots = 0;
        let complete = self.package_routes.iter().all(|(key, binding)| {
            binding
                .route
                .as_ref()
                .is_none_or(|route| raw_roots_indexed(self, route, key, &mut checked_roots))
        });
        (complete, checked_roots)
    }

    pub(super) fn package_roots_for_importer(
        &self,
        importer: &Path,
        use_root_index: bool,
    ) -> Vec<PathBuf> {
        if !use_root_index {
            return self
                .package_routes
                .values()
                .filter_map(|binding| binding.route.as_ref())
                .flat_map(crate::PackageRoute::all_routes)
                .filter(|route| importer.starts_with(&route.package_root))
                .map(|route| route.package_root.clone())
                .collect();
        }
        // A raw prefix of a canonical importer is one of its ancestors. The
        // index includes canonicalized roots, so verify the route's raw root
        // too: a logical symlink spelling must not acquire new ownership.
        importer
            .ancestors()
            .filter(|root| {
                self.package_route_roots.get(*root).is_some_and(|owners| {
                    owners
                        .iter()
                        .filter_map(|key| self.package_routes.get(key)?.route.as_ref())
                        .any(|route| has_raw_root(route, root))
                })
            })
            .map(Path::to_path_buf)
            .collect()
    }
}

fn raw_roots_indexed(
    project: &VirtualProject,
    route: &crate::PackageRoute,
    key: &crate::package_route::PackageRouteKey,
    checked_roots: &mut usize,
) -> bool {
    *checked_roots += 1;
    // Relative and empty roots also have historical prefix semantics, but are
    // not ancestors of a canonical absolute importer.
    route.package_root.is_absolute()
        && project
            .package_route_roots
            .get(&route.package_root)
            .is_some_and(|owners| owners.contains(key))
        && route
            .nested_routes
            .iter()
            .all(|nested| raw_roots_indexed(project, nested, key, checked_roots))
}

fn has_raw_root(route: &crate::PackageRoute, root: &Path) -> bool {
    route.package_root == root
        || route
            .nested_routes
            .iter()
            .any(|nested| has_raw_root(nested, root))
}

#[cfg(test)]
mod tests;
