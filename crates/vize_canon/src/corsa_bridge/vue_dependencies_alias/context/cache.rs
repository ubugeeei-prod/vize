//! Bounded session cache and strong disk fingerprints for alias contexts.
#![allow(clippy::disallowed_types)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use vize_carton::FxHashMap;

use super::AliasContext;

const CONTEXT_CACHE_CAPACITY: usize = 8;

#[derive(Default)]
pub(in crate::corsa_bridge) struct SessionCache {
    slots: FxHashMap<PathBuf, CachedContext>,
    project_snapshots: FxHashMap<PathBuf, crate::batch::virtual_project::MaterializedFileSnapshot>,
    project_members: FxHashMap<PathBuf, FxHashMap<PathBuf, ProjectMember>>,
    clock: u64,
}

pub(super) struct ProjectMember {
    pub(super) expected_files: vize_carton::FxHashSet<PathBuf>,
    pub(super) package_links: vize_carton::FxHashMap<PathBuf, PathBuf>,
    pub(super) query_path: Option<PathBuf>,
    pub(super) stamps: Vec<crate::package_route::stamp::InputStamp>,
    pub(super) overlay_identity: u64,
}

struct CachedContext {
    fingerprint: ContextFingerprint,
    context: Arc<AliasContext>,
    last_used: u64,
}

impl SessionCache {
    pub(in crate::corsa_bridge) fn clear(&mut self) {
        self.slots.clear();
        self.project_snapshots.clear();
        self.project_members.clear();
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
                self.evict_source(&lru);
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
                if let Some(query_path) = member.query_path.as_ref() {
                    query_paths.push(query_path.clone());
                }
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
        self.project_members
            .retain(|_, members| !members.is_empty());
    }

    /// Capacity eviction is the only path that can forget the last disk
    /// snapshot immediately. Invalidated or deleted members retain it until
    /// the next union build so Corsa receives exact changed/deleted deltas.
    fn evict_source(&mut self, source_path: &Path) {
        let canonical = vize_carton::path::canonicalize_non_verbatim(source_path);
        self.slots.remove(source_path);
        self.slots.remove(&canonical);
        for members in self.project_members.values_mut() {
            members.remove(source_path);
            members.remove(&canonical);
        }
        self.prune_project_state();
    }

    fn prune_project_state(&mut self) {
        self.project_members
            .retain(|_, members| !members.is_empty());
        self.project_snapshots
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

/// The import closure and disk inputs a cached editor route depends on.
#[derive(Clone)]
#[allow(clippy::disallowed_types)]
pub(super) struct ContextFingerprint {
    host_content: u64,
    overlays: u64,
    generation_options: u64,
    stamps: Vec<crate::package_route::stamp::InputStamp>,
}

impl PartialEq for ContextFingerprint {
    fn eq(&self, other: &Self) -> bool {
        self.host_content == other.host_content
            && self.overlays == other.overlays
            && self.generation_options == other.generation_options
    }
}

impl ContextFingerprint {
    #[allow(clippy::disallowed_methods)]
    pub(super) fn capture(
        source_path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
        options: crate::corsa_bridge::vue_document::CorsaVueVirtualDocumentOptions,
        virtual_ts_options: &crate::virtual_ts::VirtualTsOptions,
        project_root: Option<&Path>,
        tsconfig_path: Option<&Path>,
    ) -> Self {
        use std::hash::{Hash, Hasher};
        let mut host = std::hash::DefaultHasher::new();
        source_path.hash(&mut host);
        content.hash(&mut host);
        let mut overlay_entries: Vec<_> = overlays.iter().collect();
        overlay_entries.sort_by(|left, right| left.0.cmp(right.0));
        let mut overlay_hash = std::hash::DefaultHasher::new();
        for (path, text) in overlay_entries {
            path.hash(&mut overlay_hash);
            text.hash(&mut overlay_hash);
        }
        let generation_options =
            editor_namespace_identity(options, virtual_ts_options, project_root, tsconfig_path);
        Self {
            host_content: host.finish(),
            overlays: overlay_hash.finish(),
            generation_options,
            stamps: Vec::new(),
        }
    }

    pub(super) fn stamp(&mut self, context: &AliasContext) {
        let mut paths = vec![context.project_root.join("tsconfig.json")];
        if let Some(mirror) = context.mirror.as_ref() {
            paths.extend(mirror.governing_config_paths());
            // The materialized closure intentionally keeps content digests:
            // same-mtime, same-length edits must invalidate an editor session.
            paths.extend(mirror.registered_original_paths_sorted());
        }
        paths.extend(context.route_inputs.iter().cloned());
        paths.sort();
        paths.dedup();
        self.stamps = paths
            .into_iter()
            .map(crate::package_route::stamp::InputStamp::capture)
            .collect();
    }

    fn stamps_still_valid(&self) -> bool {
        self.stamps
            .iter()
            .all(crate::package_route::stamp::InputStamp::is_current)
    }

    pub(super) fn input_stamps(&self) -> Vec<crate::package_route::stamp::InputStamp> {
        self.stamps.clone()
    }

    pub(super) fn overlay_identity(&self) -> u64 {
        self.overlays
    }
}

#[allow(clippy::disallowed_methods)]
pub(super) fn editor_namespace_identity(
    options: crate::corsa_bridge::vue_document::CorsaVueVirtualDocumentOptions,
    virtual_ts_options: &crate::virtual_ts::VirtualTsOptions,
    project_root: Option<&Path>,
    tsconfig_path: Option<&Path>,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut generation_options = std::hash::DefaultHasher::new();
    options.options_api.hash(&mut generation_options);
    options.legacy_vue2.hash(&mut generation_options);
    options.dialect.hash(&mut generation_options);
    options
        .preserve_event_navigation
        .hash(&mut generation_options);
    for global in &virtual_ts_options.template_globals {
        global.name.hash(&mut generation_options);
        global.type_annotation.hash(&mut generation_options);
        global.default_value.hash(&mut generation_options);
    }
    virtual_ts_options.css_modules.hash(&mut generation_options);
    virtual_ts_options
        .auto_import_stubs
        .hash(&mut generation_options);
    virtual_ts_options
        .external_template_bindings
        .hash(&mut generation_options);
    virtual_ts_options
        .reference_paths
        .hash(&mut generation_options);
    project_root.hash(&mut generation_options);
    tsconfig_path.hash(&mut generation_options);
    generation_options.finish()
}

#[cfg(test)]
#[path = "cache/tests.rs"]
mod tests;
