//! Prepare and materialize one cached native project revision.

use std::path::{Path, PathBuf};

use vize_carton::FxHashMap;

use super::{AliasContext, ContextFingerprint, PreparedAliasContext, build};
use crate::batch::ModuleSpecifierFactsCache;

struct HostRevision<'a> {
    source_path: &'a Path,
    content: &'a str,
    fingerprint: ContextFingerprint,
}

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
    pub(in crate::corsa_bridge) fn for_hosts_cached(
        source_path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
        requested_sources: &[(PathBuf, &str)],
        options: super::super::super::vue_document::CorsaVueVirtualDocumentOptions,
        environment: super::super::super::vue_document::CorsaProjectEnvironment<'_>,
    ) -> Result<PreparedAliasContext, super::super::super::types::CorsaBridgeError> {
        let capture_phase = crate::corsa_bridge::preparation_trace::Phase::start(
            "alias_fingerprint",
            overlays.len(),
        );
        let fingerprint = Self::request_fingerprint(
            source_path,
            content,
            overlays,
            requested_sources,
            options,
            environment,
        );
        capture_phase.finish();
        let sibling = environment.editor_session.cache().changed_overlay_sibling(
            source_path,
            &fingerprint,
            overlays,
        );
        if let Some((sibling, root, previous)) = sibling {
            if let Some(source) = overlays.get(&sibling) {
                // This non-recursive entry refreshes all changed live sources
                // once; several dirty buffers cannot recursively prepare each
                // other. The unchanged caller retains its own graph.
                let sibling_fingerprint = Self::request_fingerprint(
                    &sibling,
                    source,
                    overlays,
                    &[],
                    options,
                    environment,
                );
                drop(Self::prepare_hosts_cached(
                    HostRevision {
                        source_path: &sibling,
                        content: source,
                        fingerprint: sibling_fingerprint,
                    },
                    overlays,
                    &[],
                    options,
                    environment,
                )?);
                let mut prepared = Self::prepare_hosts_cached(
                    HostRevision {
                        source_path,
                        content,
                        fingerprint,
                    },
                    overlays,
                    requested_sources,
                    options,
                    environment,
                )?;
                prepared.materialized_changes = environment
                    .editor_session
                    .cache()
                    .materialized_snapshot(&root)
                    .diff(&previous);
                return Ok(prepared);
            }
        }
        Self::prepare_hosts_cached(
            HostRevision {
                source_path,
                content,
                fingerprint,
            },
            overlays,
            requested_sources,
            options,
            environment,
        )
    }

    #[expect(clippy::disallowed_types, reason = "dependency API uses std String")]
    fn prepare_hosts_cached(
        host: HostRevision<'_>,
        overlays: &FxHashMap<PathBuf, &str>,
        requested_sources: &[(PathBuf, &str)],
        options: super::super::super::vue_document::CorsaVueVirtualDocumentOptions,
        environment: super::super::super::vue_document::CorsaProjectEnvironment<'_>,
    ) -> Result<PreparedAliasContext, super::super::super::types::CorsaBridgeError> {
        let HostRevision {
            source_path,
            content,
            mut fingerprint,
        } = host;
        // One preparation validates each package-route input once.
        let _route_validation = environment.package_routes.validation_scope();
        let cache_phase =
            crate::corsa_bridge::preparation_trace::Phase::start("alias_cache_lookup", 1);
        let (cached, cache_generation) = {
            let mut cache = environment.editor_session.cache();
            let generation = cache.generation();
            let cached = cache.get(source_path, &fingerprint).map(|context| {
                let catalog = context
                    .mirror
                    .as_ref()
                    .map_or_else(Default::default, |mirror| {
                        cache.source_catalog(mirror.virtual_root())
                    });
                (context, catalog)
            });
            (cached, generation)
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
        let revised = if prebuilt.is_none() {
            Self::revise_cached(
                source_path,
                content,
                overlays,
                &mut fingerprint,
                environment,
            )?
        } else {
            None
        };
        let graph_hit = revised.is_some();
        let mut context = if let Some((context, stamped)) = prebuilt {
            // Keep the request's complete overlay observation for union
            // validation; the stored prebuild retained only its own read set.
            fingerprint.adopt_stamps(&stamped);
            context
        } else if let Some(context) = revised {
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
        if cache.generation() != cache_generation {
            return Err(
                super::super::super::types::CorsaBridgeError::CommunicationError(
                    vize_carton::cstr!(
                        "project inputs changed while the editor mirror was being prepared"
                    ),
                ),
            );
        }
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
        let (materialized_changes, source_catalog) =
            super::prepare_materialize::materialize_cached_revision(
                &mut context,
                &mut cache,
                source_path,
                &mut fingerprint,
                graph_hit,
            )?;
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
