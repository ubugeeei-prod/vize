//! Prepare and materialize one cached native project revision.

use std::path::{Path, PathBuf};

use vize_carton::FxHashMap;

use super::{AliasContext, ContextFingerprint, PreparedAliasContext, ProjectMember, build};

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
    #[allow(clippy::disallowed_types)]
    pub(in crate::corsa_bridge) fn for_hosts_cached(
        source_path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
        requested_sources: &[(PathBuf, &str)],
        options: super::super::super::vue_document::CorsaVueVirtualDocumentOptions,
        environment: super::super::super::vue_document::CorsaProjectEnvironment<'_>,
    ) -> Result<PreparedAliasContext, super::super::super::types::CorsaBridgeError> {
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
        if let Some(context) = environment
            .editor_session
            .cache()
            .get(source_path, &fingerprint)
        {
            return Ok(PreparedAliasContext {
                context,
                materialized_changes: Default::default(),
            });
        }
        let mut resolver = environment.package_routes.clone();
        let context = build::build(
            source_path,
            content,
            overlays,
            requested_sources,
            &mut resolver,
            options,
            environment,
        )?;
        fingerprint.stamp(&context);
        let mut cache = environment.editor_session.cache();
        if let Some(context) = cache.get(source_path, &fingerprint) {
            return Ok(PreparedAliasContext {
                context,
                materialized_changes: Default::default(),
            });
        }
        let mut materialized_changes = Default::default();
        if let Some(mirror) = context.mirror.as_ref() {
            let source_path = vize_carton::path::canonicalize_non_verbatim(source_path);
            let expected_files = mirror.expected_materialized_files();
            let package_links = mirror.desired_package_links();
            let member_query_paths = mirror.editor_query_paths(&source_path);
            let (preserved_files, preserved_package_links, mut query_paths) = cache
                .project_union_snapshot(
                    mirror.virtual_root(),
                    &source_path,
                    fingerprint.overlay_identity(),
                );
            query_paths.extend(member_query_paths.iter().cloned());
            query_paths.sort();
            query_paths.dedup();
            let previous = cache.materialized_snapshot(mirror.virtual_root());
            let current = mirror
                .materialize_editor_union(&preserved_files, &preserved_package_links, &query_paths)
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
                },
            );
        }
        let context = std::sync::Arc::new(context);
        cache.insert(
            source_path.to_path_buf(),
            fingerprint,
            std::sync::Arc::clone(&context),
        );
        Ok(PreparedAliasContext {
            context,
            materialized_changes,
        })
    }
}
