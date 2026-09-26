//! Bounded session cache and strong disk fingerprints for alias contexts.
#![expect(clippy::disallowed_types, reason = "shared across threads")]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use vize_carton::FxHashMap;

use super::AliasContext;

mod fingerprint;
pub(super) use fingerprint::ContextFingerprint;

const CONTEXT_CACHE_CAPACITY: usize = 8;

#[derive(Default)]
pub(in crate::corsa_bridge) struct SessionCache {
    slots: FxHashMap<PathBuf, CachedContext>,
    project_snapshots: FxHashMap<PathBuf, crate::batch::virtual_project::MaterializedFileSnapshot>,
    project_members: FxHashMap<PathBuf, FxHashMap<PathBuf, ProjectMember>>,
    project_overlay_identities: FxHashMap<PathBuf, u64>,
    clock: u64,
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
        if self.project_overlay_identities.get(virtual_root) == Some(&overlay_identity)
            && self
                .project_members
                .get(virtual_root)
                .is_none_or(|members| {
                    members.values().all(|member| {
                        member
                            .stamps
                            .iter()
                            .all(crate::package_route::stamp::InputStamp::is_current)
                    })
                })
        {
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
        self.clock = 0;
    }

    pub(super) fn get(
        &mut self,
        source_path: &Path,
        fingerprint: &ContextFingerprint,
    ) -> Option<Arc<AliasContext>> {
        let valid = self.slots.get(source_path).is_some_and(|cached| {
            cached.fingerprint == *fingerprint && cached.fingerprint.stamps_still_valid()
        });
        if !valid {
            self.slots.remove(source_path);
            return None;
        }
        self.clock = self.clock.wrapping_add(1);
        let cached = self.slots.get_mut(source_path)?;
        cached.last_used = self.clock;
        Some(Arc::clone(&cached.context))
    }

    pub(super) fn insert(
        &mut self,
        source_path: PathBuf,
        fingerprint: ContextFingerprint,
        context: Arc<AliasContext>,
    ) {
        if self.slots.len() >= CONTEXT_CACHE_CAPACITY && !self.slots.contains_key(&source_path) {
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
            source_path,
            CachedContext {
                fingerprint,
                context,
                last_used: self.clock,
            },
        );
        let active_sources = self
            .slots
            .keys()
            .map(|path| vize_carton::path::canonicalize_non_verbatim(path))
            .collect::<vize_carton::FxHashSet<_>>();
        self.project_members.retain(|_, members| {
            members.keys().any(|path| {
                active_sources.contains(&vize_carton::path::canonicalize_non_verbatim(path))
            })
        });
        self.prune_project_state();
    }

    pub(super) fn project_union_snapshot(
        &mut self,
        virtual_root: &Path,
        current_source: &Path,
        overlay_identity: u64,
    ) -> (
        vize_carton::FxHashSet<PathBuf>,
        vize_carton::FxHashMap<PathBuf, PathBuf>,
        Vec<PathBuf>,
    ) {
        if let Some(members) = self.project_members.get_mut(virtual_root) {
            members.retain(|_, member| {
                member.overlay_identity == overlay_identity
                    && member
                        .stamps
                        .iter()
                        .all(crate::package_route::stamp::InputStamp::is_current)
            });
        }
        let mut files = vize_carton::FxHashSet::default();
        let mut package_links: vize_carton::FxHashMap<PathBuf, PathBuf> =
            vize_carton::FxHashMap::default();
        let mut query_paths = Vec::new();
        if let Some(members) = self.project_members.get(virtual_root) {
            for (source_path, member) in members {
                if source_path == current_source {
                    continue;
                }
                files.extend(member.expected_files.iter().cloned());
                for (path, target) in &member.package_links {
                    package_links
                        .entry(path.clone())
                        .and_modify(|current| {
                            if target < current {
                                current.clone_from(target);
                            }
                        })
                        .or_insert_with(|| target.clone());
                }
                query_paths.extend(member.query_paths.iter().cloned());
            }
        }
        query_paths.sort();
        query_paths.dedup();
        (files, package_links, query_paths)
    }

    pub(super) fn record_project_member(
        &mut self,
        virtual_root: PathBuf,
        source_path: PathBuf,
        member: ProjectMember,
    ) {
        for members in self.project_members.values_mut() {
            members.remove(&source_path);
        }
        self.project_overlay_identities
            .insert(virtual_root.clone(), member.overlay_identity);
        self.project_members
            .entry(virtual_root)
            .or_default()
            .insert(source_path, member);
        self.prune_project_state();
    }

    pub(super) fn forget_sources(&mut self, source_paths: &[PathBuf]) {
        for source_path in source_paths {
            let canonical = vize_carton::path::canonicalize_non_verbatim(source_path);
            self.slots.remove(source_path);
            self.slots.remove(&canonical);
            for members in self.project_members.values_mut() {
                members.remove(source_path);
                members.remove(&canonical);
            }
        }
        self.project_overlay_identities.clear();
        self.project_members
            .retain(|_, members| !members.is_empty());
    }

    fn prune_project_state(&mut self) {
        self.project_members
            .retain(|_, members| !members.is_empty());
        self.project_snapshots
            .retain(|root, _| self.project_members.contains_key(root));
        self.project_overlay_identities
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
