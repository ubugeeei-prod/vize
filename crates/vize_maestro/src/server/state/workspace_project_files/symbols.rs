//! Read, collect and release one closed source at a time on the existing worker.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::task::Poll;

use crate::ide::WorkspaceSymbolsService;
use tower_lsp::lsp_types::{SymbolInformation, Url};

use super::{ServerState, is_project_source};

#[derive(Clone, Copy)]
struct SymbolCacheSnapshot<'a> {
    documents: &'a dashmap::DashMap<Url, Arc<crate::ide::workspace_symbols::CachedDocumentSymbols>>,
    disk_generation: Option<u64>,
}

impl ServerState {
    #[cfg(test)]
    pub(crate) async fn search_workspace_project_symbols(
        &self,
        query: &str,
    ) -> Vec<SymbolInformation> {
        self.search_workspace_project_symbols_filtered(query, |_| true)
            .await
    }

    pub(crate) async fn search_workspace_project_symbols_filtered(
        &self,
        query: &str,
        filter: impl Fn(&Url) -> bool + Clone + Send + 'static,
    ) -> Vec<SymbolInformation> {
        loop {
            let paths = self.current_project_paths().await;
            let Some(revision) = self.documents.stable_revision() else {
                yield_to_source_changes().await;
                continue;
            };
            let open = self.documents.texts();
            if self.documents.stable_revision() != Some(revision) {
                yield_to_source_changes().await;
                continue;
            }
            let content_generation = self
                .workspace_project_files
                .content_generation
                .load(Ordering::Acquire);
            let disk_cache_generation = self
                .corsa_disk_watcher_registered
                .load(Ordering::Acquire)
                .then_some(content_generation);
            let retired = self.workspace_project_files.retired.read().clone();
            let query = query.to_lowercase();
            // Retain one shared open snapshot so a worker failure preserves
            // the original open-buffer-only answer without copying its text.
            let worker_open = Arc::clone(&open);
            let worker_uris = Arc::clone(&paths.uris);
            let worker_query = query.clone();
            let worker_filter = filter.clone();
            let worker_symbol_cache = Arc::clone(&self.workspace_symbol_cache);
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
                self.workspace_project_files
                    .worker
                    .run(move || {
                        collect_sources_filtered(
                            &worker_uris,
                            &worker_open,
                            &retired,
                            &worker_query,
                            |path| std::fs::read_to_string(path).ok(),
                            worker_filter,
                            Some(SymbolCacheSnapshot {
                                documents: &worker_symbol_cache,
                                disk_generation: disk_cache_generation,
                            }),
                        )
                    })
                    .await
            }
            .unwrap_or_else(|| {
                collect_sources_filtered(
                    &[],
                    &open,
                    &[],
                    &query,
                    |_| Option::<String>::None,
                    filter.clone(),
                    Some(SymbolCacheSnapshot {
                        documents: &self.workspace_symbol_cache,
                        disk_generation: None,
                    }),
                )
            });
            #[cfg(test)]
            {
                let pause = self.workspace_project_files.read_pause.write().take();
                if let Some((reached, resume)) = pause {
                    let _ = reached.send(());
                    let _ = resume.await;
                }
            }
            if self.project_source_content_snapshot_current(
                &paths.roots,
                paths.generation,
                content_generation,
            ) && self.documents.stable_revision() == Some(revision)
            {
                return symbols;
            }
        }
    }
}

#[cfg(test)]
fn collect_sources<Text: AsRef<str>>(
    mut uris: Vec<Url>,
    open: &[(Url, Arc<String>)],
    retired: &[PathBuf],
    query_lower: &str,
    read: impl FnMut(&Path) -> Option<Text>,
) -> Vec<SymbolInformation> {
    uris.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    uris.dedup();
    collect_sources_filtered(&uris, open, retired, query_lower, read, |_| true, None)
}

