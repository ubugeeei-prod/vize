//! Owned links to existing authored workspace shadows, never fresh type copies.

mod install_roots;

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet};

use super::VirtualProject;
use super::package_node_modules::PackageNodeModulesLink;
use super::package_shadow::PackageShadowTopology;
use crate::batch::error::CorsaResult;

impl VirtualProject {
    pub(super) fn mark_package_shadow_dependents(
        &mut self,
        keys: impl IntoIterator<Item = crate::package_route::PackageRouteKey>,
    ) {
        let mut pending = keys.into_iter().collect::<Vec<_>>();
        let mut visited = FxHashSet::default();
        while let Some(key) = pending.pop() {
            if !visited.insert(key.clone()) {
                continue;
            }
            self.package_shadow_dirty_keys.insert(key.clone());
            for ancestor in key.importer_path.ancestors() {
                if let Some(owners) = self.package_route_roots.get(ancestor) {
                    pending.extend(owners.iter().cloned());
                }
            }
        }
    }

    pub(super) fn workspace_alias_links(&self) -> Vec<PackageNodeModulesLink> {
        let mut targets = FxHashMap::<_, FxHashSet<_>>::default();
        for topology in self.package_shadow_artifacts.values() {
            for (alias, target) in &topology.aliases {
                targets
                    .entry(alias.clone())
                    .or_default()
                    .insert(target.clone());
            }
        }
        targets
            .into_iter()
            .filter_map(|(alias, targets)| {
                // Conflicting owners must not select a native identity by ordering.
                if targets.len() != 1 {
                    return None;
                }
                targets
                    .into_iter()
                    .next()
                    .map(|real_dir| PackageNodeModulesLink {
                        virtual_dir: alias,
                        real_dir,
                    })
            })
            .collect()
    }

    pub(super) fn workspace_alias_claims(&self) -> FxHashSet<PathBuf> {
        self.package_shadow_artifacts
            .values()
            .flat_map(|topology| {
                topology
                    .aliases
                    .keys()
                    .map(|alias| alias.join("package.json"))
            })
            .collect()
    }

    pub(super) fn install_workspace_alias_scopes(
        &mut self,
        topology: &PackageShadowTopology,
        scopes: &mut Vec<(PathBuf, PathBuf)>,
    ) {
        for (alias, target) in &topology.aliases {
            if !self.package_link_scope_targets.contains_key(alias) {
                // This is solely a link-selection claim. It is never a file
                // candidate/revision: writing through it could change a target.
                self.track_materialized_link_path(&alias.join("package.json"));
            }
            scopes.push((alias.clone(), target.clone()));
        }
    }

    pub(super) fn remove_workspace_alias_claims(&mut self, aliases: &FxHashMap<PathBuf, PathBuf>) {
        for alias in aliases.keys() {
            if !self.package_link_scope_targets.contains_key(alias) {
                self.untrack_materialized_link_path(&alias.join("package.json"));
            }
        }
    }

    pub(super) fn authored_workspace_alias_target(&self, target: &Path) -> bool {
        target.starts_with(&self.virtual_root)
            && self
                .package_shadow_manifests
                .contains_key(&target.join("package.json"))
    }

    pub(super) fn direct_workspace_alias_for_scope(
        &self,
        scope: &Path,
    ) -> Option<FxHashMap<PathBuf, PathBuf>> {
        let targets = self.package_link_scope_targets.get(scope)?;
        if !targets
            .keys()
            .any(|target| self.authored_workspace_alias_target(target))
        {
            return None;
        }
        if targets.len() != 1 {
            return Some(FxHashMap::default());
        }
        let target = targets.keys().next()?;
        Some(FxHashMap::from_iter([(
            scope.to_path_buf(),
            target.clone(),
        )]))
    }

