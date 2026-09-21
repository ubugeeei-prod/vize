use std::collections::BTreeMap;

use vize_davinci::diagnostic::{Severity, Tier};

use super::{DEMOTED, RULE_CONTRACTS, contract_for};
use crate::rules::css::{
    CssRule, NoDisplayNone, NoHardcodedValues, NoIdSelectors, NoImportant, NoUtilityClasses,
    NoVBindPerformance, PreferLogicalProperties, PreferNestedSelectors, PreferSlotted,
    RequireFontDisplay,
};
use crate::{RuleRegistry, builtin_musea_rules, builtin_script_rules};

fn unified(severity: crate::Severity) -> Severity {
    match severity {
        crate::Severity::Error => Severity::Error,
        crate::Severity::Warning => Severity::Warning,
    }
}

/// Every registered rule and its `default_severity`, from every registry the
/// linter draws on: template rules (all presets plus opt-in), script, CSS
/// and Musea rules.
fn registered() -> BTreeMap<&'static str, Severity> {
    let mut rules = BTreeMap::new();
    for registry in [RuleRegistry::with_all(), RuleRegistry::with_opt_in_rules()] {
        for rule in registry.rules() {
            rules.insert(rule.meta().name, unified(rule.meta().default_severity));
        }
    }
    for meta in builtin_script_rules() {
        rules.insert(meta.name, unified(meta.default_severity));
    }
    let css: [&dyn CssRule; 10] = [
        &NoDisplayNone,
        &NoHardcodedValues::default(),
        &NoIdSelectors,
        &NoImportant,
        &NoUtilityClasses,
        &NoVBindPerformance,
        &PreferLogicalProperties,
        &PreferNestedSelectors,
        &PreferSlotted,
        &RequireFontDisplay,
    ];
    let css_names: Vec<&str> = css.iter().map(|rule| rule.meta().name).collect();
    let mut expected_css = crate::linter::css_rules::all_builtin_css_rule_names().to_vec();
    let mut sorted_css = css_names.clone();
    expected_css.sort_unstable();
    sorted_css.sort_unstable();
    assert_eq!(
        sorted_css, expected_css,
        "the CSS list tracks the CSS registry"
    );
    for rule in css {
        rules.insert(rule.meta().name, unified(rule.meta().default_severity));
    }
    for meta in builtin_musea_rules() {
        rules.insert(meta.name, unified(meta.default_severity));
    }
    rules
}

#[test]
fn the_table_names_exactly_the_registered_rules_in_order() {
    let names: Vec<&str> = RULE_CONTRACTS.iter().map(|entry| entry.name).collect();
    let registered: Vec<&str> = registered().into_keys().collect();
    assert_eq!(names, registered);
    assert_eq!(names.len(), 248);
}

#[test]
fn every_severity_is_the_rules_default_except_the_demoted_heuristics() {
    let registered = registered();
    let mut demoted = Vec::new();
    for entry in RULE_CONTRACTS {
        let default = registered[entry.name];
        if entry.contract.severity() == default {
            continue;
        }
        assert_eq!(
            (default, entry.contract.severity(), entry.contract.tier()),
            (Severity::Error, Severity::Warning, Tier::Heuristic),
            "{}",
            entry.name
        );
        demoted.push(entry.name);
    }
    assert_eq!(demoted, DEMOTED);
}

#[test]
fn every_error_row_and_only_those_reports_under_its_own_exemption() {
    let mut exempt = Vec::new();
    for entry in RULE_CONTRACTS {
        match entry.exemption() {
            Some(exemption) => {
                assert_eq!(entry.contract.severity(), Severity::Error, "{}", entry.name);
                assert_eq!(
                    (exemption.producer(), exemption.code()),
                    ("vize_patina", entry.name)
                );
                exempt.push(entry.name);
            }
            None => assert_eq!(
                entry.contract.severity(),
                Severity::Warning,
                "{}",
                entry.name
            ),
        }
    }
    assert_eq!(exempt.len(), 98);
}

#[test]
fn the_tier_census_is_pinned() {
    let mut census = BTreeMap::new();
    for entry in RULE_CONTRACTS {
        let key = (
            entry.contract.tier().as_str(),
            entry.contract.severity().as_str(),
        );
        *census.entry(key).or_insert(0usize) += 1;
    }
    let census: Vec<((&str, &str), usize)> = census.into_iter().collect();
    assert_eq!(
        census,
        [
            (("complete", "error"), 10),
            (("complete", "warning"), 22),
            (("exact", "error"), 88),
            (("exact", "warning"), 111),
            (("heuristic", "warning"), 17),
        ]
    );
}

#[test]
fn lookups_find_every_rule_and_nothing_else() {
    for entry in RULE_CONTRACTS {
        assert_eq!(
            contract_for(entry.name).map(|found| found.name),
            Some(entry.name)
        );
    }
    for other in ["parser/template", "vue/", "", "vue/valid-v-if "] {
        assert_eq!(
            contract_for(other).map(|found| found.name),
            None,
            "{other:?}"
        );
    }
}
