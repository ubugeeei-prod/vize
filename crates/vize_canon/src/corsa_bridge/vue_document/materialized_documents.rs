//! Keep live native overlays on the same revision as their materialized files.

use std::path::PathBuf;

use vize_carton::{FxHashMap, FxHashSet, String};

use super::{CorsaMaterializedMappingKind, CorsaMaterializedSource};
use crate::file_uri::path_to_file_uri;

pub(in crate::corsa_bridge) fn append_materialized_documents(
    documents: &mut Vec<(String, String)>,
    sources: &[CorsaMaterializedSource],
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
        let uri = path_to_file_uri(&source.materialized_path);
        if opened.insert(uri.clone()) {
            documents.push((uri, source.code.clone()));
        }
    }
}
