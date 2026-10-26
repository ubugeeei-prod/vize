//! Bounded session cache and strong disk fingerprints for alias contexts.
#![expect(clippy::disallowed_types, reason = "shared across threads")]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use vize_carton::FxHashMap;

use super::AliasContext;

mod catalog;
mod fingerprint;
mod prebuilt;
pub(super) use fingerprint::ContextFingerprint;

const CONTEXT_CACHE_CAPACITY: usize = 8;

/// References project the whole workspace while hover projects the host
/// alone. Both revisions of one host stay cached so alternating requests do
/// not rebuild either projection.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct RevisionKey {
    pub(super) source_path: PathBuf,
    pub(super) workspace: bool,
}

impl RevisionKey {
    pub(super) fn new(source_path: PathBuf, fingerprint: &ContextFingerprint) -> Self {
        Self {
            source_path,
            workspace: fingerprint.workspace(),
        }
    }
}

#[derive(Default)]
pub(in crate::corsa_bridge) struct SessionCache {
    slots: FxHashMap<RevisionKey, CachedContext>,
    project_snapshots: FxHashMap<PathBuf, crate::batch::virtual_project::MaterializedFileSnapshot>,
    project_members: FxHashMap<PathBuf, FxHashMap<RevisionKey, ProjectMember>>,
    project_overlay_identities: FxHashMap<PathBuf, u64>,
    project_catalogs: FxHashMap<PathBuf, crate::corsa_bridge::CorsaSourceCatalog>,
    clock: u64,
    /// One revision prepared outside the native transaction.
    prebuilt: Option<prebuilt::PrebuiltContext>,
    generation: u64,
    /// The LSP owner has an active workspace watcher and invalidates this
    /// cache when any captured disk input can move.
    trust_watched_disk_inputs: bool,
}

pub(super) struct ProjectMember {
    /// Authored identities survive generated-context invalidation so their
    /// current open buffers can be rebuilt before the old revision is pruned.
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) expected_files: vize_carton::FxHashSet<PathBuf>,
    pub(super) package_links: vize_carton::FxHashMap<PathBuf, PathBuf>,
    pub(super) query_paths: Vec<PathBuf>,
    pub(super) stamps: Vec<crate::package_route::stamp::InputStamp>,
    pub(super) overlay_identity: u64,
    pub(super) requires_arbitrary_extensions: bool,
}

#[derive(Default)]
pub(super) struct ProjectUnion {
    pub(super) files: vize_carton::FxHashSet<PathBuf>,
    pub(super) package_links: FxHashMap<PathBuf, PathBuf>,
    pub(super) query_paths: Vec<PathBuf>,
    pub(super) requires_arbitrary_extensions: bool,
}

struct CachedContext {
    fingerprint: ContextFingerprint,
    context: Arc<AliasContext>,
    last_used: u64,
}

impl SessionCache {
    pub(super) fn project_sources_to_refresh(
        &self,
        virtual_root: &Path,
        overlay_identity: u64,
    ) -> Vec<PathBuf> {
        if self.project_revision_is_current(virtual_root, overlay_identity) {
            return Vec::new();
        }
        self.project_source_paths(virtual_root)
    }

    pub(super) fn project_source_paths(&self, virtual_root: &Path) -> Vec<PathBuf> {
        let mut paths = self
            .project_members
            .get(virtual_root)
            .into_iter()
            .flat_map(|members| members.values())
            .flat_map(|member| member.source_paths.iter().cloned())
            .collect::<Vec<_>>();
        paths.sort();
        paths.dedup();
        paths
    }

    pub(in crate::corsa_bridge) fn clear(&mut self) {
        self.slots.clear();
        self.project_snapshots.clear();
        self.project_members.clear();
        self.project_overlay_identities.clear();
        self.project_catalogs.clear();
        self.clock = 0;
        self.reset_prebuilt();
    }

    pub(in crate::corsa_bridge) fn trust_watched_disk_inputs(&mut self) {
        self.trust_watched_disk_inputs = true;
    }

    /// Retire compiled contexts while keeping the materialized snapshot that
    /// the next rebuild needs to remove stale mirror files precisely.
    pub(in crate::corsa_bridge) fn invalidate_disk_inputs(&mut self) {
        self.slots.clear();
        self.project_overlay_identities.clear();
        self.reset_prebuilt();
    }

    pub(super) fn get(
        &mut self,
        source_path: &Path,
        fingerprint: &ContextFingerprint,
    ) -> Option<Arc<AliasContext>> {
        // Standalone callers retain live filesystem validation. An LSP session
        // with a registered watcher invalidates slots at the notification
        // boundary and pays no lstat/read cost on unchanged warm requests.
        // One validation reads each input once across the slot and members.
        let key = RevisionKey::new(source_path.to_path_buf(), fingerprint);
        let mut observed = crate::package_route::stamp::InputStampValidation::default();
        let valid = self.slots.get(&key).is_some_and(|cached| {
            cached.fingerprint == *fingerprint
                && (self.trust_watched_disk_inputs
                    || (cached.fingerprint.stamps_still_valid(&mut observed)
                        && cached.context.mirror.as_ref().is_none_or(|mirror| {
                            !self.project_members.contains_key(mirror.virtual_root())
                                || self.project_revision_is_current_with_cache(
                                    mirror.virtual_root(),
                                    fingerprint.overlay_identity(),
                                    &mut observed,
                                )
                        })))
        });
        if !valid {
            self.slots.remove(&key);
            return None;
        }
        self.clock = self.clock.wrapping_add(1);
        let cached = self.slots.get_mut(&key)?;
        cached.last_used = self.clock;
        Some(Arc::clone(&cached.context))
    }

