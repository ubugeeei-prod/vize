//! Fixture coverage for the `eslint-plugin-vue` to Patina rule map.

use super::LintPreset;
use crate::Severity;
use crate::rule::RuleRegistry;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn eslint_vue_rule_map_matches_registered_patina_rules() {
    let rule_map: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/patina-eslint-vue-rule-map.json"
    ))
    .unwrap();
    let mappings = rule_map["entries"].as_object().unwrap();

    let mut available: BTreeSet<_> = RuleRegistry::with_all()
        .rule_names()
        .iter()
        .copied()
        .collect();
    available.extend(
        RuleRegistry::with_opt_in_rules()
            .rule_names()
            .iter()
            .copied(),
    );
    available.extend(
        crate::linter::script_rules::all_builtin_script_rule_names()
            .iter()
            .copied(),
    );

    for (eslint_rule, entry) in mappings {
        if entry["status"] == "mapped" {
            let target = entry["patinaRule"].as_str().unwrap();
            assert!(
                available.contains(target),
                "{eslint_rule} maps to unavailable Patina rule {target}"
            );
            continue;
        }

        if entry["status"] == "intentional-divergence" {
            // Patina ships a same-named rule on purpose, with semantics that
            // deliberately differ from upstream, so a registered counterpart is
            // expected here. Require the documented reason and the counterpart
            // so the entry has to be revisited if either disappears.
            let reason = entry["reason"].as_str().unwrap_or_default();
            assert!(
                !reason.trim().is_empty(),
                "{eslint_rule} is marked intentional-divergence without a reason"
            );
            let script_rule = eslint_rule.replacen("vue/", "script/", 1);
            assert!(
                available.contains(eslint_rule.as_str())
                    || available.contains(script_rule.as_str()),
                "{eslint_rule} is marked intentional-divergence without a registered Patina counterpart"
            );
            continue;
        }

        let script_rule = eslint_rule.replacen("vue/", "script/", 1);
        let alias = match eslint_rule.as_str() {
            "vue/attributes-order" => Some("vue/attribute-order"),
            "vue/block-order" => Some("vue/sfc-element-order"),
            "vue/no-async-in-computed-properties" => Some("script/no-async-in-computed"),
            _ => None,
        };
        let hidden_rule = available.contains(eslint_rule.as_str())
            || available.contains(script_rule.as_str())
            || alias.is_some_and(|rule| available.contains(rule));
        assert!(
            !hidden_rule,
            "{eslint_rule} is marked unimplemented despite a registered Patina counterpart"
        );
    }
}

/// The rule map records each mapped rule's default severity and preset
/// membership so `tools/fixtures/lint-divergence-report.mjs` can configure the
/// `eslint-plugin-vue` baseline from the checked-in fixture alone, without a
/// native binding. Both fields are classification inputs: the comparator matches
/// on severity, and a rule no preset activates would turn every upstream finding
/// into a false negative, so drift has to fail here.
#[test]
fn eslint_vue_rule_map_records_current_severity_and_preset_membership() {
    let rule_map: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/patina-eslint-vue-rule-map.json"
    ))
    .unwrap();

    let mut severities: BTreeMap<&'static str, &'static str> = BTreeMap::new();
    for registry in [RuleRegistry::with_all(), RuleRegistry::with_opt_in_rules()] {
        for rule in registry.rules() {
            let meta = rule.meta();
            severities.insert(meta.name, severity_name(meta.default_severity));
        }
    }
    for script_rule in crate::linter::script_rules::builtin_script_rules() {
        severities.insert(
            script_rule.name,
            severity_name(script_rule.default_severity),
        );
    }
    let presets_by_rule = current_plugin_presets_by_rule();

    for (eslint_rule, entry) in rule_map["entries"].as_object().unwrap() {
        if entry["status"] != "mapped" {
            continue;
        }
        let target = entry["patinaRule"].as_str().unwrap();
        assert_eq!(
            entry["patinaSeverity"].as_str(),
            severities.get(target).copied(),
            "{eslint_rule} records a stale default severity for {target}"
        );
        let recorded: Vec<&str> = entry["patinaPresets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|preset| preset.as_str().unwrap())
            .collect();
        let expected = presets_by_rule.get(target).cloned().unwrap_or_default();
        assert_eq!(
            recorded, expected,
            "{eslint_rule} records stale preset membership for {target}"
        );
    }
}

const fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    }
}

fn current_plugin_presets_by_rule() -> BTreeMap<&'static str, Vec<&'static str>> {
    let mut presets_by_rule: BTreeMap<&'static str, BTreeSet<&'static str>> = BTreeMap::new();
    for (preset, registry) in [
        (
            "essential",
            RuleRegistry::with_preset(LintPreset::Essential),
        ),
        (
            "general-recommended",
            RuleRegistry::with_preset(LintPreset::HappyPath),
        ),
        ("nuxt", RuleRegistry::with_preset(LintPreset::Nuxt)),
        ("ecosystem", RuleRegistry::with_ecosystem()),
        (
            "opinionated",
            RuleRegistry::with_preset(LintPreset::Opinionated),
        ),
    ] {
        for rule in registry.rules() {
            presets_by_rule
                .entry(rule.meta().name)
                .or_default()
                .insert(preset);
        }
    }
    for script_rule in crate::linter::script_rules::builtin_script_rules() {
        for preset in script_rule.presets {
            presets_by_rule
                .entry(script_rule.name)
                .or_default()
                .insert(plugin_preset_name_from_raw(preset));
        }
    }

    presets_by_rule
        .into_iter()
        .map(|(rule, presets)| (rule, presets.into_iter().collect()))
        .collect()
}

fn plugin_preset_name_from_raw(preset: &'static str) -> &'static str {
    match preset {
        "general-recommended" | "GeneralRecommended" | "generalRecommended" => {
            "general-recommended"
        }
        "happy-path" | "happy_path" | "happy" | "default" | "recommended" => "general-recommended",
        "essential" | "Essential" => "essential",
        "incremental" | "Incremental" => "incremental",
        "ecosystem" | "Ecosystem" | "eco" | "Eco" => "ecosystem",
        "opinionated" | "Opinionated" | "strict" | "all" | "opnionated" => "opinionated",
        "nuxt" | "Nuxt" => "nuxt",
        _ => preset,
    }
}
