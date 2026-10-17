//! Prepare and materialize one cached native project revision.

use std::path::{Path, PathBuf};

use vize_carton::FxHashMap;

use super::{AliasContext, PreparedAliasContext, ProjectMember, build};
use crate::batch::ModuleSpecifierFactsCache;

impl AliasContext {
    pub(in crate::corsa_bridge) fn for_host_cached(
        source_path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
        options: super::super::super::vue_document::CorsaVueVirtualDocumentOptions,
        environment: super::super::super::vue_document::CorsaProjectEnvironment<'_>,
    ) -> Result<PreparedAliasContext, super::super::super::types::CorsaBridgeError> {
        Self::for_hosts_cached(source_path, content, overlays, &[], options, environment)
    }

    /// Build or reuse a context while every route input remains unchanged.
    #[expect(clippy::disallowed_types, reason = "dependency API uses std String")]
    pub(in crate::corsa_bridge) fn for_hosts_cached(
        source_path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
        requested_sources: &[(PathBuf, &str)],
        options: super::super::super::vue_document::CorsaVueVirtualDocumentOptions,
        environment: super::super::super::vue_document::CorsaProjectEnvironment<'_>,
    ) -> Result<PreparedAliasContext, super::super::super::types::CorsaBridgeError> {
        // One preparation validates each package-route input once.
        let _route_validation = environment.package_routes.validation_scope();
        let capture_phase = crate::corsa_bridge::preparation_trace::Phase::start(
            "alias_fingerprint",
            overlays.len(),
        );
        let mut fingerprint = Self::request_fingerprint(
            source_path,
            content,
            overlays,
            requested_sources,
            options,
            environment,
        );
        capture_phase.finish();
        let cache_phase =
            crate::corsa_bridge::preparation_trace::Phase::start("alias_cache_lookup", 1);
        let cached = {
            let mut cache = environment.editor_session.cache();
            cache.get(source_path, &fingerprint).map(|context| {
                let catalog = context
                    .mirror
                    .as_ref()
                    .map_or_else(Default::default, |mirror| {
                        cache.source_catalog(mirror.virtual_root())
                    });
                (context, catalog)
            })
        };
        cache_phase.finish();
        if let Some((context, source_catalog)) = cached {
            return Ok(PreparedAliasContext {
                source_catalog,
                context,
                materialized_changes: Default::default(),
            });
        }
        // A revision prepared outside the native transaction is adopted only
        // while it equals what building here would produce.
        let prebuilt =
            environment
                .editor_session
                .cache()
                .take_prebuilt(source_path, &fingerprint, overlays);
        let mut context = if let Some((context, stamped)) = prebuilt {
            fingerprint = stamped;
            context
        } else {
            // Pure syntax facts live through build and its immediate stamp only.
            // The owner drops before any context is materialized or stored.
            let mut facts = ModuleSpecifierFactsCache::default();
            let mut resolver = environment.package_routes.clone();
            let build_phase = crate::corsa_bridge::preparation_trace::Phase::start(
                "alias_build",
                requested_sources.len(),
            );
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
            build_phase.finish();
            let stamp_phase =
                crate::corsa_bridge::preparation_trace::Phase::start("alias_stamp", 1);
            fingerprint.stamp_with_import_facts(&context, &mut facts);
            stamp_phase.finish();
            context
        };
        #[cfg(test)]
        ModuleSpecifierFactsCache::assert_no_live_preparation_facts();
        let mut cache = environment.editor_session.cache();
        if let Some(context) = cache.get(source_path, &fingerprint) {
            return Ok(PreparedAliasContext {
                source_catalog: context
                    .mirror
                    .as_ref()
                    .map_or_else(Default::default, |mirror| {
                        cache.source_catalog(mirror.virtual_root())
                    }),
                context,
                materialized_changes: Default::default(),
            });
        }
        let materialize_phase =
            crate::corsa_bridge::preparation_trace::Phase::start("alias_materialize", 1);
        let mut materialized_changes = Default::default();
        let mut source_catalog = Default::default();
        let materialized_sources = context.materialized_sources();
        if let Some(mirror) = context.mirror.as_mut() {
            let previous = cache.materialized_snapshot(mirror.virtual_root());
            mirror.restore_editor_package_links(&previous);
            let previous_catalog = cache.source_catalog(mirror.virtual_root());
            source_catalog = cache.include_source_catalog(
                mirror.virtual_root(),
                fingerprint.overlay_identity(),
                materialized_sources,
            );
            let source_path = vize_carton::path::canonicalize_non_verbatim(source_path);
            let expected_files = mirror.expected_materialized_files();
            let package_links = mirror.desired_package_links();
            let member_query_paths = mirror.editor_query_paths(&source_path);
            let mut union = cache.project_union_snapshot(
                mirror.virtual_root(),
                &source_path,
                fingerprint.overlay_identity(),
            );
            union.query_paths.extend(member_query_paths.iter().cloned());
            if !source_catalog.shares_revision_with(&previous_catalog) {
                source_catalog.retain_live_files(&expected_files, &union.files);
            }
            union.query_paths.sort();
            union.query_paths.dedup();
            let current = mirror
                .materialize_editor_union(
                    &union.files,
                    &union.package_links,
                    &union.query_paths,
                    union.requires_arbitrary_extensions,
                )
                .map_err(|error| {
                    super::super::super::types::CorsaBridgeError::CommunicationError(
                        vize_carton::cstr!("Failed to materialize Canon project union: {error}"),
                    )
                })?;
            materialized_changes = current.diff(&previous);
            cache.set_materialized_snapshot(mirror.virtual_root().to_path_buf(), current);
            cache.record_project_member(
                mirror.virtual_root().to_path_buf(),
                source_path,
                ProjectMember {
                    source_paths: mirror.registered_original_paths_sorted(),
                    expected_files,
                    package_links,
                    query_paths: member_query_paths,
                    stamps: fingerprint.input_stamps(),
                    overlay_identity: fingerprint.overlay_identity(),
                    requires_arbitrary_extensions: mirror.requires_arbitrary_extensions(),
                },
            );
        }
        materialize_phase.finish();
        let context = std::sync::Arc::new(context);
        cache.insert(
            source_path.to_path_buf(),
            fingerprint,
            std::sync::Arc::clone(&context),
        );
        Ok(PreparedAliasContext {
            context,
            materialized_changes,
            source_catalog,
        })
    }
}
