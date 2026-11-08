//! Exact host, overlay and disk identities for cached project projections.

use super::super::{AliasContext, namespace::editor_namespace_identity};
use std::path::{Path, PathBuf};
use vize_carton::FxHashMap;

use crate::batch::ModuleSpecifierFactsCache;
use crate::corsa_bridge::vue_document::{
    OverlayFingerprint, OverlaySources, PersistentOverlayMap, overlay_snapshot,
};

#[derive(Clone)]
enum OverlayInputs {
    Complete(std::sync::Arc<PersistentOverlayMap>),
    Compact(std::sync::Arc<FxHashMap<PathBuf, u64>>),
}

impl OverlayInputs {
    fn get(&self, path: &Path) -> Option<u64> {
        match self {
            Self::Complete(inputs) => inputs.content_identity(path),
            Self::Compact(inputs) => inputs.get(path).copied(),
        }
    }

    fn contains_key(&self, path: &Path) -> bool {
        self.get(path).is_some()
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        match self {
            Self::Complete(inputs) => inputs.len(),
            Self::Compact(inputs) => inputs.len(),
        }
    }

    #[cfg(test)]
    fn shares_storage_with(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Complete(left), Self::Complete(right)) => std::sync::Arc::ptr_eq(left, right),
            (Self::Compact(left), Self::Compact(right)) => std::sync::Arc::ptr_eq(left, right),
            _ => false,
        }
    }
}

/// The import closure and disk inputs a cached editor route depends on.
#[derive(Clone)]
pub(crate) struct ContextFingerprint {
    host_content: u64,
    host_path: PathBuf,
    host_graph: Option<u64>,
    /// Requested roots, independently of the open-buffer set.
    dependencies: u64,
    overlays: u64,
    /// Requests capture every overlay; stored revisions retain only their
    /// registered sources and resolution probes, including failed probes.
    overlay_inputs: OverlayInputs,
    generation_options: u64,
    /// Requested workspace sources are part of this revision.
    workspace: bool,
    stamps: Vec<crate::package_route::stamp::InputStamp>,
}

impl PartialEq for ContextFingerprint {
    fn eq(&self, other: &Self) -> bool {
        self.host_content == other.host_content
            && self.overlays == other.overlays
            && self.revises(other)
    }
}

