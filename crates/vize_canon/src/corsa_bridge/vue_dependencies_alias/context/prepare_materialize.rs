//! Publish one editor revision's mirror.
//!
//! A graph-fingerprint hit writes and hashes only the files that revision
//! changed. A cold build and a fingerprint miss keep the full editor union,
//! including its projection and the snapshot a later diff retires against.

use std::path::Path;

use super::AliasContext;
use super::cache::{ContextFingerprint, ProjectMember, RevisionKey};

pub(super) fn materialize_cached_revision(
    context: &mut AliasContext,
    cache: &mut super::SessionCache,
    source_path: &Path,
    fingerprint: &mut ContextFingerprint,
    graph_hit: bool,
) -> Result<
    (
        crate::batch::virtual_project::MaterializedFileDelta,
        crate::corsa_bridge::CorsaSourceCatalog,
        bool,
    ),
    super::super::super::types::CorsaBridgeError,
> {
    let refresh_live_overlays = context.mirror.as_ref().is_some_and(|mirror| {
        cache.project_overlay_epoch_changed(mirror.virtual_root(), fingerprint.overlay_identity())
    });
    let mut materialized_changes = Default::default();
    let mut source_catalog = Default::default();
    let previous = context
        .mirror
        .as_ref()
        .map(|mirror| cache.materialized_snapshot(mirror.virtual_root()));
    let source_path = vize_carton::path::canonicalize_non_verbatim(source_path);
    let revision = RevisionKey::new(source_path.clone(), fingerprint);
    let mut union = context
        .mirror
        .as_ref()
        .map_or_else(Default::default, |mirror| {
            cache.project_union_snapshot(mirror.virtual_root(), &revision, fingerprint)
        });
    let narrow = graph_hit
        && !union.retired_members
        && previous.as_ref().is_some_and(|previous| {
            context.mirror.as_ref().is_some_and(|mirror| {
                cache.has_project_overlay_epoch(mirror.virtual_root())
                    && cache.has_project_member(mirror.virtual_root(), &revision)
                    && mirror.fingerprint_hit_is_file_patch(previous)
            })
        });
    let sources = if narrow {
        context.materialized_sources_for_candidates()
    } else {
        context.materialized_sources()
    };
    let Some(mirror) = context.mirror.as_mut() else {
        return Ok((materialized_changes, source_catalog, refresh_live_overlays));
    };
    let previous = previous.unwrap_or_default();
    mirror.restore_editor_package_links(&previous);
    source_catalog = cache.include_source_catalog(mirror.virtual_root(), sources);
    let expected_files = mirror.expected_materialized_files();
    let package_links = mirror.desired_package_links();
    let member_query_paths = mirror.editor_query_paths(&source_path);
    union.query_paths.extend(member_query_paths.iter().cloned());
    source_catalog = source_catalog.retaining_live_files(&expected_files, &union.files);
    cache.set_source_catalog(mirror.virtual_root(), source_catalog);
    union.query_paths.sort();
    union.query_paths.dedup();
    let current = if narrow {
        mirror.materialize_fingerprint_hit(&previous)
    } else {
        let current = mirror.materialize_editor_union(
            &previous,
            &union.files,
            &union.package_links,
            &union.query_paths,
            union.requires_arbitrary_extensions,
        );
        mirror.discard_incremental_materialization();
        current
    }
    .map_err(|error| match error {
        crate::CorsaError::Configuration(message) => {
            super::super::super::types::CorsaBridgeError::Configuration(message)
        }
        error => super::super::super::types::CorsaBridgeError::CommunicationError(
            vize_carton::cstr!("Failed to materialize Canon project union: {error}"),
        ),
    })?;
    materialized_changes = current.diff(&previous);
    source_catalog = cache.bind_materialized_source_catalog(mirror.virtual_root(), &current);
    cache.set_materialized_snapshot(mirror.virtual_root().to_path_buf(), current);
    fingerprint.compact_overlay_inputs();
    cache.record_project_member(
        mirror.virtual_root().to_path_buf(),
        revision,
        ProjectMember {
            source_paths: mirror.registered_original_paths_sorted(),
            expected_files,
            package_links,
            query_paths: member_query_paths,
            fingerprint: fingerprint.clone(),
            requires_arbitrary_extensions: mirror.requires_arbitrary_extensions(),
        },
    );
    Ok((materialized_changes, source_catalog, refresh_live_overlays))
}
