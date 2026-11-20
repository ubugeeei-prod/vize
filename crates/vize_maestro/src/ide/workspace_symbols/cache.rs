//! Query-independent symbol indexes for resident and watched sources.

use std::sync::{Arc, Weak};

use tower_lsp::lsp_types::{SymbolInformation, Url};

#[cfg(all(test, feature = "micro-profiling"))]
use super::rank;
use super::{WorkspaceSymbolsService, rank::SymbolSelection};

pub(crate) struct CachedDocumentSymbols {
    source: CachedSymbolSource,
    symbols: Arc<[CachedSymbol]>,
}

enum CachedSymbolSource {
    Open(Weak<String>),
    #[cfg(feature = "native")]
    Disk {
        generation: u64,
    },
}

struct CachedSymbol {
    lowercase_name: String,
    symbol: SymbolInformation,
}

impl WorkspaceSymbolsService {
    pub(crate) fn select_cached_open_source(
        cache: &dashmap::DashMap<Url, Arc<CachedDocumentSymbols>>,
        uri: &Url,
        content: &Arc<String>,
        include: impl FnOnce() -> bool,
        selection: &mut SymbolSelection<'_>,
    ) {
        let cached = Self::cached_open_source(cache, uri, content);
        Self::select_cached_source(cached, include, selection);
    }

    /// Reuse a closed-file index only for the exact watched disk generation.
    /// `None` probes the cache before I/O; `Some` builds after a cold read.
    #[cfg(feature = "native")]
    pub(crate) fn select_cached_disk_source(
        cache: &dashmap::DashMap<Url, Arc<CachedDocumentSymbols>>,
        uri: &Url,
        source: Option<&str>,
        generation: u64,
        include: impl FnOnce() -> bool,
        selection: &mut SymbolSelection<'_>,
    ) -> bool {
        let Some(cached) = Self::cached_disk_source(cache, uri, source, generation) else {
            return false;
        };
        Self::select_cached_source(cached, include, selection);
        true
    }

    fn select_cached_source(
        cached: Arc<CachedDocumentSymbols>,
        include: impl FnOnce() -> bool,
        selection: &mut SymbolSelection<'_>,
    ) {
        let Some(first_match) = cached.first_match(selection.query()) else {
            return;
        };
        if include() {
            selection.consider_cached_from(cached, first_match);
        }
    }

    fn cached_open_source(
        cache: &dashmap::DashMap<Url, Arc<CachedDocumentSymbols>>,
        uri: &Url,
        content: &Arc<String>,
    ) -> Arc<CachedDocumentSymbols> {
        if let Some(cached) = cache.get(uri)
            && cached.matches_open(content)
        {
            Arc::clone(cached.value())
        } else {
            let candidate = Self::index_source(
                uri,
                content,
                CachedSymbolSource::Open(Arc::downgrade(content)),
            );
            Self::publish_cached_source(cache, uri, candidate, |cached| {
                cached.matches_open(content)
            })
        }
    }

    #[cfg(feature = "native")]
    fn cached_disk_source(
        cache: &dashmap::DashMap<Url, Arc<CachedDocumentSymbols>>,
        uri: &Url,
        source: Option<&str>,
        generation: u64,
    ) -> Option<Arc<CachedDocumentSymbols>> {
        if let Some(cached) = cache.get(uri)
            && cached.matches_disk(generation)
        {
            Some(Arc::clone(cached.value()))
        } else {
            let source = source?;
            let candidate =
                Self::index_source(uri, source, CachedSymbolSource::Disk { generation });
            Some(Self::publish_cached_source(
                cache,
                uri,
                candidate,
                |cached| cached.matches_disk(generation),
            ))
        }
    }

    fn index_source(
        uri: &Url,
        content: &str,
        source: CachedSymbolSource,
    ) -> Arc<CachedDocumentSymbols> {
        let mut discovered = Vec::new();
        #[cfg(feature = "native")]
        Self::collect_source(uri, content, "", &mut discovered);
        #[cfg(not(feature = "native"))]
        Self::collect_symbols_from_document(uri, content, "", &mut discovered);
        let discovered = discovered
            .into_iter()
            .map(|symbol| CachedSymbol {
                lowercase_name: symbol.name.to_lowercase(),
                symbol,
            })
            .collect::<Vec<_>>();
        Arc::new(CachedDocumentSymbols {
            source,
            symbols: Arc::from(discovered),
        })
    }

    fn publish_cached_source(
        cache: &dashmap::DashMap<Url, Arc<CachedDocumentSymbols>>,
        uri: &Url,
        candidate: Arc<CachedDocumentSymbols>,
        matches: impl Fn(&CachedDocumentSymbols) -> bool,
    ) -> Arc<CachedDocumentSymbols> {
        match cache.entry(uri.clone()) {
            dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                if matches(entry.get()) {
                    Arc::clone(entry.get())
                } else {
                    entry.insert(Arc::clone(&candidate));
                    candidate
                }
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                entry.insert(Arc::clone(&candidate));
                candidate
            }
        }
    }

    #[cfg(all(test, feature = "micro-profiling"))]
    pub(crate) fn search_cached_open_sources_legacy(
        state: &crate::server::ServerState,
        query: &str,
    ) -> Vec<SymbolInformation> {
        let query = query.to_lowercase();
        let mut symbols = Vec::new();
        for (uri, content) in state.documents.vue_texts().iter() {
            Self::cached_open_source(&state.workspace_symbol_cache, uri, content)
                .collect_legacy(&query, &mut symbols);
        }
        rank::legacy_sort(symbols, &query)
    }
}

impl CachedDocumentSymbols {
    pub(super) fn len(&self) -> usize {
        self.symbols.len()
    }

    pub(super) fn lowercase_name(&self, index: usize) -> Option<&str> {
        self.symbols
            .get(index)
            .map(|symbol| symbol.lowercase_name.as_str())
    }

    pub(super) fn symbol(&self, index: usize) -> Option<&SymbolInformation> {
        self.symbols.get(index).map(|symbol| &symbol.symbol)
    }

    fn first_match(&self, query: &str) -> Option<usize> {
        self.symbols
            .iter()
            .position(|symbol| symbol.lowercase_name.contains(query))
    }

    fn matches_open(&self, content: &Arc<String>) -> bool {
        match &self.source {
            CachedSymbolSource::Open(source) => source
                .upgrade()
                .is_some_and(|source| Arc::ptr_eq(&source, content)),
            #[cfg(feature = "native")]
            CachedSymbolSource::Disk { .. } => false,
        }
    }

    #[cfg(feature = "native")]
    fn matches_disk(&self, generation: u64) -> bool {
        matches!(
            self.source,
            CachedSymbolSource::Disk {
                generation: cached
            } if cached == generation
        )
    }

    #[cfg(all(test, feature = "micro-profiling"))]
    fn collect_legacy(&self, query: &str, symbols: &mut Vec<SymbolInformation>) {
        symbols.extend(
            self.symbols
                .iter()
                .filter(|symbol| symbol.lowercase_name.contains(query))
                .map(|symbol| symbol.symbol.clone()),
        );
    }
}
