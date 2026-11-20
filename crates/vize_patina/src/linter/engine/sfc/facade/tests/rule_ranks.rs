//! The facade's memoized rule ranks select exactly what a full registry scan does.

use super::super::RULES;
use crate::LintPreset;
use crate::linter::Linter;

fn scanned(linter: &Linter, count: usize) -> Vec<usize> {
    linter
        .markup_rules(count, |name, _| RULES.contains(&name))
        .iter()
        .map(|rule| rule.rank())
        .collect()
}

fn ranked(linter: &Linter, count: usize) -> Vec<usize> {
    linter
        .named_markup_rules(count, RULES)
        .iter()
        .map(|rule| rule.rank())
        .collect()
}

#[test]
fn memoized_ranks_match_the_registry_scan() {
    let linters = [
        Linter::with_preset(LintPreset::HappyPath),
        Linter::with_preset(LintPreset::Opinionated),
        Linter::with_preset(LintPreset::Ecosystem),
        Linter::with_all(),
        Linter::with_preset(LintPreset::HappyPath)
            .with_disabled_rules(vec!["vue/no-multi-spaces".into(), "a11y/img-alt".into()]),
        Linter::with_preset(LintPreset::HappyPath)
            .with_enabled_rules(Some(vec!["vue/no-textarea-mustache".into()])),
    ];
    for linter in &linters {
        let total = linter.registry.rules().len();
        for count in [0, 1, total / 2, total, usize::MAX] {
            // The first call computes the memo; the second reads it.
            assert_eq!(
                ranked(linter, count),
                scanned(linter, count),
                "count {count}"
            );
            assert_eq!(
                ranked(linter, count),
                scanned(linter, count),
                "count {count}"
            );
        }
    }
}

#[test]
fn registering_a_rule_refreshes_the_memoized_ranks() {
    let linter = Linter::with_preset(LintPreset::Incremental);
    assert!(ranked(&linter, usize::MAX).is_empty());
    let linter = linter.with_rule(Box::new(crate::rules::vue::NoTextareaMustache));
    assert_eq!(ranked(&linter, usize::MAX), scanned(&linter, usize::MAX));
    assert_eq!(ranked(&linter, usize::MAX).len(), 1);
}

#[test]
fn memoized_legacy_plan_matches_fresh_indices_and_hooks() {
    for linter in [
        Linter::with_preset(LintPreset::HappyPath),
        Linter::with_preset(LintPreset::Opinionated),
        Linter::with_preset(LintPreset::Ecosystem),
        Linter::with_all(),
    ] {
        let rules = linter.registry.rules();
        for count in [0, rules.len() / 2, rules.len(), usize::MAX] {
            let facade = ranked(&linter, count);
            let bounded = count.min(rules.len());
            let indices = (0..bounded)
                .filter(|rank| facade.binary_search(rank).is_err())
                .collect::<Vec<_>>();
            let hooks = crate::rule::IndexedRulePlans::new(rules, &indices);
            let plan = linter.registry.legacy_template_plan_named(count, RULES);
            assert_eq!(plan.indices, indices, "count {count}");
            assert_eq!(plan.hooks, hooks, "count {count}");
        }
    }
}

#[test]
fn registering_a_rule_refreshes_the_memoized_legacy_plan() {
    let linter = Linter::with_preset(LintPreset::Incremental);
    assert!(
        linter
            .registry
            .legacy_template_plan_named(usize::MAX, RULES)
            .indices
            .is_empty()
    );
    let linter = linter.with_rule(Box::new(crate::rules::vue::ValidVIf));
    assert_eq!(
        linter
            .registry
            .legacy_template_plan_named(usize::MAX, RULES)
            .indices,
        [0]
    );
}
