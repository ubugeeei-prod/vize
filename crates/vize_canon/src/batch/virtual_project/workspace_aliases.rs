//! Owned links to existing authored workspace shadows, never fresh type copies.

mod install_roots;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use vize_carton::{FxHashMap, FxHashSet};

use super::VirtualProject;
use super::package_node_modules::PackageNodeModulesLink;
use super::package_shadow::PackageShadowTopology;
use crate::batch::error::CorsaResult;

const PACKAGE_LINK_RECEIPT_VERSION: u32 = 1;
const PACKAGE_LINK_RECEIPT_FILE: &str = ".vize-buildinfo/package-links.json";

#[derive(Debug, Deserialize, Serialize)]
struct PackageLinkReceipt {
    version: u32,
    links: Vec<PackageLinkReceiptEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PackageLinkReceiptEntry {
    path: PathBuf,
    target: PathBuf,
}

fn is_normalized_descendant(path: &Path, root: &Path) -> bool {
    path.strip_prefix(root).is_ok_and(|relative| {
        let mut components = relative.components();
        components.next().is_some_and(|component| {
            matches!(component, std::path::Component::Normal(_))
                && components.all(|component| matches!(component, std::path::Component::Normal(_)))
        })
    })
}

impl VirtualProject {
    fn package_link_receipt_path(&self) -> PathBuf {
        self.virtual_root.join(PACKAGE_LINK_RECEIPT_FILE)
    }

    fn committed_package_link_receipt(&self) -> FxHashMap<PathBuf, PathBuf> {
        let path = self.package_link_receipt_path();
        let Ok(metadata) = std::fs::symlink_metadata(&path) else {
            return FxHashMap::default();
        };
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return FxHashMap::default();
        }
        let Ok(content) = std::fs::read(&path) else {
            return FxHashMap::default();
        };
        let Ok(receipt) = serde_json::from_slice::<PackageLinkReceipt>(&content) else {
            return FxHashMap::default();
        };
        if receipt.version != PACKAGE_LINK_RECEIPT_VERSION {
            return FxHashMap::default();
        }
        let mut links = FxHashMap::default();
        for entry in receipt.links {
            if !is_normalized_descendant(&entry.path, &self.virtual_root)
                || !entry.target.is_absolute()
                || links.insert(entry.path, entry.target).is_some()
            {
                return FxHashMap::default();
            }
        }
        links
    }

    pub(super) fn write_package_link_receipt(
        &self,
        links: &FxHashMap<PathBuf, PathBuf>,
    ) -> CorsaResult<()> {
        let mut entries = links
            .iter()
            .filter_map(|(path, target)| {
                let linked = std::fs::read_link(path).ok()?;
                let linked = vize_carton::path::normalize_windows_verbatim_path(linked);
                (linked == *target).then(|| PackageLinkReceiptEntry {
                    path: path.clone(),
                    target: target.clone(),
                })
            })
            .collect::<Vec<_>>();
        entries
            .sort_by(|left, right| (&left.path, &left.target).cmp(&(&right.path, &right.target)));
        let content = serde_json::to_vec(&PackageLinkReceipt {
            version: PACKAGE_LINK_RECEIPT_VERSION,
            links: entries,
        })?;
        let path = self.package_link_receipt_path();
        if let Some(parent) = path.parent() {
            crate::batch::materialize_fs::ensure_dir(parent)?;
        }
        crate::batch::materialize_fs::write_if_changed(&path, &content)?;
        Ok(())
    }

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
        let unique_targets = targets
            .into_iter()
            .filter_map(|(alias, targets)| {
                // Conflicting owners must not select a native identity by ordering.
                if targets.len() != 1 {
                    return None;
                }
                targets.into_iter().next().map(|target| (alias, target))
            })
            .collect::<FxHashMap<_, _>>();
        unique_targets
            .keys()
            .filter_map(|alias| {
                let mut real_dir = unique_targets.get(alias)?.clone();
                let mut visited = FxHashSet::from_iter([alias.clone()]);
                // A package-private shadow can reuse a visible workspace alias.
                // Rebase through every such link so the final filesystem link
                // points at owned shadow bytes instead of another symlink.
                while let Some((bridge, target)) = real_dir
                    .ancestors()
                    .find_map(|candidate| unique_targets.get_key_value(candidate))
                {
                    if !visited.insert(bridge.clone()) {
                        return None;
                    }
                    let suffix = real_dir.strip_prefix(bridge).ok()?;
                    real_dir = target.join(suffix);
                }
                Some(PackageNodeModulesLink {
                    virtual_dir: alias.clone(),
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
        let mut link_files = self.expected_materialized_files();
        link_files.extend(extra_owned_files.iter().cloned());
        let links = self.desired_package_links_for_files(&link_files);
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
        for alias in self.workspace_alias_links() {
            if links.get(&alias.virtual_dir) != Some(&alias.real_dir) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Workspace package link plan omits an owned alias",
                )
                .into());
            }
        }
        let committed_links = self.committed_package_link_receipt();
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
                    // A current cold plan also proves an external leaf link:
                    // its authored target and exact cache path were derived
                    // again without trusting the existing filesystem entry.
                    let planned = links.get(&path) == Some(&target);
                    if path == self.virtual_root
                        || (!install_roots.contains(&path)
                            && !planned
                            && committed_links.get(&path) != Some(&target)
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
