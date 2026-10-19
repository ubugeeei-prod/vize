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
#[path = "import_rewriter_facts_lifetime.rs"]
mod lifetime;

#[cfg(test)]
mod tests {
    //! Syntax reuse must preserve uncached parsing, resolution calls and every offset.

    use std::cell::RefCell;

    use super::*;
    use crate::batch::ImportRewriter;

    fn assert_rewrite_equal(
        actual: &super::super::RewriteResult,
        expected: &super::super::RewriteResult,
    ) {
        assert_eq!(actual.code, expected.code);
        assert_eq!(actual.source_map, expected.source_map);
        for offset in 0..=actual.code.len() as u32 {
            assert_eq!(
                actual.source_map.get_original_offset(offset),
                expected.source_map.get_original_offset(offset),
            );
        }
    }

    #[test]
    fn ordered_views_keep_occurrence_modes_and_reuse_one_parse() {
        use PackageResolutionMode::{Contextual, Import, Require};

        let source = r#"
import A from 'same';
export { A } from 'same';
const required = require('same');
const dynamic = import('same');
type Explicit = import('typed', { with: { 'resolution-mode': 'require' } });
import type { Type } from 'typed' with { 'resolution-mode': 'import' };
"#;
        let source_type = SourceType::ts();
        let mut cache = ModuleSpecifierFactsCache::default();
        let occurrences = cache.collect_occurrences(source, source_type);
        assert_eq!(
            occurrences,
            vec![
                ("same".into(), Contextual),
                ("same".into(), Require),
                ("same".into(), Import),
                ("typed".into(), Require),
                ("typed".into(), Import),
            ],
        );
        assert_eq!(
            occurrences,
            ImportRewriter::new().collect_all_specifier_occurrences(source, source_type),
        );
        assert_eq!(
            cache.collect_all_specifiers(source, source_type),
            vec![String::from("same"), "typed".into()]
        );
        assert_eq!(
            cache.stats(),
            ModuleSpecifierFactsCacheStats {
                hits: 1,
                parses: 1,
                retained_entries: 1
            }
        );
    }

    #[test]
    fn key_includes_exact_bytes_and_every_source_type_dimension() {
        let source = "import value from 'before';";
        let source_types = [
            SourceType::ts(),
            SourceType::tsx(),
            SourceType::d_ts(),
            SourceType::mjs(),
            SourceType::cjs(),
            // ts() is already unambiguous; change only its module kind here.
            SourceType::ts().with_module(true),
        ];
        for (index, source_type) in source_types.iter().enumerate() {
            for previous in &source_types[..index] {
                assert_ne!(
                    source_type, previous,
                    "each case must exercise a distinct key"
                );
            }
        }
        let mut cache = ModuleSpecifierFactsCache::default();
        for source_type in source_types {
            for _ in 0..2 {
                assert_eq!(
                    cache.collect_occurrences(source, source_type),
                    ImportRewriter::new().collect_all_specifier_occurrences(source, source_type),
                );
            }
        }
        assert_eq!(cache.stats().parses, source_types.len());
        assert_eq!(cache.stats().hits, source_types.len());
        let changed = "import value from 'after!';";
        assert_eq!(source.len(), changed.len());
        assert_eq!(
            cache.collect_all_specifiers(changed, SourceType::ts()),
            vec![String::from("after!")]
        );
        assert_eq!(
            cache.collect_all_specifiers(source, SourceType::ts()),
            vec![String::from("before")]
        );
        assert_eq!(cache.stats().parses, source_types.len() + 1);
        // A new preparation cannot inherit the previous invocation's syntax cache.
        assert_eq!(
            ModuleSpecifierFactsCache::default()
                .stats()
                .retained_entries,
            0
        );
    }

