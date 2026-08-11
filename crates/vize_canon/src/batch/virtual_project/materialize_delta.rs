//! Exact revisions for files owned by an editor Canon snapshot.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet, hash::hash_bytes};

use crate::batch::error::CorsaResult;
use crate::batch::materialize_fs::{ensure_dir, ensure_materialize_root, write_if_changed};

use super::{
    AUTO_IMPORT_STUBS_FILE, MODULE_AUGMENTATION_STUBS_FILE, PACKAGE_BOUNDARY_FILE,
    SHARED_HELPERS_FILE, VUE_MODULE_STUBS_FILE, VirtualProject,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct MaterializedFileDelta {
    pub(crate) changed: Vec<PathBuf>,
    pub(crate) created: Vec<PathBuf>,
    pub(crate) deleted: Vec<PathBuf>,
    pub(crate) topology_changed: bool,
}

impl MaterializedFileDelta {
    pub(crate) fn is_empty(&self) -> bool {
        self.changed.is_empty() && self.created.is_empty() && self.deleted.is_empty()
    }

    pub(crate) fn has_topology_changes(&self) -> bool {
        self.topology_changed
            || !self.created.is_empty()
            || !self.deleted.is_empty()
            || self.changed.iter().any(|path| {
                path.file_name().is_some_and(|name| {
                    name == "tsconfig.json" || name == "jsconfig.json" || name == "package.json"
                })
            })
    }
}

#[derive(Debug, Default)]
pub(crate) struct IncrementalMaterialization {
    pub(crate) delta: MaterializedFileDelta,
    pub(crate) considered: usize,
    pub(crate) source_nodes_rebuilt: usize,
    pub(crate) dependency_nodes_reconciled: usize,
    pub(crate) shadow_bindings_rebuilt: usize,
    pub(crate) full_topology_rebuild: bool,
}

impl VirtualProject {
    pub(super) fn mark_incremental_link_topology(&mut self) {
        self.incremental_link_topology_dirty = true;
    }

    pub(super) fn mark_incremental_config_file(&mut self) {
        self.incremental_materialized_candidates
            .insert(self.virtual_root.join("tsconfig.json"));
    }

    pub(super) fn mark_incremental_stub_files(&mut self) {
        self.incremental_materialized_candidates.extend([
            self.virtual_root.join(AUTO_IMPORT_STUBS_FILE),
            self.virtual_root.join(MODULE_AUGMENTATION_STUBS_FILE),
            self.virtual_root.join(VUE_MODULE_STUBS_FILE),
            self.virtual_root.join(SHARED_HELPERS_FILE),
        ]);
    }

    pub(crate) fn discard_incremental_materialization(&mut self) {
        self.incremental_materialized_candidates.clear();
        self.incremental_source_nodes_rebuilt = 0;
        self.incremental_dependency_nodes_reconciled = 0;
        self.incremental_shadow_bindings_rebuilt = 0;
        self.incremental_package_link_scopes.clear();
        self.incremental_link_topology_dirty = false;
    }

    /// Write and hash only artifacts owned by the persistent source/route
    /// patch. The cold path remains the full materializer and GC recovery.
    pub(crate) fn materialize_incremental_delta(
        &mut self,
    ) -> CorsaResult<IncrementalMaterialization> {
        let mut candidates = std::mem::take(&mut self.incremental_materialized_candidates);
        let full_topology_rebuild = self.incremental_link_topology_dirty;
        let local_link_patch = (!full_topology_rebuild
            && !self.incremental_package_link_scopes.is_empty())
        .then(|| self.prepare_incremental_package_link_patch());
        let desired_links = full_topology_rebuild.then(|| self.desired_package_links());
        let package_links_changed = desired_links.as_ref().map_or_else(
            || {
                local_link_patch
                    .as_ref()
                    .is_some_and(|patch| patch.previous != patch.desired)
            },
            |desired| desired != &self.materialized_package_links,
        );
        if let Some(desired) = desired_links.as_ref() {
            candidates.extend(self.materialized_package_links.keys().cloned());
            candidates.extend(desired.keys().cloned());
        }
        if let Some(patch) = local_link_patch.as_ref() {
            candidates.extend(patch.candidates.iter().cloned());
        }
        let considered = candidates.len();
        let before = candidates
            .iter()
            .map(|path| Ok((path.clone(), revision_if_file(path)?)))
            .collect::<CorsaResult<FxHashMap<_, _>>>()?;

        ensure_materialize_root(&self.virtual_root)?;
        if let Some(desired) = desired_links.as_ref() {
            remove_stale_package_links(&self.materialized_package_links, desired)?;
        } else if let Some(patch) = local_link_patch.as_ref() {
            remove_stale_package_links(&patch.previous, &patch.desired)?;
        }
        if candidates.contains(&self.virtual_root.join(PACKAGE_BOUNDARY_FILE)) {
            self.write_package_boundary()?;
        }
        if candidates.contains(&self.virtual_root.join(AUTO_IMPORT_STUBS_FILE)) {
            self.write_auto_import_stubs()?;
        }
        if candidates.contains(&self.virtual_root.join(MODULE_AUGMENTATION_STUBS_FILE)) {
            self.write_module_augmentation_stubs()?;
        }
        if candidates.contains(&self.virtual_root.join(VUE_MODULE_STUBS_FILE)) {
            self.write_vue_module_stubs()?;
        }
        if candidates.contains(&self.virtual_root.join(SHARED_HELPERS_FILE))
            && self.uses_shared_helpers()
        {
            self.write_shared_helpers()?;
        }
        if candidates.contains(&self.virtual_root.join("tsconfig.json")) {
            self.write_tsconfig_file(&self.virtual_root.join("tsconfig.json"), None, false)?;
        }

        for path in &candidates {
            // Package links are materialized as directories/symlinks after the
            // ordinary file pass.  Treating an unchanged desired link as a
            // stale file here would unlink and recreate it on every topology
            // reconciliation, invalidating native filesystem caches even when
            // the target identity did not change.
            if desired_links
                .as_ref()
                .is_some_and(|links| links.contains_key(path))
                || local_link_patch
                    .as_ref()
                    .is_some_and(|patch| patch.desired.contains_key(path))
            {
                continue;
            }
            if let Some(parent) = path.parent() {
                ensure_dir(parent)?;
            }
            if let Some(file) = self.virtual_files.get(path) {
                write_if_changed(path, file.content.as_bytes())?;
            } else if let Some(original) = self.passthrough_files.get(path) {
                write_if_changed(path, &std::fs::read(original)?)?;
            } else if let Some(canonical) = self.package_shadow_files.get(path) {
                let content = self.package_shadow_content(path, canonical)?;
                write_if_changed(path, content.as_bytes())?;
            } else if let Some(original) = self.package_shadow_manifests.get(path) {
                write_if_changed(path, &std::fs::read(original)?)?;
            } else if !self.is_current_generated_path(path) {
                remove_file_if_present(path)?;
            }
        }

        if let Some(desired) = desired_links.as_ref() {
            materialize_package_links(desired)?;
            self.materialized_package_links = desired.clone();
            self.rebuild_materialized_package_link_owners();
            self.incremental_link_topology_dirty = false;
        } else if let Some(patch) = local_link_patch {
            materialize_package_links(&patch.desired)?;
            self.commit_incremental_package_link_patch(patch);
        }

        let mut delta = MaterializedFileDelta {
            topology_changed: package_links_changed,
            ..Default::default()
        };
        for path in candidates {
            let previous = before.get(&path).copied().flatten();
            let current = revision_if_file(&path)?;
            match (previous, current) {
                (None, Some(_)) => delta.created.push(path),
                (Some(left), Some(right)) if left != right => delta.changed.push(path),
                (Some(_), None) => delta.deleted.push(path),
                _ => {}
            }
        }
        delta.changed.sort();
        delta.created.sort();
        delta.deleted.sort();
        Ok(IncrementalMaterialization {
            delta,
            considered,
            source_nodes_rebuilt: std::mem::take(&mut self.incremental_source_nodes_rebuilt),
            dependency_nodes_reconciled: std::mem::take(
                &mut self.incremental_dependency_nodes_reconciled,
            ),
            shadow_bindings_rebuilt: std::mem::take(&mut self.incremental_shadow_bindings_rebuilt),
            full_topology_rebuild,
        })
    }

    fn is_current_generated_path(&self, path: &Path) -> bool {
        path == self.virtual_root.join(PACKAGE_BOUNDARY_FILE)
            || path == self.virtual_root.join("tsconfig.json")
            || path == self.virtual_root.join(VUE_MODULE_STUBS_FILE)
            || (path == self.virtual_root.join(SHARED_HELPERS_FILE) && self.uses_shared_helpers())
            || (path == self.virtual_root.join(AUTO_IMPORT_STUBS_FILE)
                && self.has_global_auto_import_stubs())
            || (path == self.virtual_root.join(MODULE_AUGMENTATION_STUBS_FILE)
                && self.has_module_augmentation_stubs())
    }
}

fn revision_if_file(path: &Path) -> CorsaResult<Option<u64>> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Ok(Some(hash_path(&std::fs::read_link(path)?)))
        }
        Ok(metadata) if metadata.is_file() => Ok(Some(hash_bytes(&std::fs::read(path)?))),
        Ok(_) => Ok(None),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn hash_path(path: &Path) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::hash::DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}

