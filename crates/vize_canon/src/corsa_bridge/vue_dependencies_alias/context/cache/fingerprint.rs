//! Exact host, overlay and disk identities for cached project projections.

use super::super::{AliasContext, namespace::editor_namespace_identity};
use std::path::{Path, PathBuf};
use vize_carton::FxHashMap;

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

    pub(crate) fn stamp(&mut self, context: &AliasContext) {
        let mut paths = vec![context.project_root.join("tsconfig.json")];
        if let Some(mirror) = context.mirror.as_ref() {
            paths.extend(mirror.governing_config_paths());
            // The materialized closure intentionally keeps content digests:
            // same-mtime, same-length edits must invalidate an editor session.
            paths.extend(mirror.registered_original_paths_sorted());
            paths.extend(mirror.editor_resolution_inputs());
        }
        paths.extend(context.route_inputs.iter().cloned());
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

    pub(super) fn stamps_still_valid(&self) -> bool {
        self.stamps
            .iter()
            .all(crate::package_route::stamp::InputStamp::is_current)
    }

    pub(crate) fn input_stamps(&self) -> Vec<crate::package_route::stamp::InputStamp> {
        self.stamps.clone()
    }

    pub(crate) fn overlay_identity(&self) -> u64 {
        self.overlays
    }
}