fn collect_sources_filtered<Text: AsRef<str>>(
    uris: &[Url],
    open: &[(Url, Arc<String>)],
    retired: &[PathBuf],
    query_lower: &str,
    mut read: impl FnMut(&Path) -> Option<Text>,
    filter: impl Fn(&Url) -> bool,
    symbol_cache: Option<SymbolCacheSnapshot<'_>>,
) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();
    let mut disk_index = 0;
    let mut open_index = 0;
    while disk_index < uris.len() || open_index < open.len() {
        while open
            .get(open_index)
            .is_some_and(|(uri, _)| !is_project_uri(uri))
        {
            open_index += 1;
        }
        match (uris.get(disk_index), open.get(open_index)) {
            (Some(disk_uri), Some((open_uri, open_source))) => {
                match disk_uri.as_str().cmp(open_uri.as_str()) {
                    std::cmp::Ordering::Less => {
                        collect_disk_source(
                            disk_uri,
                            retired,
                            query_lower,
                            &mut read,
                            &filter,
                            symbol_cache,
                            &mut symbols,
                        );
                        disk_index += 1;
                    }
                    std::cmp::Ordering::Equal => {
                        collect_open_source(
                            open_uri,
                            open_source,
                            query_lower,
                            &filter,
                            symbol_cache,
                            &mut symbols,
                        );
                        disk_index += 1;
                        open_index += 1;
                    }
                    std::cmp::Ordering::Greater => {
                        collect_open_source(
                            open_uri,
                            open_source,
                            query_lower,
                            &filter,
                            symbol_cache,
                            &mut symbols,
                        );
                        open_index += 1;
                    }
                }
            }
            (Some(disk_uri), None) => {
                collect_disk_source(
                    disk_uri,
                    retired,
                    query_lower,
                    &mut read,
                    &filter,
                    symbol_cache,
                    &mut symbols,
                );
                disk_index += 1;
            }
            (None, Some((open_uri, open_source))) => {
                collect_open_source(
                    open_uri,
                    open_source,
                    query_lower,
                    &filter,
                    symbol_cache,
                    &mut symbols,
                );
                open_index += 1;
            }
            (None, None) => break,
        }
    }
    WorkspaceSymbolsService::rank_sources(symbols, query_lower)
}

fn collect_open_source(
    uri: &Url,
    source: &Arc<String>,
    query_lower: &str,
    filter: &impl Fn(&Url) -> bool,
    symbol_cache: Option<SymbolCacheSnapshot<'_>>,
    symbols: &mut Vec<SymbolInformation>,
) {
    let before = symbols.len();
    if let Some(cache) = symbol_cache {
        WorkspaceSymbolsService::collect_cached_open_source(
            cache.documents,
            uri,
            source,
            query_lower,
            symbols,
        );
    } else {
        WorkspaceSymbolsService::collect_source(uri, source.as_ref(), query_lower, symbols);
    }
    if symbols.len() > before && !filter(uri) {
        symbols.truncate(before);
    }
}

fn collect_disk_source<Text: AsRef<str>>(
    uri: &Url,
    retired: &[PathBuf],
    query_lower: &str,
    read: &mut impl FnMut(&Path) -> Option<Text>,
    filter: &impl Fn(&Url) -> bool,
    symbol_cache: Option<SymbolCacheSnapshot<'_>>,
    symbols: &mut Vec<SymbolInformation>,
) {
    let Ok(path) = uri.to_file_path() else {
        return;
    };
    if retired.iter().any(|prefix| path.starts_with(prefix)) {
        return;
    }
    let before = symbols.len();
    if let Some(SymbolCacheSnapshot {
        documents: cache,
        disk_generation: Some(generation),
    }) = symbol_cache
        && WorkspaceSymbolsService::collect_cached_disk_source(
            cache,
            uri,
            None,
            generation,
            query_lower,
            symbols,
        )
    {
        if symbols.len() > before && !filter(uri) {
            symbols.truncate(before);
        }
        return;
    }
    let Some(source) = read(&path) else { return };
    if let Some(SymbolCacheSnapshot {
        documents: cache,
        disk_generation: Some(generation),
    }) = symbol_cache
    {
        WorkspaceSymbolsService::collect_cached_disk_source(
            cache,
            uri,
            Some(source.as_ref()),
            generation,
            query_lower,
            symbols,
        );
    } else {
        WorkspaceSymbolsService::collect_source(uri, source.as_ref(), query_lower, symbols);
    }
    if symbols.len() > before && !filter(uri) {
        symbols.truncate(before);
    }
    // No closed-file buffer survives into the next file read. Sorting and
    // truncation happen once over the complete result, never per source.
}

fn is_project_uri(uri: &Url) -> bool {
    uri.to_file_path()
        .is_ok_and(|path| is_project_source(&path))
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

#[cfg(all(test, feature = "micro-profiling"))]
mod allocation_tests;
#[cfg(test)]
mod filtered_tests;
#[cfg(test)]
mod tests;