fn remove_stale_package_links(
    previous: &FxHashMap<PathBuf, PathBuf>,
    desired: &FxHashMap<PathBuf, PathBuf>,
) -> CorsaResult<()> {
    for (path, previous_target) in previous {
        if desired.get(path) != Some(previous_target) {
            crate::batch::materialize_fs::remove_path(path)?;
        }
    }
    Ok(())
}

fn materialize_package_links(desired: &FxHashMap<PathBuf, PathBuf>) -> CorsaResult<()> {
    for (virtual_dir, real_dir) in desired {
        crate::batch::runtime_deps::symlink_package_dir(real_dir, virtual_dir)?;
    }
    Ok(())
}

fn remove_file_if_present(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct MaterializedFileSnapshot {
    revisions: FxHashMap<PathBuf, u64>,
    package_links: FxHashMap<PathBuf, PathBuf>,
}

impl MaterializedFileSnapshot {
    #[cfg(test)]
    pub(crate) fn capture(paths: &FxHashSet<PathBuf>) -> CorsaResult<Self> {
        Self::capture_with_links(paths, &FxHashMap::default())
    }

    pub(crate) fn capture_with_links(
        paths: &FxHashSet<PathBuf>,
        package_links: &FxHashMap<PathBuf, PathBuf>,
    ) -> CorsaResult<Self> {
        let mut revisions = FxHashMap::default();
        revisions.reserve(paths.len());
        for path in paths {
            revisions.insert(path.clone(), revision(path)?);
        }
        Ok(Self {
            revisions,
            package_links: package_links.clone(),
        })
    }

    pub(crate) fn diff(&self, previous: &Self) -> MaterializedFileDelta {
        let mut delta = MaterializedFileDelta::default();
        for (path, revision) in &self.revisions {
            match previous.revisions.get(path) {
                None => delta.created.push(path.clone()),
                Some(previous) if previous != revision => delta.changed.push(path.clone()),
                Some(_) => {}
            }
        }
        for path in previous.revisions.keys() {
            if !self.revisions.contains_key(path) {
                delta.deleted.push(path.clone());
            }
        }
        for (path, target) in &self.package_links {
            match previous.package_links.get(path) {
                None => delta.created.push(path.clone()),
                Some(previous) if previous != target => delta.changed.push(path.clone()),
                Some(_) => {}
            }
        }
        for path in previous.package_links.keys() {
            if !self.package_links.contains_key(path) {
                delta.deleted.push(path.clone());
            }
        }
        delta.topology_changed = self.package_links != previous.package_links;
        delta.changed.sort();
        delta.changed.dedup();
        delta.created.sort();
        delta.created.dedup();
        delta.deleted.sort();
        delta.deleted.dedup();
        delta
    }
}

fn revision(path: &Path) -> CorsaResult<u64> {
    Ok(hash_bytes(&std::fs::read(path)?))
}

#[cfg(test)]
#[path = "materialize_delta/tests.rs"]
mod tests;
