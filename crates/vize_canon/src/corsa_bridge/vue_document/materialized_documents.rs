//! Keep live native overlays on the same revision as their materialized files.
#![expect(
    clippy::disallowed_types,
    reason = "materialized payloads are shared with the source catalog"
)]

use std::path::PathBuf;
use std::sync::Arc;

use vize_carton::{FxHashMap, FxHashSet};

use super::{CorsaMaterializedMappingKind, CorsaMaterializedSource, CorsaNativeDocumentBatch};

/// Append each materialized leaf's own payload, so the batch shares its text.
pub(in crate::corsa_bridge) fn append_materialized_documents(
    documents: &mut CorsaNativeDocumentBatch,
    sources: &[Arc<CorsaMaterializedSource>],
    overlays: &FxHashMap<PathBuf, &str>,
    full_workspace: bool,
) {
    let mut opened = documents
        .iter()
        .map(|(uri, _)| uri.clone())
        .collect::<FxHashSet<_>>();
    for source in sources {
        if matches!(source.mapping_kind, CorsaMaterializedMappingKind::Synthetic)
            || (!full_workspace && !overlays.contains_key(&source.source_path))
        {
            continue;
        }
        let document = source.code.native_document();
        if opened.insert(document.0.clone()) {
            documents.push_shared(Arc::clone(document));
        }
    }
}
