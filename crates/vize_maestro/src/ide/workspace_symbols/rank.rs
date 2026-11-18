use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;

use tower_lsp::lsp_types::SymbolInformation;

use super::CachedDocumentSymbols;

const SYMBOL_LIMIT: usize = 100;

/// Bounded relevance selection for one workspace-symbol request.
///
/// Cached candidates keep their index owner alive but do not clone the wire
/// symbol until they survive the global top-100 selection. The heap root is
/// the worst retained candidate, so replacing it keeps memory bounded while
/// preserving the prior stable full-sort order exactly.
pub(crate) struct SymbolSelection<'query> {
    query: &'query str,
    candidates: BinaryHeap<RankedSymbol>,
    next_ordinal: usize,
}

enum SymbolSource {
    Cached {
        document: Arc<CachedDocumentSymbols>,
        index: usize,
    },
    #[cfg(any(feature = "native", test))]
    Owned {
        symbol: Box<SymbolInformation>,
        lowercase_name: String,
    },
}

struct RankedSymbol {
    source: SymbolSource,
    class: u8,
    ordinal: usize,
}

impl<'query> SymbolSelection<'query> {
    pub(crate) fn new(query: &'query str) -> Self {
        Self {
            query,
            candidates: BinaryHeap::new(),
            next_ordinal: 0,
        }
    }

    pub(crate) fn query(&self) -> &'query str {
        self.query
    }

    pub(crate) fn consider_cached_from(
        &mut self,
        document: Arc<CachedDocumentSymbols>,
        first_match: usize,
    ) {
        for index in first_match..document.len() {
            let Some(lowercase_name) = document.lowercase_name(index) else {
                continue;
            };
            let Some(class) = match_class(lowercase_name, self.query) else {
                continue;
            };
            let ordinal = self.take_ordinal();
            if !self.should_retain(class, lowercase_name, ordinal) {
                continue;
            }
            self.insert(RankedSymbol {
                source: SymbolSource::Cached {
                    document: Arc::clone(&document),
                    index,
                },
                class,
                ordinal,
            });
        }
    }

    #[cfg(any(feature = "native", test))]
    pub(crate) fn extend_owned(&mut self, symbols: impl IntoIterator<Item = SymbolInformation>) {
        for symbol in symbols {
            let lowercase_name = symbol.name.to_lowercase();
            let Some(class) = match_class(&lowercase_name, self.query) else {
                continue;
            };
            let ordinal = self.take_ordinal();
            if !self.should_retain(class, &lowercase_name, ordinal) {
                continue;
            }
            self.insert(RankedSymbol {
                source: SymbolSource::Owned {
                    symbol: Box::new(symbol),
                    lowercase_name,
                },
                class,
                ordinal,
            });
        }
    }

    pub(crate) fn finish(self) -> Vec<SymbolInformation> {
        let mut candidates = self.candidates.into_vec();
        candidates.sort_unstable();
        candidates
            .into_iter()
            .filter_map(RankedSymbol::into_symbol)
            .collect()
    }

    fn take_ordinal(&mut self) -> usize {
        let ordinal = self.next_ordinal;
        self.next_ordinal = self.next_ordinal.wrapping_add(1);
        ordinal
    }

    fn should_retain(&self, class: u8, lowercase_name: &str, ordinal: usize) -> bool {
        self.candidates.len() < SYMBOL_LIMIT
            || self.candidates.peek().is_some_and(|worst| {
                class
                    .cmp(&worst.class)
                    .then_with(|| lowercase_name.cmp(worst.lowercase_name()))
                    .then_with(|| ordinal.cmp(&worst.ordinal))
                    .is_lt()
            })
    }

    fn insert(&mut self, candidate: RankedSymbol) {
        if self.candidates.len() == SYMBOL_LIMIT {
            let _ = self.candidates.pop();
        } else if self.candidates.len() == self.candidates.capacity()
            && self.candidates.len() >= SYMBOL_LIMIT / 2
        {
            // Preserve cheap sparse queries, then avoid growing the final
            // high-hit heap from 64 slots to the allocator's 128-slot step.
            self.candidates
                .reserve_exact(SYMBOL_LIMIT - self.candidates.len());
        }
        self.candidates.push(candidate);
    }
}

