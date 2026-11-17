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
