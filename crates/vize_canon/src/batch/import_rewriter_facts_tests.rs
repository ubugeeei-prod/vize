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
