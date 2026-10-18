//! Build one workspace revision outside the native editor transaction.

use std::path::{Path, PathBuf};

use vize_carton::FxHashMap;

use super::{AliasContext, ContextFingerprint, build};
use crate::batch::ModuleSpecifierFactsCache;
use crate::corsa_bridge::types::CorsaBridgeError;
use crate::corsa_bridge::vue_document::{CorsaProjectEnvironment, CorsaVueVirtualDocumentOptions};

impl AliasContext {
    pub(super) fn request_fingerprint(
        source_path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
        requested_sources: &[(PathBuf, &str)],
        options: CorsaVueVirtualDocumentOptions,
        environment: CorsaProjectEnvironment<'_>,
    ) -> ContextFingerprint {
        let mut fingerprint = ContextFingerprint::capture(
            source_path,
            content,
            overlays,
            options,
            environment.virtual_ts_options,
            environment.project_root,
            environment.tsconfig_path,
        );
        fingerprint.include_requested_sources(requested_sources);
        fingerprint
    }

    /// Identity of one preparation; concurrent equal requests share a build.
    pub(in crate::corsa_bridge) fn preparation_key(
        source_path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
        requested_sources: &[(PathBuf, &str)],
        options: CorsaVueVirtualDocumentOptions,
        environment: CorsaProjectEnvironment<'_>,
    ) -> u64 {
        Self::request_fingerprint(
            source_path,
            content,
            overlays,
            requested_sources,
            options,
            environment,
        )
        .identity()
    }

    /// Whether opening this exact revision reuses finished work instead of
    /// building the whole workspace projection on the calling thread.
    pub(in crate::corsa_bridge) fn hosts_are_prepared(
        source_path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
        requested_sources: &[(PathBuf, &str)],
        options: CorsaVueVirtualDocumentOptions,
        environment: CorsaProjectEnvironment<'_>,
    ) -> bool {
        let fingerprint = Self::request_fingerprint(
            source_path,
            content,
            overlays,
            requested_sources,
            options,
            environment,
        );
        let mut cache = environment.editor_session.cache();
        cache.get(source_path, &fingerprint).is_some()
            || cache.has_prebuilt(source_path, &fingerprint, overlays)
    }

    /// Build and stamp a revision without materializing it. The request's own
    /// transaction adopts it, so mirror files and native state only change
    /// while that transaction holds the editor session.
    pub(in crate::corsa_bridge) fn prebuild_for_hosts(
        source_path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
        requested_sources: &[(PathBuf, &str)],
        options: CorsaVueVirtualDocumentOptions,
        environment: CorsaProjectEnvironment<'_>,
    ) -> Result<(), CorsaBridgeError> {
        let mut fingerprint = Self::request_fingerprint(
            source_path,
            content,
            overlays,
            requested_sources,
            options,
            environment,
        );
        let generation = {
            let mut cache = environment.editor_session.cache();
            if cache.get(source_path, &fingerprint).is_some()
                || cache.has_prebuilt(source_path, &fingerprint, overlays)
            {
                return Ok(());
            }
            cache.generation()
        };
        let phase = crate::corsa_bridge::preparation_trace::Phase::start(
            "alias_prebuild",
            requested_sources.len(),
        );
        let context = {
            // Keep syntax facts only until the immediately following stamp.
            // Stored prebuilt revisions never retain the source/facts cache.
            let mut facts = ModuleSpecifierFactsCache::default();
            let mut resolver = environment.package_routes.clone();
            let context = build::build_with_import_facts(
                source_path,
                content,
                overlays,
                build::SourceRevision {
                    requested_sources,
                    overlay_identity: fingerprint.overlay_identity(),
                },
                &mut resolver,
                options,
                environment,
                &mut facts,
            )?;
            fingerprint.stamp_with_import_facts(&context, &mut facts);
            context
        };
        phase.finish();
        #[cfg(test)]
        ModuleSpecifierFactsCache::assert_no_live_preparation_facts();
        environment.editor_session.cache().store_prebuilt(
            generation,
            source_path.to_path_buf(),
            fingerprint,
            context,
        );
        Ok(())
    }
}