    #[test]
    fn cached_rewrite_reinvokes_resolver_for_every_literal_and_preserves_unicode_maps() {
        let source = r#"const label = '日本語🍣';
import A from 'same'; export { A } from 'same';
const required = require('same'); const dynamic = import('same');
import escaped from '\u0073ame';
const sentinel = label;
"#;
        let source_type = SourceType::ts();
        let rewriter = ImportRewriter::new();
        let mut cache = ModuleSpecifierFactsCache::default();
        cache.collect_all_specifiers(source, source_type);
        for target in ["/first/long/module.ts", "/second.ts"] {
            let calls = RefCell::new(Vec::new());
            let resolver = |path: &str, mode| {
                calls.borrow_mut().push((String::from(path), mode));
                Some(target.to_owned())
            };
            let actual = rewriter.rewrite_with_alias_resolver_and_missing_vue_policy_cached(
                source,
                source_type,
                None,
                &resolver,
                true,
                &mut cache,
            );
            assert_eq!(calls.borrow().len(), 5);
            assert_eq!(calls.borrow()[0], calls.borrow()[1]);
            assert_eq!(calls.borrow()[2].1, PackageResolutionMode::Require);
            assert_eq!(calls.borrow()[3].1, PackageResolutionMode::Import);
            assert_eq!(calls.borrow()[4].0, "same");
            let expected = rewriter.rewrite_with_alias_resolver_and_missing_vue_policy(
                source,
                source_type,
                None,
                &|_, _| Some(target.to_owned()),
                true,
            );
            assert_rewrite_equal(&actual, &expected);
            assert_eq!(actual.code.matches(target).count(), 5);
            assert!(actual.code.contains("日本語🍣"));
            let original = source.find("sentinel").unwrap() as u32;
            let generated = actual.code.find("sentinel").unwrap() as u32;
            assert_eq!(actual.source_map.get_virtual_offset(original), generated);
            assert_eq!(actual.source_map.get_original_offset(generated), original);
        }
        for target in [None, Some("/late/target.ts"), None] {
            let calls = RefCell::new(0);
            let resolver = |_: &str, _| {
                *calls.borrow_mut() += 1;
                target.map(str::to_owned)
            };
            let actual = rewriter.rewrite_with_alias_resolver_and_missing_vue_policy_cached(
                source,
                source_type,
                None,
                &resolver,
                true,
                &mut cache,
            );
            assert_eq!(*calls.borrow(), 5);
            let expected = rewriter.rewrite_with_alias_resolver_and_missing_vue_policy(
                source,
                source_type,
                None,
                &|_, _| target.map(str::to_owned),
                true,
            );
            assert_rewrite_equal(&actual, &expected);
            if target.is_none() {
                assert_eq!(actual.code, source);
            }
        }
        assert_eq!(cache.stats().parses, 1);
        assert_eq!(cache.stats().hits, 5);
    }

    #[test]
    fn malformed_partial_ast_and_uncached_control_keep_identical_results() {
        let rewriter = ImportRewriter::new();
        let mut cache = ModuleSpecifierFactsCache::default();
        let mut uncached = ModuleSpecifierFactsCache::uncached();
        for (source, source_type) in [
            (
                "import Early from 'first'; const broken = ; import Late from 'last';",
                SourceType::ts(),
            ),
            (
                "import Early from 'first'; export default 1; export default 2;",
                SourceType::mjs(),
            ),
        ] {
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, source, source_type).parse();
            assert!(parsed.panicked || !parsed.diagnostics.is_empty());
            for _ in 0..2 {
                let expected = rewriter.collect_all_specifier_occurrences(source, source_type);
                if source_type == SourceType::mjs() {
                    assert_eq!(
                        expected,
                        vec![(String::from("first"), PackageResolutionMode::Contextual)]
                    );
                }
                assert_eq!(cache.collect_occurrences(source, source_type), expected);
                assert_eq!(uncached.collect_occurrences(source, source_type), expected);
                let resolver = |_: &str, _| Some("/resolved/longer.ts".to_owned());
                let actual = rewriter.rewrite_with_alias_resolver_and_missing_vue_policy_cached(
                    source,
                    source_type,
                    None,
                    &resolver,
                    true,
                    &mut cache,
                );
                let expected = rewriter.rewrite_with_alias_resolver_and_missing_vue_policy(
                    source,
                    source_type,
                    None,
                    &resolver,
                    true,
                );
                assert_rewrite_equal(&actual, &expected);
            }
        }
        assert_eq!(cache.stats().parses, 2);
        assert_eq!(uncached.stats().parses, 4);
        assert_eq!(uncached.stats().hits, 0);
        assert_eq!(uncached.stats().retained_entries, 0);
    }
}
