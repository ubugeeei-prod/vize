//! Cached, importer-scoped routes for Vue editor virtual documents.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, String as CompactString};

use crate::batch::virtual_project::VirtualProject;
use crate::batch::virtual_project::dependency_scan::resolve_dependency;

#[path = "context/build.rs"]
mod build;
#[path = "context/cache.rs"]
mod cache;
pub(in crate::corsa_bridge) use cache::SessionCache;
pub(in crate::corsa_bridge) use cache::recover_lock;
use cache::{ContextFingerprint, ProjectMember};
#[path = "context/routes.rs"]
mod routes;

// Project snapshots are shared by independent semantic requests in the
// process-wide bounded cache; a scoped borrow cannot represent that lifetime.
#[allow(clippy::disallowed_types)]
pub(in crate::corsa_bridge) struct PreparedAliasContext {
    context: std::sync::Arc<AliasContext>,
    pub(in crate::corsa_bridge) materialized_changes:
        crate::batch::virtual_project::MaterializedFileDelta,
}

impl std::ops::Deref for PreparedAliasContext {
    type Target = AliasContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

/// The alias map for the host document's package, resolved once per open,
/// plus the materialized mirror the resolutions point into.
///
/// The checker resolves modules from the disk only — open in-memory documents
/// are never resolution targets — so first-party Vue imports must land on real
/// generated files. Package routes retain the importer in their key instead of
/// leaking into a global exact-specifier map.
#[allow(clippy::disallowed_types)]
pub(in crate::corsa_bridge) struct AliasContext {
    pub(in crate::corsa_bridge) project_root: PathBuf,
    pub(in crate::corsa_bridge) aliases: Vec<(std::string::String, std::string::String)>,
    pub(in crate::corsa_bridge) package_routes:
        FxHashMap<(PathBuf, CompactString), crate::PackageRoute>,
    route_inputs: Vec<PathBuf>,
    mirror: Option<VirtualProject>,
    virtual_ts_options: crate::virtual_ts::VirtualTsOptions,
}

impl AliasContext {
    /// Build or reuse a context while every route input remains unchanged.
    #[allow(clippy::disallowed_types)]
    pub(in crate::corsa_bridge) fn for_host_cached(
        source_path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
        options: super::super::vue_document::CorsaVueVirtualDocumentOptions,
        environment: super::super::vue_document::CorsaProjectEnvironment<'_>,
    ) -> Result<PreparedAliasContext, super::super::types::CorsaBridgeError> {
        let fingerprint = ContextFingerprint::capture(
            source_path,
            content,
            overlays,
            options,
            environment.virtual_ts_options,
            environment.project_root,
            environment.tsconfig_path,
        );
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
            &mut resolver,
            options,
            environment,
        )?;
        let mut fingerprint = fingerprint;
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
            let query_path = mirror.preferred_materialized_path_for_original(&source_path);
            let (preserved_files, preserved_package_links, mut query_paths) = cache
                .project_union_snapshot(
                    mirror.virtual_root(),
                    &source_path,
                    fingerprint.overlay_identity(),
                );
            if let Some(query_path) = query_path.as_ref() {
                query_paths.push(query_path.clone());
            }
            query_paths.sort();
            query_paths.dedup();
            let previous = cache.materialized_snapshot(mirror.virtual_root());
            let current = mirror
                .materialize_editor_union(&preserved_files, &preserved_package_links, &query_paths)
                .map_err(|error| {
                    super::super::types::CorsaBridgeError::CommunicationError(vize_carton::cstr!(
                        "Failed to materialize Canon project union: {error}"
                    ))
                })?;
            materialized_changes = current.diff(&previous);
            cache.set_materialized_snapshot(mirror.virtual_root().to_path_buf(), current);
            cache.record_project_member(
                mirror.virtual_root().to_path_buf(),
                source_path,
                ProjectMember {
                    expected_files,
                    package_links,
                    query_path,
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

    #[cfg(test)]
    pub(in crate::corsa_bridge) fn for_host(
        source_path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
    ) -> Self {
        build::build(
            source_path,
            content,
            overlays,
            &mut crate::PackageRouteResolver::default(),
            Default::default(),
            super::super::vue_document::CorsaProjectEnvironment {
                virtual_ts_options: &Default::default(),
                package_routes: &crate::PackageRouteResolver::default(),
                project_root: None,
                tsconfig_path: None,
                editor_session: crate::corsa_bridge::editor_session::fallback_editor_session(),
            },
        )
        .expect("test alias context")
    }

    /// Resolve a Vue route to the materialized companion used by Corsa.
    #[allow(clippy::disallowed_types)]
    pub(in crate::corsa_bridge) fn resolve_specifier_to_mirror_path(
        &self,
        specifier: &str,
        importer_dir: &Path,
        occurrence_mode: crate::PackageResolutionMode,
    ) -> Option<std::string::String> {
        if let Some(path) =
            resolve_dependency(specifier, importer_dir, &self.project_root, &self.aliases)
        {
            let key = std::fs::canonicalize(&path).unwrap_or(path);
            if super::inside_node_modules(&key) || super::is_declaration(&key) {
                return None;
            }
            return self.mirror_specifier_for_source(&key);
        }
        // Bare/package-private spellings stay authored. The importer itself is
        // queried inside the Canon mirror, where native TypeScript sees its
        // importer-local node_modules and the byte-identical raw manifest.
        let _ = occurrence_mode;
        None
    }

    #[allow(clippy::disallowed_types)]
    pub(in crate::corsa_bridge) fn resolve_relative_vue_to_mirror_path(
        &self,
        specifier: &str,
        importer_dir: &Path,
    ) -> Option<std::string::String> {
        if !specifier.starts_with("./") && !specifier.starts_with("../") {
            return None;
        }
        let source = std::fs::canonicalize(importer_dir.join(specifier)).ok()?;
        if source
            .extension()
            .is_none_or(|extension| extension != "vue")
        {
            return None;
        }
        self.mirror_specifier_for_source(&source)
    }

    #[allow(clippy::disallowed_types)]
    fn mirror_specifier_for_source(&self, source: &Path) -> Option<std::string::String> {
        let target = self
            .mirror
            .as_ref()?
            .find_by_original(source)?
            .virtual_path
            .clone();
        let spelled = target.to_string_lossy().replace('\\', "/");
        Some(
            spelled
                .strip_suffix(".tsx")
                .or_else(|| spelled.strip_suffix(".ts"))
                .unwrap_or(&spelled)
                .to_owned(),
        )
    }

    pub(in crate::corsa_bridge) fn mirror_virtual_path(&self, source: &Path) -> Option<PathBuf> {
        self.mirror
            .as_ref()?
            .preferred_materialized_path_for_original(source)
    }

    pub(in crate::corsa_bridge) fn virtual_ts_options(
        &self,
    ) -> &crate::virtual_ts::VirtualTsOptions {
        &self.virtual_ts_options
    }

    pub(in crate::corsa_bridge) fn mirror_project_root_for_source(
        &self,
        source: &Path,
    ) -> Option<PathBuf> {
        let mirror = self.mirror.as_ref()?;
        mirror.preferred_materialized_path_for_original(source)?;
        Some(mirror.virtual_root().to_path_buf())
    }

    pub(in crate::corsa_bridge) fn materialized_sources(
        &self,
    ) -> Vec<super::super::vue_document::CorsaMaterializedSource> {
        self.mirror
            .as_ref()
            .map(|mirror| {
                mirror
                    .materialized_source_documents()
                    .into_iter()
                    .map(
                        |document| super::super::vue_document::CorsaMaterializedSource {
                            materialized_path: document.materialized_path,
                            source_path: document.source_path,
                            source: document.source,
                            code: document.code,
                            mappings: document.mappings,
                            semantic_links: document.semantic_links,
                            import_source_map: document.import_source_map,
                            mapping_kind: match document.mapping_kind {
                                crate::batch::virtual_project::MaterializedSourceMappingKind::Generated => {
                                    super::super::vue_document::CorsaMaterializedMappingKind::Generated
                                }
                                crate::batch::virtual_project::MaterializedSourceMappingKind::AuthoredIdentity => {
                                    super::super::vue_document::CorsaMaterializedMappingKind::AuthoredIdentity
                                }
                                crate::batch::virtual_project::MaterializedSourceMappingKind::Synthetic => {
                                    super::super::vue_document::CorsaMaterializedMappingKind::Synthetic
                                }
                            },
                        },
                    )
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(in crate::corsa_bridge) fn resolve_first_party_source(
        &self,
        specifier: &str,
        importer_dir: &Path,
    ) -> Option<PathBuf> {
        resolve_dependency(specifier, importer_dir, &self.project_root, &self.aliases)
    }

    pub(in crate::corsa_bridge) fn package_route(
        &self,
        specifier: &str,
        importer_dir: &Path,
    ) -> Option<&crate::PackageRoute> {
        let package_key = (
            vize_carton::path::canonicalize_non_verbatim(importer_dir),
            CompactString::from(specifier),
        );
        self.package_routes.get(&package_key)
    }

    pub(in crate::corsa_bridge) fn forget_cached_sources(
        session: &crate::corsa_bridge::EditorMirrorSession,
        source_paths: &[PathBuf],
    ) {
        session.cache().forget_sources(source_paths);
    }
}