    pub(super) fn insert(
        &mut self,
        source_path: PathBuf,
        fingerprint: ContextFingerprint,
        context: Arc<AliasContext>,
    ) {
        let key = RevisionKey::new(source_path, &fingerprint);
        if self.slots.len() >= CONTEXT_CACHE_CAPACITY && !self.slots.contains_key(&key) {
            let lru = self
                .slots
                .iter()
                .min_by_key(|(_, cached)| cached.last_used)
                .map(|(path, _)| path.clone());
            if let Some(lru) = lru {
                // Compiled context eviction must not remove live project
                // members while a references query opens the workspace.
                self.slots.remove(&lru);
            }
        }
        self.clock = self.clock.wrapping_add(1);
        self.slots.insert(
            key,
            CachedContext {
                fingerprint,
                context,
                last_used: self.clock,
            },
        );
        let active_sources = self
            .slots
            .keys()
            .map(|key| vize_carton::path::canonicalize_non_verbatim(&key.source_path))
            .collect::<vize_carton::FxHashSet<_>>();
        self.project_members.retain(|_, members| {
            members.keys().any(|key| {
                active_sources.contains(&vize_carton::path::canonicalize_non_verbatim(
                    &key.source_path,
                ))
            })
        });
        self.prune_project_state();
    }

    /// The other revision of the current host stays in the union, so its
    /// cached context keeps every materialized file it queries.
    pub(super) fn project_union_snapshot(
        &mut self,
        virtual_root: &Path,
        current: &RevisionKey,
        overlay_identity: u64,
    ) -> ProjectUnion {
        if let Some(members) = self.project_members.get_mut(virtual_root) {
            members.retain(|_, member| {
                member.overlay_identity == overlay_identity
                    && member
                        .stamps
                        .iter()
                        .all(crate::package_route::stamp::InputStamp::is_current)
            });
        }
        let mut union = ProjectUnion::default();
        if let Some(members) = self.project_members.get(virtual_root) {
            for (key, member) in members {
                if key == current {
                    continue;
                }
                union.files.extend(member.expected_files.iter().cloned());
                union.requires_arbitrary_extensions |= member.requires_arbitrary_extensions;
                for (path, target) in &member.package_links {
                    union
                        .package_links
                        .entry(path.clone())
                        .and_modify(|current| {
                            if target < current {
                                current.clone_from(target);
                            }
                        })
                        .or_insert_with(|| target.clone());
                }
                union.query_paths.extend(member.query_paths.iter().cloned());
            }
        }
        union.query_paths.sort();
        union.query_paths.dedup();
        union
    }

    pub(super) fn record_project_member(
        &mut self,
        virtual_root: PathBuf,
        key: RevisionKey,
        member: ProjectMember,
    ) {
        for members in self.project_members.values_mut() {
            members.remove(&key);
        }
        self.project_overlay_identities
            .insert(virtual_root.clone(), member.overlay_identity);
        self.project_members
            .entry(virtual_root)
            .or_default()
            .insert(key, member);
        self.prune_project_state();
    }

    pub(super) fn forget_sources(&mut self, source_paths: &[PathBuf]) {
        let forgotten = source_paths
            .iter()
            .flat_map(|path| {
                [
                    path.clone(),
                    vize_carton::path::canonicalize_non_verbatim(path),
                ]
            })
            .collect::<vize_carton::FxHashSet<_>>();
        self.slots
            .retain(|key, _| !forgotten.contains(&key.source_path));
        for members in self.project_members.values_mut() {
            members.retain(|key, _| !forgotten.contains(&key.source_path));
        }
        self.project_overlay_identities.clear();
        self.project_members
            .retain(|_, members| !members.is_empty());
        self.reset_prebuilt();
    }

    fn prune_project_state(&mut self) {
        self.project_members
            .retain(|_, members| !members.is_empty());
        self.project_snapshots
            .retain(|root, _| self.project_members.contains_key(root));
        self.project_overlay_identities
            .retain(|root, _| self.project_members.contains_key(root));
        self.project_catalogs
            .retain(|root, _| self.project_members.contains_key(root));
    }

    pub(super) fn materialized_snapshot(
        &self,
        virtual_root: &Path,
    ) -> crate::batch::virtual_project::MaterializedFileSnapshot {
        self.project_snapshots
            .get(virtual_root)
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn set_materialized_snapshot(
        &mut self,
        virtual_root: PathBuf,
        snapshot: crate::batch::virtual_project::MaterializedFileSnapshot,
    ) {
        self.project_snapshots.insert(virtual_root, snapshot);
    }
}

pub(in crate::corsa_bridge) fn recover_lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            mutex.clear_poison();
            poisoned.into_inner()
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod scaling_tests;

#[cfg(test)]
mod revision_tests;

#[cfg(test)]
mod catalog_tests;
