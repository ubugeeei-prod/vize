//! Open-importer refresh and Corsa overlay eviction for workspace file events.

use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::Url;

use super::super::{ServerState, importers};

pub(super) fn versioned_open_vue_dependents<'a>(
    state: &ServerState,
    uris: impl Iterator<Item = &'a str>,
) -> Vec<(Url, i32)> {
    let mut dependents = uris
        .filter_map(|uri| Url::parse(uri).ok())
        .flat_map(|uri| importers::open_vue_dependents(state, &uri))
        .collect::<Vec<_>>();
    dependents.sort();
    dependents.dedup();
    dependents
        .into_iter()
        .filter_map(|uri| state.documents.version(&uri).map(|version| (uri, version)))
        .collect()
}

pub(super) fn affected_vue_source_paths<'a>(
    state: &ServerState,
    uris: impl Iterator<Item = &'a str>,
) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    for path in uris.filter_map(file_path) {
        if is_vue_file(&path) {
            sources.push(path.clone());
        }
        sources.extend(
            importers::indexed_dependency_paths(state, &path)
                .into_iter()
                .filter(|dependency| is_vue_file(dependency)),
        );
    }
    sources.sort();
    sources.dedup();
    sources
}

pub(super) async fn forget_corsa_vue_files(state: &ServerState, deleted: &[PathBuf]) {
    if !state.has_corsa_bridge() {
        return;
    }
    let Some(bridge) = state.get_corsa_bridge().await else {
        return;
    };
    if let Err(error) = bridge.forget_vue_virtual_documents(deleted).await {
        tracing::warn!("failed to forget deleted Corsa Vue documents: {error}");
        state.retire_corsa_bridge(&bridge);
    }
}

fn file_path(uri: &str) -> Option<PathBuf> {
    Url::parse(uri).ok()?.to_file_path().ok()
}

fn is_vue_file(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "vue")
}
