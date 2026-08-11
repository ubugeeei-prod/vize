//! Cold/editor package-link collection shared with affected warm ownership.

use std::path::PathBuf;

use vize_carton::FxHashSet;

use super::VirtualProject;
use super::package_node_modules::{PackageNodeModulesLink, package_node_modules_links};

impl VirtualProject {
    pub(crate) fn capture_materialized_package_links(&mut self) {
        self.materialized_package_links = self.desired_package_links();
        self.rebuild_materialized_package_link_owners();
        self.incremental_link_topology_dirty = false;
    }

    fn package_node_modules_links(
        &self,
        expected_files: &FxHashSet<PathBuf>,
    ) -> Vec<PackageNodeModulesLink> {
        package_node_modules_links(
            &self.project_root,
            &self.virtual_root,
            expected_files.iter().map(PathBuf::as_path),
        )
    }

    pub(crate) fn desired_package_links(&self) -> vize_carton::FxHashMap<PathBuf, PathBuf> {
        let expected_files = self.expected_materialized_files();
        self.desired_package_links_for_files(&expected_files)
    }

    pub(super) fn package_links_for_files(
        &self,
        expected_files: &FxHashSet<PathBuf>,
    ) -> Vec<PackageNodeModulesLink> {
        let mut links = self.package_node_modules_links(expected_files);
        links.extend(self.package_shadow_dependency_links(expected_files));
        links.sort_by(|left, right| {
            (&left.virtual_dir, &left.real_dir).cmp(&(&right.virtual_dir, &right.real_dir))
        });
        links.dedup();
        links
    }

    pub(super) fn desired_package_links_for_files(
        &self,
        expected_files: &FxHashSet<PathBuf>,
    ) -> vize_carton::FxHashMap<PathBuf, PathBuf> {
        let links = self.package_links_for_files(expected_files);
        let mut desired = vize_carton::FxHashMap::default();
        for link in links {
            let real_dir = std::fs::canonicalize(&link.real_dir)
                .map(vize_carton::path::normalize_windows_verbatim_path)
                .unwrap_or_else(|_| vize_carton::path::canonicalize_non_verbatim(&link.real_dir));
            desired.entry(link.virtual_dir).or_insert(real_dir);
        }
        desired
    }

    pub(super) fn preserved_prune_roots(
        &self,
        package_links: &[PackageNodeModulesLink],
    ) -> Vec<PathBuf> {
        let mut roots = Vec::with_capacity(package_links.len() + 1);
        roots.push(self.virtual_root.join("node_modules"));
        roots.extend(package_links.iter().map(|link| link.virtual_dir.clone()));
        roots
    }
}