    pub(super) fn validate_workspace_alias_targets(
        &self,
        extra_owned_files: &FxHashSet<PathBuf>,
    ) -> CorsaResult<Vec<PathBuf>> {
        let mut targets = FxHashMap::default();
        for topology in self.package_shadow_artifacts.values() {
            for (alias, target) in &topology.aliases {
                if !self.authored_workspace_alias_target(target) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Workspace package alias has no owned target manifest",
                    )
                    .into());
                }
                if targets
                    .insert(alias, target)
                    .is_some_and(|old| old != target)
                {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Conflicting workspace package alias targets",
                    )
                    .into());
                }
            }
        }
        let links = self
            .workspace_alias_links()
            .into_iter()
            .map(|link| (link.virtual_dir, link.real_dir))
            .collect();
        self.validate_workspace_alias_links(&links, extra_owned_files)
    }

    pub(super) fn preserved_workspace_alias_claims(
        &self,
        links: &FxHashMap<PathBuf, PathBuf>,
        files: &FxHashSet<PathBuf>,
    ) -> CorsaResult<FxHashSet<PathBuf>> {
        let mut claims = FxHashSet::default();
        for (alias, target) in links {
            if !target.starts_with(&self.virtual_root) {
                continue;
            }
            if !files.contains(&target.join("package.json")) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Preserved workspace alias has no owned manifest",
                )
                .into());
            }
            claims.insert(alias.join("package.json"));
        }
        Ok(claims)
    }

    pub(super) fn validate_workspace_alias_links(
        &self,
        links: &FxHashMap<PathBuf, PathBuf>,
        extra_owned_files: &FxHashSet<PathBuf>,
    ) -> CorsaResult<Vec<PathBuf>> {
        let mut checked = self.workspace_owned_parent_dirs(extra_owned_files);
        let install_roots = self.workspace_install_roots()?;
        for root in &install_roots {
            if links.keys().any(|path| root.starts_with(path)) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Owned workspace installation overlaps a package link",
                )
                .into());
            }
            checked.extend(
                root.ancestors()
                    .filter(|path| path.starts_with(&self.virtual_root))
                    .map(Path::to_path_buf),
            );
        }
        for (alias, target) in links {
            if !target.starts_with(&self.virtual_root) {
                continue;
            }
            if !alias.starts_with(&self.virtual_root)
                || links.keys().any(|path| {
                    path != alias
                        && (alias.starts_with(path)
                            || path.starts_with(alias)
                            || target.starts_with(path))
                })
            {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Workspace package alias overlaps another package link",
                )
                .into());
            }
            checked.extend(
                target
                    .ancestors()
                    .chain(alias.parent().into_iter().flat_map(Path::ancestors))
                    .filter(|path| path.starts_with(&self.virtual_root))
                    .map(Path::to_path_buf),
            );
        }
        let mut directories = checked.into_iter().collect::<Vec<_>>();
        directories.sort_by(|left, right| {
            left.components()
                .count()
                .cmp(&right.components().count())
                .then_with(|| left.cmp(right))
        });
        let mut retire = Vec::<PathBuf>::new();
        for path in directories {
            // Descendants of a proved cache link disappear with that link;
            // never inspect their metadata through its external endpoint.
            if retire.iter().any(|parent| path.starts_with(parent)) {
                continue;
            }
            match std::fs::symlink_metadata(&path) {
                Ok(metadata)
                    if metadata.file_type().is_symlink() || std::fs::read_link(&path).is_ok() =>
                {
                    // A selected, owner-pinned ordinary install may replace its
                    // FINAL cache link. Other ancestors/whole-parent links still
                    // require the exact previously committed target receipt.
                    // Root-first validation never unlinks through a raw parent.
                    let target = std::fs::read_link(&path)?;
                    let target = vize_carton::path::normalize_windows_verbatim_path(target);
                    if path == self.virtual_root
                        || (!install_roots.contains(&path)
                            && self.materialized_package_links.get(&path) != Some(&target))
                    {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Workspace package alias requires owned directories",
                        )
                        .into());
                    }
                    retire.push(path);
                }
                Ok(metadata) if !metadata.is_dir() => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Workspace package alias requires owned directories",
                    )
                    .into());
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(retire)
    }

    pub(super) fn retire_workspace_cache_links(&self, paths: Vec<PathBuf>) -> CorsaResult<()> {
        // All identity, overlap and independent ancestry proofs precede this.
        for path in paths {
            crate::batch::materialize_fs::remove_path(&path)?;
        }
        Ok(())
    }

    pub(super) fn ensure_workspace_alias_targets(&self) -> CorsaResult<()> {
        let retire = self.validate_workspace_alias_targets(&FxHashSet::default())?;
        self.retire_workspace_cache_links(retire)?;
        for link in self.workspace_alias_links() {
            // Windows decides symlink_dir/file from target existence. The
            // planned authored root must exist before the first link is made.
            if self.authored_workspace_alias_target(&link.real_dir) {
                crate::batch::materialize_fs::ensure_dir(&link.real_dir)?;
            }
        }
        Ok(())
    }
}
