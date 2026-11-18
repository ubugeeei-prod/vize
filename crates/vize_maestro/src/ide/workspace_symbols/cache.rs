//! Query-independent symbol indexes for resident and watched sources.

use std::sync::{Arc, Weak};

use tower_lsp::lsp_types::{SymbolInformation, Url};

use super::WorkspaceSymbolsService;

pub(crate) struct CachedDocumentSymbols {
    source: CachedSymbolSource,
    symbols: Arc<[CachedSymbol]>,
}

enum CachedSymbolSource {
    Open(Weak<String>),
    Disk { generation: u64 },
}

struct CachedSymbol {
    lowercase_name: String,
    symbol: SymbolInformation,
}

impl WorkspaceSymbolsService {
    pub(crate) fn collect_cached_open_source(
        cache: &dashmap::DashMap<Url, Arc<CachedDocumentSymbols>>,
        uri: &Url,
        content: &Arc<String>,
        query: &str,
        symbols: &mut Vec<SymbolInformation>,
    ) {
        let cached = if let Some(cached) = cache.get(uri)
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
        };
        cached.collect(query, symbols);
    }

    /// Reuse a closed-file index only for the exact watched disk generation.
    /// `None` probes the cache before I/O; `Some` builds after a cold read.
    #[cfg(feature = "native")]
    pub(crate) fn collect_cached_disk_source(
        cache: &dashmap::DashMap<Url, Arc<CachedDocumentSymbols>>,
        uri: &Url,
        source: Option<&str>,
        generation: u64,
        query: &str,
        symbols: &mut Vec<SymbolInformation>,
    ) -> bool {
        let cached = if let Some(cached) = cache.get(uri)
            && cached.matches_disk(generation)
        {
            Arc::clone(cached.value())
        } else {
            let Some(source) = source else { return false };
            let candidate =
                Self::index_source(uri, source, CachedSymbolSource::Disk { generation });
            Self::publish_cached_source(cache, uri, candidate, |cached| {
                cached.matches_disk(generation)
            })
        };
        cached.collect(query, symbols);
        true
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
}

impl CachedDocumentSymbols {
    fn matches_open(&self, content: &Arc<String>) -> bool {
        let CachedSymbolSource::Open(source) = &self.source else {
            return false;
        };
        source
            .upgrade()
            .is_some_and(|source| Arc::ptr_eq(&source, content))
    }

    fn matches_disk(&self, generation: u64) -> bool {
        matches!(
            self.source,
            CachedSymbolSource::Disk {
                generation: cached
            } if cached == generation
        )
    }

    fn collect(&self, query: &str, symbols: &mut Vec<SymbolInformation>) {
        symbols.extend(
            self.symbols
                .iter()
                .filter(|symbol| symbol.lowercase_name.contains(query))
                .map(|symbol| symbol.symbol.clone()),
        );
    }
}