impl RankedSymbol {
    fn lowercase_name(&self) -> &str {
        match &self.source {
            SymbolSource::Cached { document, index } => {
                document.lowercase_name(*index).unwrap_or_default()
            }
            #[cfg(any(feature = "native", test))]
            SymbolSource::Owned { lowercase_name, .. } => lowercase_name,
        }
    }

    fn into_symbol(self) -> Option<SymbolInformation> {
        match self.source {
            SymbolSource::Cached { document, index } => document.symbol(index).cloned(),
            #[cfg(any(feature = "native", test))]
            SymbolSource::Owned { symbol, .. } => Some(*symbol),
        }
    }
}

impl PartialEq for RankedSymbol {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for RankedSymbol {}

impl PartialOrd for RankedSymbol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RankedSymbol {
    fn cmp(&self, other: &Self) -> Ordering {
        self.class
            .cmp(&other.class)
            .then_with(|| self.lowercase_name().cmp(other.lowercase_name()))
            .then_with(|| self.ordinal.cmp(&other.ordinal))
    }
}

fn match_class(lowercase_name: &str, query: &str) -> Option<u8> {
    if lowercase_name == query {
        Some(0)
    } else if lowercase_name.starts_with(query) {
        Some(1)
    } else if lowercase_name.contains(query) {
        Some(2)
    } else {
        None
    }
}

#[cfg(test)]
pub(super) fn sort(symbols: Vec<SymbolInformation>, query_lower: &str) -> Vec<SymbolInformation> {
    let mut selection = SymbolSelection::new(query_lower);
    selection.extend_owned(symbols);
    selection.finish()
}

#[cfg(all(test, feature = "micro-profiling"))]
pub(super) fn legacy_sort(
    mut symbols: Vec<SymbolInformation>,
    query_lower: &str,
) -> Vec<SymbolInformation> {
    let mut ranked = Vec::with_capacity(symbols.len());
    ranked.extend(
        symbols
            .drain(..)
            .map(|symbol| (symbol.name.to_lowercase(), symbol)),
    );
    ranked.sort_by(|(left, _), (right, _)| {
        match_class(left, query_lower)
            .cmp(&match_class(right, query_lower))
            .then_with(|| left.cmp(right))
    });
    ranked.truncate(SYMBOL_LIMIT);
    symbols.extend(ranked.into_iter().map(|(_, symbol)| symbol));
    symbols
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::{Location, Position, Range, SymbolKind, Url};

    use super::*;

    #[expect(deprecated, reason = "wire fixture uses SymbolInformation")]
    fn symbol(name: &str) -> SymbolInformation {
        SymbolInformation {
            name: name.into(),
            kind: SymbolKind::VARIABLE,
            tags: None,
            deprecated: None,
            location: Location {
                uri: Url::parse("file:///workspace/App.vue").unwrap(),
                range: Range::new(Position::new(0, 0), Position::new(0, 0)),
            },
            container_name: None,
        }
    }

    fn full_sort_reference(
        mut symbols: Vec<SymbolInformation>,
        query: &str,
    ) -> Vec<SymbolInformation> {
        symbols.sort_by_cached_key(|symbol| {
            let lowercase = symbol.name.to_lowercase();
            (match_class(&lowercase, query), lowercase)
        });
        symbols.truncate(SYMBOL_LIMIT);
        symbols
    }

    #[test]
    fn relevance_order_matches_the_prior_stable_sort() {
        let names = sort(
            vec![
                symbol("BetaCount"),
                symbol("counter"),
                symbol("COUNT"),
                symbol("account"),
                symbol("Count"),
            ],
            "count",
        )
        .into_iter()
        .map(|symbol| symbol.name)
        .collect::<Vec<_>>();
        assert_eq!(names, ["COUNT", "Count", "counter", "account", "BetaCount"]);
    }

    #[test]
    fn bounded_heap_preserves_stable_ties_past_the_wire_limit() {
        let symbols = (0..240)
            .map(|index| symbol(&vize_l0::cstr!("target_{:03}", index % 120)))
            .collect::<Vec<_>>();
        let expected = full_sort_reference(symbols.clone(), "target");
        assert_eq!(sort(symbols, "target"), expected);
    }
}
