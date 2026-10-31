//! Distinguish dependency requests from actual syntax-fact parser inputs.

use super::{ModuleSpecifierFactsCache, SourceType};

impl ModuleSpecifierFactsCache {
    pub(crate) fn observe_dependency_parse(&self, source: &str, source_type: SourceType) {
        // Match facts()'s complete key without changing or priming the cache.
        // A hit supplies owned syntax facts and must not count as a parse.
        let hit = self
            .sources
            .get(&source_type)
            .is_some_and(|sources| sources.contains_key(source));
        #[cfg(test)]
        let hit = hit && !self.uncached;
        if hit {
            vize_carton::micro_count!("canon.dependency_scan.occurrences.cache_hit.calls", 1);
        } else {
            vize_carton::micro_count!("canon.dependency_scan.occurrences.cache_miss.calls", 1);
            // The caller immediately invokes collect_occurrences; a missing
            // exact key takes ModuleSpecifierFacts::parse on these same bytes.
            crate::profiling::observe_dependency_input(source, source_type);
        }
    }
}
