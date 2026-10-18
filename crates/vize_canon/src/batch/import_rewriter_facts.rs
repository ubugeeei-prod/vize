//! Owned syntax facts shared only within one preparation invocation.

use oxc_allocator::Allocator;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashMap, FxHashSet, String};

use super::collect::ModuleSpecifierCollector;
use crate::PackageResolutionMode;

type SpanOccurrence = (u32, u32, String, PackageResolutionMode);

struct ModuleSpecifierFacts {
    spans: Vec<SpanOccurrence>,
    occurrences: Vec<(String, PackageResolutionMode)>,
    specifiers: Vec<String>,
}

impl ModuleSpecifierFacts {
    fn parse(source: &str, source_type: SourceType) -> Self {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, source_type).parse();
        let mut collector = ModuleSpecifierCollector::new();
        // Match uncached collectors, including their partial-AST behavior.
        collector.visit_program(&parsed.program);
        let mut occurrences = Vec::new();
        let mut specifiers = Vec::new();
        let mut seen_occurrences = FxHashSet::default();
        let mut seen_specifiers = FxHashSet::default();
        for (_, _, specifier, mode) in &collector.specifiers {
            if seen_occurrences.insert((specifier.clone(), *mode)) {
                occurrences.push((specifier.clone(), *mode));
            }
            if seen_specifiers.insert(specifier.clone()) {
                specifiers.push(specifier.clone());
            }
        }
        Self {
            spans: collector.specifiers,
            occurrences,
            specifiers,
        }
    }
}

/// Callers own this cache for one preparation, never a session or native query.
/// Complete SourceType and exact source bytes determine syntax; no paths,
/// resolution decisions, callbacks, or rewritten output are retained.
#[cfg_attr(not(test), derive(Default))]
pub(crate) struct ModuleSpecifierFactsCache {
    sources: FxHashMap<SourceType, FxHashMap<String, ModuleSpecifierFacts>>,
    #[cfg(test)]
    uncached: bool,
    #[cfg(test)]
    scratch: Option<ModuleSpecifierFacts>,
    #[cfg(test)]
    hits: usize,
    #[cfg(test)]
    parses: usize,
    #[cfg(test)]
    _observed_lifetime: Option<lifetime::ObservedLifetime>,
}

#[cfg(test)]
impl Default for ModuleSpecifierFactsCache {
    fn default() -> Self {
        Self {
            sources: FxHashMap::default(),
            uncached: false,
            scratch: None,
            hits: 0,
            parses: 0,
            _observed_lifetime: lifetime::created(),
        }
    }
}

impl ModuleSpecifierFactsCache {
    pub(crate) fn collect_occurrences(
        &mut self,
        source: &str,
        source_type: SourceType,
    ) -> Vec<(String, PackageResolutionMode)> {
        self.facts(source, source_type).occurrences.clone()
    }

    pub(crate) fn collect_all_specifiers(
        &mut self,
        source: &str,
        source_type: SourceType,
    ) -> Vec<String> {
        self.facts(source, source_type).specifiers.clone()
    }

    pub(super) fn span_occurrences(
        &mut self,
        source: &str,
        source_type: SourceType,
    ) -> &[SpanOccurrence] {
        &self.facts(source, source_type).spans
    }

    fn facts(&mut self, source: &str, source_type: SourceType) -> &ModuleSpecifierFacts {
        #[cfg(test)]
        if self.uncached {
            self.parses += 1;
            return self
                .scratch
                .insert(ModuleSpecifierFacts::parse(source, source_type));
        }
        let sources = self.sources.entry(source_type).or_default();
        if !sources.contains_key(source) {
            sources.insert(
                source.into(),
                ModuleSpecifierFacts::parse(source, source_type),
            );
            #[cfg(test)]
            {
                self.parses += 1;
            }
        } else {
            #[cfg(test)]
            {
                self.hits += 1;
            }
        }
        &sources[source]
    }

    #[cfg(test)]
    pub(crate) fn observe_lifetimes() -> lifetime::Observation {
        lifetime::observe()
    }

    #[cfg(test)]
    pub(crate) fn assert_no_live_preparation_facts() {
        lifetime::checkpoint();
    }

    #[cfg(test)]
    pub(crate) fn uncached() -> Self {
        Self {
            uncached: true,
            ..Self::default()
        }
    }

    #[cfg(test)]
    pub(crate) fn stats(&self) -> ModuleSpecifierFactsCacheStats {
        ModuleSpecifierFactsCacheStats {
            hits: self.hits,
            parses: self.parses,
            retained_entries: self.sources.values().map(FxHashMap::len).sum(),
        }
    }
}

#[cfg(test)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ModuleSpecifierFactsCacheStats {
    pub(crate) hits: usize,
    pub(crate) parses: usize,
    pub(crate) retained_entries: usize,
}

#[cfg(test)]
#[path = "import_rewriter_facts_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "import_rewriter_facts_lifetime.rs"]
mod lifetime;
