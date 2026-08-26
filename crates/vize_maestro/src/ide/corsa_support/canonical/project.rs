use std::collections::VecDeque;

use tower_lsp::lsp_types::Url;
use vize_canon::{CorsaBridge, CorsaBridgeError};
use vize_s0::{FxHashSet, String};

use super::{CanonicalDependencyDocument, CanonicalVirtualDocument, location_matches_uri};
use crate::ide::IdeContext;

#[derive(Debug)]
pub(crate) enum CanonicalProjectOpenError {
    Primary(CorsaBridgeError),
    Importer(CorsaBridgeError),
}

impl CanonicalVirtualDocument {
    fn include_opened_document(&mut self, source_uri: Url, source: String, mut opened: Self) {
        self.include_dependency(CanonicalDependencyDocument {
            source_uri,
            source,
            request_uri: opened.request_uri,
            virtual_result: opened.virtual_result,
        });
        for dependency in opened.dependencies.drain(..) {
            self.include_dependency(dependency);
        }
        for materialized in opened.materialized_sources.drain(..) {
            if !self.materialized_sources.iter().any(|existing| {
                location_matches_uri(&existing.request_uri, &materialized.request_uri)
            }) {
                self.materialized_sources.push(materialized);
            }
        }
        self.session_project_roots
            .append(&mut opened.session_project_roots);
        self.session_project_roots.sort();
        self.session_project_roots.dedup();
    }

    fn include_dependency(&mut self, dependency: CanonicalDependencyDocument) {
        if location_matches_uri(&dependency.request_uri, &self.request_uri)
            || self.dependencies.iter().any(|existing| {
                location_matches_uri(&existing.request_uri, &dependency.request_uri)
            })
        {
            return;
        }
        self.dependencies.push(dependency);
    }

    /// Query a package source through one exact importer mirror once reverse
    /// importers have materialized it. The standalone overlay is no longer a
    /// live configured-project identity after the mirror reload; references and
    /// rename fan out only to the remaining live materialized identities.
    fn promote_materialized_query_identity(&mut self) {
        if self
            .materialized_sources
            .iter()
            .any(|source| location_matches_uri(&source.request_uri, self.request_uri.as_str()))
        {
            return;
        }
        let mut candidates = self
            .materialized_sources
            .iter()
            .enumerate()
            .filter(|(_, source)| {
                source.mapping_kind.is_mappable()
                    && authored_uris_match(&source.source_uri, &self.source_uri)
            })
            .map(|(index, source)| {
                let mapping_rank = match source.mapping_kind {
                    vize_canon::CorsaMaterializedMappingKind::Generated => 0,
                    vize_canon::CorsaMaterializedMappingKind::AuthoredIdentity => 1,
                    vize_canon::CorsaMaterializedMappingKind::Synthetic => 2,
                };
                (mapping_rank, source.request_uri.clone(), index)
            })
            .collect::<Vec<_>>();
        candidates.sort();
        let Some((_, _, index)) = candidates.into_iter().next() else {
            return;
        };
        let selected = self.materialized_sources.remove(index);
        self.request_uri = selected.request_uri;
        self.virtual_result = selected.virtual_result;
    }
}

fn authored_uris_match(left: &Url, right: &Url) -> bool {
    if left == right {
        return true;
    }
    match (left.to_file_path(), right.to_file_path()) {
        (Ok(left), Ok(right)) => {
            vize_s0::path::canonicalize_non_verbatim(&left)
                == vize_s0::path::canonicalize_non_verbatim(&right)
        }
        _ => false,
    }
}

/// Open the query document and every currently-open reverse importer in one
/// canonical project session. This is intentionally separate from the hot
/// hover/definition path: project-wide operations need the fan-out, local
/// operations should not pay for it.
pub(crate) async fn open_canonical_virtual_project_document(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
) -> Option<CanonicalVirtualDocument> {
    open_canonical_virtual_project_document_strict(ctx, bridge)
        .await
        .ok()
        .flatten()
}

/// Open every authored Vue file in the workspace surface for operations whose
/// semantics are project-wide, such as `textDocument/references`.
pub(crate) async fn open_canonical_virtual_workspace_document(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
) -> Option<CanonicalVirtualDocument> {
    open_canonical_virtual_project_document_with_scope(ctx, bridge, true)
        .await
        .ok()
        .flatten()
}

/// Open a project-wide canonical document while retaining the bridge error
/// that lenient editor routes intentionally turn into synchronous fallback.
pub(crate) async fn open_canonical_virtual_project_document_strict(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
) -> Result<Option<CanonicalVirtualDocument>, CanonicalProjectOpenError> {
    open_canonical_virtual_project_document_with_scope(ctx, bridge, false).await
}