impl ContextFingerprint {
    #[cfg(test)]
    pub(crate) fn capture(
        source_path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
        options: crate::corsa_bridge::vue_document::CorsaVueVirtualDocumentOptions,
        virtual_ts_options: &crate::virtual_ts::VirtualTsOptions,
        project_root: Option<&Path>,
        tsconfig_path: Option<&Path>,
    ) -> Self {
        Self::capture_with_overlay(
            source_path,
            content,
            overlays,
            None,
            options,
            virtual_ts_options,
            project_root,
            tsconfig_path,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the fingerprint binds every independent revision input at one boundary"
    )]
    pub(crate) fn capture_with_overlay(
        source_path: &Path,
        content: &str,
        overlays: &dyn OverlaySources,
        overlay_fingerprint: Option<&OverlayFingerprint>,
        options: crate::corsa_bridge::vue_document::CorsaVueVirtualDocumentOptions,
        virtual_ts_options: &crate::virtual_ts::VirtualTsOptions,
        project_root: Option<&Path>,
        tsconfig_path: Option<&Path>,
    ) -> Self {
        use std::hash::{Hash, Hasher};
        let host_path = vize_carton::path::canonicalize_non_verbatim(source_path);
        let mut host = vize_l0::micro_default_hasher!("canon.context.host");
        source_path.hash(&mut host);
        match overlay_fingerprint
            .and_then(|fingerprint| fingerprint.inputs.content_identity(&host_path))
        {
            Some(identity) => identity.hash(&mut host),
            None => content.hash(&mut host),
        }
        let mut dependencies = vize_l0::micro_default_hasher!("canon.context.dependencies");
        source_path.hash(&mut dependencies);
        let (overlays, overlay_inputs) = match overlay_fingerprint {
            Some(fingerprint) => (
                fingerprint.identity,
                OverlayInputs::Complete(std::sync::Arc::clone(&fingerprint.inputs)),
            ),
            None => {
                let mut overlay_entries = Vec::with_capacity(overlays.len());
                overlays.visit(&mut |path, text| overlay_entries.push((path, text)));
                overlay_entries.sort_by(|left, right| left.0.cmp(right.0));
                let mut overlay_hash = vize_l0::micro_default_hasher!("canon.context.overlays");
                let mut overlay_inputs = FxHashMap::default();
                for (path, text) in overlay_entries {
                    path.hash(&mut overlay_hash);
                    text.hash(&mut overlay_hash);
                    overlay_inputs.insert(
                        path.to_path_buf(),
                        overlay_snapshot::overlay_content_identity(text),
                    );
                }
                (
                    overlay_hash.finish(),
                    OverlayInputs::Compact(std::sync::Arc::new(overlay_inputs)),
                )
            }
        };
        let generation_options =
            editor_namespace_identity(options, virtual_ts_options, project_root, tsconfig_path);
        Self {
            host_content: host.finish(),
            host_path,
            host_graph: None,
            dependencies: dependencies.finish(),
            overlays,
            overlay_inputs,
            generation_options,
            workspace: false,
            stamps: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn stamp(&mut self, context: &AliasContext) {
        self.stamp_inner(context, None);
    }

    /// Syntax facts end with this build/stamp invocation; probe and disk
    /// observations still belong to the current fingerprint capture.
    pub(in super::super) fn stamp_with_import_facts(
        &mut self,
        context: &AliasContext,
        facts: &mut ModuleSpecifierFactsCache,
    ) {
        self.stamp_inner(context, Some(facts));
    }

    fn stamp_inner(
        &mut self,
        context: &AliasContext,
        facts: Option<&mut ModuleSpecifierFactsCache>,
    ) {
        use std::hash::{Hash, Hasher};
        let mut graph = vize_l0::micro_default_hasher!("canon.context.host_graph");
        context.host_facts.hash(&mut graph);
        self.host_graph = Some(graph.finish());
        let mut paths = vec![self.host_path.clone()];
        if let Some(mirror) = context.mirror.as_ref() {
            // The immutable compiler epoch belongs to this editor session.
            // Its physical dependencies remain watched by the owner, while
            // source/import/package inputs retain their live content stamps.
            if !mirror.has_admitted_compiler_configuration() {
                paths.push(context.project_root.join("tsconfig.json"));
            }
            paths.extend(
                mirror
                    .governing_config_paths()
                    .into_iter()
                    .filter(|path| !mirror.is_admitted_compiler_input(path)),
            );
            // The materialized closure intentionally keeps content digests:
            // same-mtime, same-length edits must invalidate an editor session.
            paths.extend(mirror.registered_original_paths_sorted());
            paths.extend(match facts {
                Some(facts) => mirror.editor_resolution_inputs_with_import_facts(facts),
                None => mirror.editor_resolution_inputs(),
            });
            paths.extend(
                context
                    .route_inputs
                    .iter()
                    .filter(|path| !mirror.is_admitted_compiler_input(path))
                    .cloned(),
            );
        } else {
            paths.push(context.project_root.join("tsconfig.json"));
            paths.extend(context.route_inputs.iter().cloned());
        }
        paths.sort();
        paths.dedup();
        self.stamps = paths
            .into_iter()
            .map(crate::package_route::stamp::InputStamp::capture)
            .collect();
    }

    /// The build registers the host from its own buffer, never from a
    /// requested entry, so the host's entry is not a dependency input.
    pub(crate) fn include_requested_sources(
        &mut self,
        source_path: &Path,
        sources: &[(PathBuf, &str)],
    ) {
        if sources.is_empty() {
            return;
        }
        use std::hash::{Hash, Hasher};
        let host_path = vize_carton::path::canonicalize_non_verbatim(source_path);
        let mut hash = vize_l0::micro_default_hasher!("canon.context.requested_sources");
        self.dependencies.hash(&mut hash);
        let mut sources = sources
            .iter()
            .filter(|(path, _)| path != source_path && *path != host_path)
            .collect::<Vec<_>>();
        sources.sort_by(|left, right| left.0.cmp(&right.0));
        for (path, source) in sources {
            path.hash(&mut hash);
            source.hash(&mut hash);
        }
        self.dependencies = hash.finish();
        self.workspace = true;
    }

    /// Whether `self` can differ from `previous` only in the host's text, so
    /// the previous revision's graph can take the host's new buffer.
    pub(super) fn revises(&self, previous: &Self) -> bool {
        self.dependencies == previous.dependencies
            && self.generation_options == previous.generation_options
            && self.workspace == previous.workspace
            && self.dependency_overlays_match(previous)
    }

    /// Compare the inputs that produced the previous graph. A request still
    /// carries every current overlay, including newly opened failed probes.
    fn dependency_overlays_match(&self, previous: &Self) -> bool {
        if self.overlays == previous.overlays {
            return true;
        }
        if previous.stamps.is_empty() {
            return false;
        }
        previous.stamps.iter().all(|stamp| {
            let path = stamp.path();
            !previous.reads_dependency_overlay(path)
                || self.overlay_inputs.get(path) == previous.overlay_inputs.get(path)
        })
    }

    fn reads_dependency_overlay(&self, path: &Path) -> bool {
        path != self.host_path
            && self
                .stamps
                .binary_search_by(|stamp| stamp.path().cmp(path))
                .is_ok()
    }

    /// Source text equality does not make a live root current when one of its
    /// imported open buffers changed. Keep that dependent root in the rebuild.
    pub(super) fn dependency_overlays_current(&self, overlays: &dyn OverlaySources) -> bool {
        self.stamps.iter().all(|stamp| {
            let path = stamp.path();
            !self.reads_dependency_overlay(path)
                || overlays
                    .get(path)
                    .map(overlay_snapshot::overlay_content_identity)
                    == self.overlay_inputs.get(path)
        })
    }

    pub(super) fn matches_revision(&self, previous: &Self) -> bool {
        self.host_content == previous.host_content && self.revises(previous)
    }

    /// New host bytes replace its old projection in the shared mirror, while
    /// the other revision still owns unchanged dependency files. A close, or a
    /// changed dependency buffer, does retire the member.
    pub(super) fn preserves_member(&self, previous: &Self) -> bool {
        let host_is_preserved = if self.host_path == previous.host_path {
            self.host_content == previous.host_content
                || (self.host_graph.is_some() && self.host_graph == previous.host_graph)
        } else {
            // A refreshed sibling's old topology is not this host's graph.
            // Retire its old revision rather than preserve obsolete suffixes,
            // roots or package routes on the strength of our own host facts.
            self.overlay_inputs.get(&previous.host_path)
                == previous.overlay_inputs.get(&previous.host_path)
        };
        host_is_preserved
            && self.overlay_inputs.contains_key(&previous.host_path)
                == previous.overlay_inputs.contains_key(&previous.host_path)
            && self.dependency_overlays_match(previous)
    }

    pub(in super::super) fn compact_overlay_inputs(&mut self) {
        // A revision snapshot is shared by every host. Materialize only this
        // graph's read set instead of cloning and filtering the complete map.
        let mut compact = FxHashMap::default();
        if let Some(identity) = self.overlay_inputs.get(&self.host_path) {
            compact.insert(self.host_path.clone(), identity);
        }
        for stamp in &self.stamps {
            let path = stamp.path();
            if path != self.host_path
                && let Some(identity) = self.overlay_inputs.get(path)
            {
                compact.insert(path.to_path_buf(), identity);
            }
        }
        self.overlay_inputs = OverlayInputs::Compact(std::sync::Arc::new(compact));
    }

    /// Disk inputs do not move with the host's buffer; a revision keeps them.
    pub(in super::super) fn adopt_stamps(&mut self, previous: &Self) {
        self.stamps.clone_from(&previous.stamps);
        self.host_graph = previous.host_graph;
    }

    pub(super) fn workspace(&self) -> bool {
        self.workspace
    }

    pub(super) fn stamps_still_valid(
        &self,
        observed: &mut crate::package_route::stamp::InputStampValidation,
    ) -> bool {
        self.stamps
            .iter()
            .all(|stamp| observed.is_current(stamp, None))
    }

    #[cfg(test)]
    pub(crate) fn input_stamps(&self) -> Vec<crate::package_route::stamp::InputStamp> {
        self.stamps.clone()
    }

    pub(crate) fn overlay_identity(&self) -> u64 {
        self.overlays
    }

    /// Host, overlay and option identity without the disk stamps, so equal
    /// requests can share one preparation.
    pub(crate) fn identity(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hash = vize_l0::micro_default_hasher!("canon.context.identity");
        (
            self.host_content,
            self.dependencies,
            self.overlays,
            self.generation_options,
        )
            .hash(&mut hash);
        hash.finish()
    }
}

#[cfg(test)]
impl super::SessionCache {
    pub(in super::super) fn cached_input_stamps(
        &self,
        source_path: &Path,
    ) -> Option<Vec<crate::package_route::stamp::InputStamp>> {
        self.slots
            .iter()
            .filter(|(key, _)| key.source_path == source_path)
            .max_by_key(|(_, cached)| cached.last_used)
            .map(|(_, cached)| cached.fingerprint.input_stamps())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::ContextFingerprint;
    use crate::corsa_bridge::vue_document::CorsaOverlaySnapshot;

    #[test]
    fn revision_snapshot_shares_complete_identities_then_compacts_per_host() {
        let host = std::path::PathBuf::from("/workspace/Host.vue");
        let dependency = std::path::PathBuf::from("/workspace/Dependency.vue");
        let snapshot = CorsaOverlaySnapshot::new(
            7,
            vec![
                (host.clone(), Arc::new("<template />".to_owned())),
                (
                    dependency.clone(),
                    Arc::new("<template>{{ value }}</template>".to_owned()),
                ),
            ],
        );
        let overlays: vize_carton::FxHashMap<std::path::PathBuf, &str> = snapshot
            .iter()
            .map(|(path, content)| (path.clone(), content.as_str()))
            .collect();
        let capture = || {
            ContextFingerprint::capture_with_overlay(
                &host,
                "<template />",
                &overlays,
                Some(snapshot.fingerprint()),
                Default::default(),
                &Default::default(),
                None,
                None,
            )
        };

        let first = capture();
        let mut second = capture();
        assert_eq!(first.overlay_identity(), 7);
        assert!(
            first
                .overlay_inputs
                .shares_storage_with(&second.overlay_inputs)
        );

        second.compact_overlay_inputs();
        assert_eq!(second.overlay_inputs.len(), 1);
        assert!(second.overlay_inputs.contains_key(&host));
        assert!(
            !first
                .overlay_inputs
                .shares_storage_with(&second.overlay_inputs)
        );
    }
}
