//! Exact host, overlay and disk identities for cached project projections.

use super::super::{AliasContext, namespace::editor_namespace_identity};
use std::path::{Path, PathBuf};
use vize_carton::FxHashMap;

use crate::batch::ModuleSpecifierFactsCache;

/// The import closure and disk inputs a cached editor route depends on.
#[derive(Clone)]
pub(crate) struct ContextFingerprint {
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
    pub(crate) fn capture(
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
        let mut paths = Vec::new();
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

    pub(crate) fn include_requested_sources(&mut self, sources: &[(PathBuf, &str)]) {
        if sources.is_empty() {
            return;
        }
        use std::hash::{Hash, Hasher};
        let mut hash = std::hash::DefaultHasher::new();
        self.host_content.hash(&mut hash);
        let mut sources = sources.iter().collect::<Vec<_>>();
        sources.sort_by(|left, right| left.0.cmp(&right.0));
        for (path, source) in sources {
            path.hash(&mut hash);
            source.hash(&mut hash);
        }
        self.host_content = hash.finish();
    }

    pub(super) fn stamps_still_valid(
        &self,
        observed: &mut crate::package_route::stamp::InputStampCache,
    ) -> bool {
        self.stamps
            .iter()
            .all(|stamp| stamp.is_current_with_cache(observed))
    }

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
        let mut hash = std::hash::DefaultHasher::new();
        (self.host_content, self.overlays, self.generation_options).hash(&mut hash);
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
            .get(source_path)
            .map(|cached| cached.fingerprint.input_stamps())
    }
}
