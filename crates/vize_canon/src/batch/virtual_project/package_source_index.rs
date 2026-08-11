//! Physical package-root -> generated source topology index.

use std::path::Path;

use super::VirtualProject;

impl VirtualProject {
    pub(super) fn index_package_source(&mut self, source: &Path, virtual_path: &Path) {
        let source = vize_carton::path::canonicalize_non_verbatim(source);
        let roots = source
            .ancestors()
            .filter(|ancestor| self.package_route_roots.contains_key(*ancestor))
            .map(Path::to_path_buf)
            .collect::<Vec<_>>();
        for root in roots {
            let Ok(relative) = source.strip_prefix(&root) else {
                continue;
            };
            self.package_source_index.entry(root).or_default().insert(
                source.clone(),
                (relative.to_path_buf(), virtual_path.to_path_buf()),
            );
        }
    }

    pub(super) fn remove_package_source_from_index(&mut self, source: &Path) {
        let source = vize_carton::path::canonicalize_non_verbatim(source);
        for ancestor in source.ancestors() {
            if let Some(sources) = self.package_source_index.get_mut(ancestor) {
                sources.remove(&source);
            }
        }
    }

    pub(super) fn index_existing_package_route_sources(&mut self, route: &crate::PackageRoute) {
        for route in route.all_routes() {
            let root = vize_carton::path::canonicalize_non_verbatim(&route.package_root);
            self.package_route_roots.entry(root.clone()).or_default();
            for source in route.all_source_paths() {
                let source = vize_carton::path::canonicalize_non_verbatim(source);
                let Some(virtual_path) = self.original_index.get(&source).cloned() else {
                    continue;
                };
                let Ok(relative) = source.strip_prefix(&root) else {
                    continue;
                };
                let relative = relative.to_path_buf();
                self.package_source_index
                    .entry(root.clone())
                    .or_default()
                    .insert(source, (relative, virtual_path));
            }
        }
    }
}
