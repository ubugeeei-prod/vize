use std::collections::VecDeque;

use tower_lsp::lsp_types::Url;
use vize_canon::{CorsaBridge, CorsaBridgeError};
use vize_carton::{FxHashSet, String};

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

/// Open a project-wide canonical document while retaining the bridge error
/// that lenient editor routes intentionally turn into synchronous fallback.
pub(crate) async fn open_canonical_virtual_project_document_strict(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
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

    Ok(Some(document))
}
