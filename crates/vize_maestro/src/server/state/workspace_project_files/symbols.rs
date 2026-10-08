//! Read, collect and release one closed source at a time on the existing worker.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::task::Poll;

use tower_lsp::lsp_types::{SymbolInformation, Url};
use vize_l0::FxHashMap;

use crate::ide::WorkspaceSymbolsService;

use super::{ServerState, background, is_project_source};

impl ServerState {
    pub(crate) async fn search_workspace_project_symbols(
        &self,
        query: &str,
    ) -> Vec<SymbolInformation> {
        loop {
            let paths = self.current_project_paths().await;
            let Some(revision) = self.documents.stable_revision() else {
                yield_to_source_changes().await;
                continue;
            };
            let mut open = FxHashMap::default();
            for uri in self.documents.uris() {
                if uri
                    .to_file_path()
                    .is_ok_and(|path| is_project_source(&path))
                    && let Some(source) = self.documents.text(&uri)
                {
                    open.insert(uri, source);
                }
            }
            if self.documents.stable_revision() != Some(revision) {
                yield_to_source_changes().await;
                continue;
            }
            let retired = self.workspace_project_files.retired.read().clone();
            let query = query.to_lowercase();
            // Retain one shared open snapshot so a worker failure preserves
            // the original open-buffer-only answer without copying its text.
            let open = Arc::new(open);
            let worker_open = Arc::clone(&open);
            let worker_query = query.clone();
            #[cfg(test)]
            let worker_failed = self
                .workspace_project_files
                .symbol_worker_failure
                .swap(false, std::sync::atomic::Ordering::SeqCst);
            #[cfg(not(test))]
            let worker_failed = false;
            let symbols = if worker_failed {
                None
            } else {
                background(move || {
                    collect_sources(paths.uris, &worker_open, &retired, &worker_query, |path| {
                        std::fs::read_to_string(path).ok()
                    })
                })
                .await
            }
            .unwrap_or_else(|| collect_sources(Vec::new(), &open, &[], &query, |_| None));
            #[cfg(test)]
            {
                let pause = self.workspace_project_files.read_pause.write().take();
                if let Some((reached, resume)) = pause {
                    let _ = reached.send(());
                    let _ = resume.await;
                }
            }
            if self.project_source_snapshot_current(&paths.roots, paths.generation)
                && self.documents.stable_revision() == Some(revision)
            {
                return symbols;
            }
        }
    }
}

fn collect_sources<Text: AsRef<str>>(
    mut uris: Vec<Url>,
    open: &FxHashMap<Url, Text>,
    retired: &[PathBuf],
    query_lower: &str,
    mut read: impl FnMut(&Path) -> Option<Text>,
) -> Vec<SymbolInformation> {
    uris.extend(open.keys().cloned());
    uris.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    uris.dedup();
    let mut symbols = Vec::new();
    for uri in uris {
        if let Some(source) = open.get(&uri) {
            WorkspaceSymbolsService::collect_source(
                &uri,
                source.as_ref(),
                query_lower,
                &mut symbols,
            );
            continue;
        }
        let Ok(path) = uri.to_file_path() else {
            continue;
        };
        if retired.iter().any(|prefix| path.starts_with(prefix)) {
            continue;
        }
        let Some(source) = read(&path) else { continue };
        WorkspaceSymbolsService::collect_source(&uri, source.as_ref(), query_lower, &mut symbols);
        // No closed-file buffer survives into the next file read. Sorting and
        // truncation happen once over the complete result, never per source.
    }
    WorkspaceSymbolsService::rank_sources(symbols, query_lower)
}

async fn yield_to_source_changes() {
    let mut yielded = false;
    futures::future::poll_fn(|context| {
        if yielded {
            Poll::Ready(())
        } else {
            yielded = true;
            context.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await;
}

#[cfg(test)]
mod tests;
