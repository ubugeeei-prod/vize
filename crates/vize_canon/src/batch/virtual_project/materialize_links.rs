//! Cold/editor package-link collection shared with affected warm ownership.

use std::path::{Path, PathBuf};

use vize_carton::FxHashSet;

use super::VirtualProject;
use super::canonical_paths::{CanonicalPathGuard, canonical_path};
use super::package_node_modules::{
    PackageNodeModulesLink, package_node_modules_links_with_passthrough,
};
use super::workspace_aliases::WorkspaceAliasPlan;

impl VirtualProject {
    pub(super) fn preserved_package_link_overlaps_files(
        &self,
        path: &Path,
        target: &Path,
        files: &FxHashSet<PathBuf>,
    ) -> bool {
        // A live member's former external package link must not override the
        // complete union's newly owned private files at that same path.
        !target.starts_with(&self.virtual_root) && files.iter().any(|file| file.starts_with(path))
    }

    /// Retain the last completed editor union's link identities. Validation
    /// still compares every existing link to its exact recorded target before
    /// retiring it; newly discovered disk links never establish ownership.
    pub(crate) fn restore_editor_package_links(
        &mut self,
        snapshot: &super::MaterializedFileSnapshot,
    ) {
        self.materialized_package_links
            .clone_from(snapshot.package_links());
    }

    #[cfg(test)]
    pub(crate) fn capture_materialized_package_links(&mut self) {
        let workspace_alias_plan = self.workspace_alias_plan();
        self.materialized_package_links = self.desired_package_links(&workspace_alias_plan);
        self.rebuild_materialized_package_link_owners(&workspace_alias_plan);
        self.materialized_workspace_alias_plan = workspace_alias_plan;
        self.incremental_link_topology_dirty = false;
    }

    pub(super) fn commit_materialized_package_links(
        &mut self,
        links: vize_carton::FxHashMap<PathBuf, PathBuf>,
        workspace_alias_plan: WorkspaceAliasPlan,
    ) {
        self.materialized_package_links = links;
        self.rebuild_materialized_package_link_owners(&workspace_alias_plan);
        self.materialized_workspace_alias_plan = workspace_alias_plan;
        self.incremental_link_topology_dirty = false;
    }

    fn package_node_modules_links(
        &self,
        expected_files: &FxHashSet<PathBuf>,
    ) -> Vec<PackageNodeModulesLink> {
        let mut passthrough: FxHashSet<PathBuf> = self.passthrough_files.keys().cloned().collect();
        passthrough.extend(self.installed_file_mirrors());
        package_node_modules_links_with_passthrough(
            &self.project_root,
            &self.virtual_root,
            expected_files.iter().map(PathBuf::as_path),
            &passthrough,
        )
    }

    /// Mirrored sources that are installed files at their own coordinates.
    ///
    /// A program can contain a dependency's file directly: Nuxt's generated
    /// declarations reference `../node_modules/<pkg>/…` by path, and a
    /// `paths` target can name one. The mirror rewrites that one file, but the
    /// package around it is still the installed one: its `package.json`, the
    /// `exports` targets and every sibling declaration must keep resolving
    /// through the real install, or a bare `import "<pkg>"` reports `TS2307`
    /// beside a file of that very package. Such a file therefore owns only
    /// itself, exactly like a passthrough copy (#9690). Package shadows, which
    /// do own their package directory, are separate maps.
    fn installed_file_mirrors(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.virtual_files.values().filter_map(|file| {
            let relative = file.virtual_path.strip_prefix(&self.virtual_root).ok()?;
            let installed = relative
                .components()
                .any(|component| component.as_os_str() == "node_modules");
            (installed && file.original_path.strip_prefix(&self.project_root).ok()? == relative)
                .then(|| file.virtual_path.clone())
        })
    }

    pub(crate) fn desired_package_links(
        &self,
        workspace_alias_plan: &WorkspaceAliasPlan,
    ) -> vize_carton::FxHashMap<PathBuf, PathBuf> {
        let expected_files = self.expected_materialized_files();
        self.desired_package_links_for_files(&expected_files, workspace_alias_plan)
    }

    pub(super) fn package_links_for_files(
        &self,
        expected_files: &FxHashSet<PathBuf>,
        workspace_alias_plan: &WorkspaceAliasPlan,
    ) -> Vec<PackageNodeModulesLink> {
        let mut claims = expected_files.clone();
        claims.extend(self.workspace_alias_claims(workspace_alias_plan));
        let mut links = self.package_node_modules_links(&claims);
        links.extend(self.package_shadow_dependency_links(&claims));
        links.extend(workspace_alias_plan.links.iter().cloned());
        self.rebase_owned_workspace_alias_descendants(&mut links, &workspace_alias_plan.targets);
        links.sort_by(|left, right| {
            (&left.virtual_dir, &left.real_dir).cmp(&(&right.virtual_dir, &right.real_dir))
        });
        links.dedup();
        links
    }

    pub(super) fn desired_package_links_for_files(
        &self,
        expected_files: &FxHashSet<PathBuf>,
        workspace_alias_plan: &WorkspaceAliasPlan,
    ) -> vize_carton::FxHashMap<PathBuf, PathBuf> {
        let _canonical_paths = CanonicalPathGuard::enter();
        let links = self.package_links_for_files(expected_files, workspace_alias_plan);
        let mut desired = vize_carton::FxHashMap::default();
        for link in links {
            let real_dir = if self.authored_workspace_alias_target(&link.real_dir) {
                link.real_dir
            } else {
                canonical_path(&link.real_dir)
            };
            // A link onto its own path is never a package link: whatever
            // the mirror owns there is the directory itself.
            if real_dir == link.virtual_dir {
                continue;
            }
            desired.entry(link.virtual_dir).or_insert(real_dir);
        }
        // A resolved workspace alias is the identity authority at its exact
        // path. Ordinary package discovery can propose the same leaf, but it
        // must not displace the private target after canonicalization.
        desired.extend(workspace_alias_plan.targets.clone());
        desired
    }

    pub(super) fn preserved_prune_roots(
        &self,
        package_links: &[PackageNodeModulesLink],
    ) -> Vec<PathBuf> {
        let mut roots = Vec::with_capacity(package_links.len() + 2);
        roots.push(self.virtual_root.join("node_modules"));
        roots.push(self.virtual_root.join(super::CLI_BUILD_INFO_DIR));
        roots.extend(package_links.iter().map(|link| link.virtual_dir.clone()));
        roots
    }
}

#[cfg(test)]
mod installed_file_mirror_tests;