async fn open_canonical_virtual_project_document_with_scope(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
    include_workspace: bool,
) -> Result<Option<CanonicalVirtualDocument>, CanonicalProjectOpenError> {
    let cached_overlays = ctx.state.corsa_overlays();
    let overlays = cached_overlays
        .iter()
        .map(|(path, content)| (path.clone(), &**content))
        .collect::<Vec<_>>();
    let Some(mut document) =
        super::open::open_canonical_virtual_document_with_overlays_strict(ctx, bridge, &overlays)
            .await
            .map_err(CanonicalProjectOpenError::Primary)?
    else {
        return Ok(None);
    };
    let mut visited = FxHashSet::default();
    visited.insert(ctx.uri.clone());
    let mut queue = VecDeque::from(ctx.state.open_importers(ctx.uri));

    while let Some(uri) = queue.pop_front() {
        if !visited.insert(uri.clone()) {
            continue;
        }
        queue.extend(ctx.state.open_importers(&uri));
        let Some(importer_ctx) = IdeContext::new(ctx.state, &uri, 0) else {
            continue;
        };
        if let Some(opened) = super::open::open_canonical_virtual_document_with_overlays_strict(
            &importer_ctx,
            bridge,
            &overlays,
        )
        .await
        .map_err(CanonicalProjectOpenError::Importer)?
        {
            document.include_opened_document(uri.clone(), importer_ctx.content.into(), opened);
        }
    }

    if include_workspace {
        let workspace_sources = ctx.state.discover_workspace_vue_sources().await;
        for (uri, source) in same_typescript_project(ctx, workspace_sources) {
            if !visited.insert(uri.clone()) || uri.path().ends_with(".art.vue") {
                continue;
            }
            let importer_ctx = IdeContext::with_content(ctx.state, &uri, 0, source.clone());
            if let Some(opened) = super::open::open_canonical_virtual_document_with_overlays_strict(
                &importer_ctx,
                bridge,
                &overlays,
            )
            .await
            .map_err(CanonicalProjectOpenError::Importer)?
            {
                document.include_opened_document(uri, source.into(), opened);
            }
        }
    }

    let materialized_documents = document
        .materialized_sources
        .iter()
        .filter(|source| source.mapping_kind.is_mappable())
        .filter(|source| {
            !location_matches_uri(&source.request_uri, &document.request_uri)
                && !document.dependencies.iter().any(|dependency| {
                    location_matches_uri(&source.request_uri, &dependency.request_uri)
                })
        })
        .map(|source| {
            (
                source.request_uri.clone(),
                source.virtual_result.code.as_str().into(),
            )
        })
        .collect::<Vec<_>>();
    bridge
        .open_virtual_documents_batch(&materialized_documents)
        .await
        .map_err(CanonicalProjectOpenError::Importer)?;
    document.promote_materialized_query_identity();

    Ok(Some(document))
}

/// Keep configured-project operations out of unrelated workspace packages.
/// If the governing config is missing, preserve the inferred-project fallback
/// and search the discovered workspace surface. A configured project that does
/// not own the query must not donate any workspace sources to that query.
fn same_typescript_project(
    ctx: &IdeContext<'_>,
    sources: Vec<(Url, std::string::String)>,
) -> Vec<(Url, std::string::String)> {
    let Some(source_path) = ctx.uri.to_file_path().ok() else {
        return sources;
    };
    let Some(tsconfig) = source_path
        .ancestors()
        .skip(1)
        .map(|directory| directory.join("tsconfig.json"))
        .find(|candidate| candidate.is_file())
    else {
        return sources;
    };
    let mut ownership = vize_canon::batch::TsconfigOwnershipCache::default();
    let projects = ownership.project_paths(&tsconfig);
    let owns = |ownership: &mut vize_canon::batch::TsconfigOwnershipCache,
                path: &std::path::Path| {
        projects.iter().any(|project| {
            ownership.project_owns_source(
                project,
                path,
                vize_canon::batch::TsconfigSourceKind::Typed,
            )
        })
    };
    if !owns(&mut ownership, &source_path) {
        return Vec::new();
    }
    sources
        .into_iter()
        .filter(|(uri, _)| {
            uri.to_file_path()
                .ok()
                .is_some_and(|path| owns(&mut ownership, &path))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tower_lsp::lsp_types::Url;

    use super::same_typescript_project;
    use crate::ide::IdeContext;
    use crate::server::ServerState;

    #[test]
    fn configured_project_does_not_feed_workspace_sources_to_excluded_query() {
        let project = tempfile::TempDir::new().expect("temp project");
        let src = project.path().join("src");
        let ignored = project.path().join("ignored");
        fs::create_dir_all(&src).expect("src directory");
        fs::create_dir_all(&ignored).expect("ignored directory");
        fs::write(
            project.path().join("tsconfig.json"),
            r#"{ "include": ["src/**/*"] }"#,
        )
        .expect("tsconfig");

        let included_path = src.join("Included.vue");
        let excluded_path = ignored.join("Excluded.vue");
        let source = "<script setup lang=\"ts\">const shared = 1</script>";
        fs::write(&included_path, source).expect("included component");
        fs::write(&excluded_path, source).expect("excluded component");
        let included_uri = Url::from_file_path(included_path).expect("included URI");
        let excluded_uri = Url::from_file_path(excluded_path).expect("excluded URI");

        let state = ServerState::new();
        state.documents.open(
            excluded_uri.clone(),
            source.to_string(),
            1,
            "vue".to_string(),
        );
        let ctx = IdeContext::new(&state, &excluded_uri, 0).expect("excluded query context");

        let filtered = same_typescript_project(
            &ctx,
            vec![
                (included_uri, source.to_string()),
                (excluded_uri.clone(), source.to_string()),
            ],
        );

        assert!(
            filtered.is_empty(),
            "a query excluded by the governing tsconfig must not search its workspace surface",
        );
    }
}
