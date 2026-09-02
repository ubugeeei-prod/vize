//! Preset and rule-map coverage for the built-in Patina lint presets.

use super::LintPreset;
use crate::Severity;
use crate::rule::RuleRegistry;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn parses_common_aliases() {
    assert_eq!(LintPreset::parse("default"), Some(LintPreset::Ecosystem));
    assert_eq!(
        LintPreset::parse("recommended"),
        Some(LintPreset::Ecosystem)
    );
    assert_eq!(LintPreset::parse("all"), Some(LintPreset::Opinionated));
    assert_eq!(LintPreset::parse("strict"), Some(LintPreset::Opinionated));
    assert_eq!(
        LintPreset::parse("incremental"),
        Some(LintPreset::Incremental)
    );
    assert_eq!(LintPreset::parse("ecosystem"), Some(LintPreset::Ecosystem));
    assert_eq!(LintPreset::parse("nuxt"), Some(LintPreset::Nuxt));
    assert_eq!(LintPreset::parse("unknown"), None);
}

#[test]
fn preset_rule_membership_snapshot() {
    let snapshot = serde_json::json!({
        "happy_path": rule_names(LintPreset::HappyPath),
        "opinionated": rule_names(LintPreset::Opinionated),
        "essential": rule_names(LintPreset::Essential),
        "ecosystem": ecosystem_rule_names(),
        "incremental": rule_names(LintPreset::Incremental),
        "nuxt": rule_names(LintPreset::Nuxt),
        "opt_in": opt_in_rule_names(),
    });

    insta::assert_snapshot!(
        "lint_preset_rule_membership",
        serde_json::to_string_pretty(&snapshot).unwrap()
    );
}

#[test]
fn script_preset_membership_matches_builtin_rule_metadata() {
    for meta in crate::linter::script_rules::builtin_script_rules() {
        for preset_name in meta.presets {
            assert!(
                LintPreset::parse(preset_name).is_some(),
                "{} declares unknown preset {preset_name}",
                meta.name
            );
        }

        for preset in LintPreset::ALL {
            assert_eq!(
                super::builtin_script_rule_names(preset).contains(&meta.name),
                meta.presets.contains(&preset.as_str()),
                "{} has stale script-rule membership for {}",
                meta.name,
                preset.as_str()
            );
        }
    }
}

#[test]
fn happy_path_keeps_opinionated_rules_opt_in() {
    let happy_path = RuleRegistry::with_preset(LintPreset::HappyPath);
    let opinionated = RuleRegistry::with_preset(LintPreset::Opinionated);

    assert!(happy_path.has_rule("vue/attribute-order"));
    assert!(happy_path.has_rule("vue/component-definition-name-casing"));
    assert!(happy_path.has_rule("vue/html-quotes"));
    assert!(happy_path.has_rule("vue/mustache-interpolation-spacing"));
    assert!(happy_path.has_rule("vue/no-lone-template"));
    assert!(happy_path.has_rule("vue/no-multi-spaces"));
    assert!(happy_path.has_rule("vue/no-unused-properties"));
    assert!(happy_path.has_rule("vue/prop-name-casing"));
    assert!(happy_path.has_rule("vue/require-scoped-style"));
    assert!(happy_path.has_rule("vue/sfc-element-order"));
    assert!(happy_path.has_rule("vue/single-style-block"));
    assert!(happy_path.has_rule("vue/v-on-style"));
    assert!(happy_path.has_rule("vue/v-slot-style"));
    assert!(happy_path.has_rule("vapor/no-vue-lifecycle-events"));
    assert!(happy_path.has_rule("type/require-typed-props"));
    assert!(happy_path.has_rule("type/require-typed-emits"));
    assert!(!happy_path.has_rule("type/no-unsafe-template-binding"));
    assert!(!happy_path.has_rule("type/no-reactivity-loss"));
    assert!(happy_path.has_rule("html/no-empty-palpable-content"));
    assert!(!happy_path.has_rule("vue/multi-word-component-names"));
    assert!(!happy_path.has_rule("vue/no-undefined-refs"));
    assert!(!happy_path.has_rule("a11y/use-list"));
    assert!(RuleRegistry::with_opt_in_rules().has_rule("vue/no-undefined-refs"));
    assert!(opinionated.has_rule("vue/attribute-order"));
    assert!(opinionated.has_rule("vue/component-definition-name-casing"));
    assert!(opinionated.has_rule("vue/html-quotes"));
    assert!(opinionated.has_rule("vue/mustache-interpolation-spacing"));
    assert!(opinionated.has_rule("vue/no-lone-template"));
    assert!(opinionated.has_rule("vue/no-multi-spaces"));
    assert!(opinionated.has_rule("vue/no-unused-properties"));
    assert!(opinionated.has_rule("vue/prop-name-casing"));
    assert!(opinionated.has_rule("vue/require-scoped-style"));
    assert!(opinionated.has_rule("vue/sfc-element-order"));
    assert!(opinionated.has_rule("vue/single-style-block"));
    assert!(opinionated.has_rule("vue/v-on-style"));
    assert!(opinionated.has_rule("vue/v-slot-style"));
    assert!(opinionated.has_rule("vapor/no-vue-lifecycle-events"));
    assert!(opinionated.has_rule("type/require-typed-props"));
    assert!(opinionated.has_rule("type/require-typed-emits"));
    assert!(opinionated.has_rule("type/no-unsafe-template-binding"));
    assert!(opinionated.has_rule("type/no-reactivity-loss"));
    assert!(opinionated.has_rule("html/no-empty-palpable-content"));
    assert!(opinionated.has_rule("vue/multi-word-component-names"));
    assert!(opinionated.has_rule("a11y/use-list"));
    assert!(!opinionated.has_rule("ecosystem/router-link-require-to"));
    assert!(RuleRegistry::with_ecosystem().has_rule("ecosystem/router-link-require-to"));
    assert!(RuleRegistry::with_ecosystem().has_rule("ecosystem/vue-i18n-no-missing-key"));
    let prefer_nuxt_link = "ecosystem/nuxt-prefer-nuxt-link";
    assert!(RuleRegistry::with_preset(LintPreset::Nuxt).has_rule(prefer_nuxt_link));
    assert!(!RuleRegistry::with_ecosystem().has_rule(prefer_nuxt_link));
    assert!(RuleRegistry::with_all().has_rule(prefer_nuxt_link));
    assert!(RuleRegistry::with_opt_in_rules().has_rule("ecosystem/router-link-require-to"));
    assert!(RuleRegistry::with_opt_in_rules().has_rule("ecosystem/vue-i18n-no-missing-key"));
    assert!(
        !super::builtin_script_rule_names(LintPreset::HappyPath).contains(&"script/no-options-api")
    );
    assert!(
        super::builtin_script_rule_names(LintPreset::Opinionated)
            .contains(&"script/no-options-api")
    );
    assert!(!super::builtin_script_rule_names(LintPreset::Nuxt).contains(&"script/no-options-api"));
    assert!(
        !super::builtin_script_rule_names(LintPreset::Nuxt)
            .contains(&"script/no-get-current-instance")
    );
    assert!(!super::builtin_script_rule_names(LintPreset::Nuxt).contains(&"script/no-next-tick"));
    assert!(
        !super::builtin_script_rule_names(LintPreset::Opinionated)
            .contains(&"script/no-export-in-script-setup")
    );
    assert!(
        !super::builtin_script_rule_names(LintPreset::Nuxt)
            .contains(&"script/no-export-in-script-setup")
    );
    assert!(
        super::builtin_script_rule_names(LintPreset::Nuxt).contains(&"nuxt/prefer-import-meta")
    );
    assert!(
        super::builtin_script_rule_names(LintPreset::Nuxt)
            .contains(&"nuxt/no-page-meta-runtime-values")
    );
    assert!(
        super::builtin_script_rule_names(LintPreset::Opinionated)
            .contains(&"script/no-get-current-instance")
    );
    assert!(
        !super::builtin_script_rule_names(LintPreset::Opinionated).contains(&"script/no-next-tick")
    );
    assert!(
        !super::builtin_script_rule_names(LintPreset::Opinionated)
            .contains(&"ecosystem/pinia-prefer-store-to-refs")
    );
    assert!(
        crate::linter::script_rules::opt_in_script_rule_names()
            .contains(&"ecosystem/pinia-prefer-store-to-refs")
    );
    assert!(
        super::ecosystem_builtin_script_rule_names()
            .contains(&"ecosystem/vue-router-prefer-named-push")
    );
}

#[test]
fn incremental_starts_empty() {
    let incremental = RuleRegistry::with_preset(LintPreset::Incremental);

    assert!(!incremental.has_rule("vue/require-v-for-key"));
    assert!(super::builtin_script_rule_names(LintPreset::Incremental).is_empty());
}

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
/// native binding. Both fields are classification inputs — the comparator
/// matches on severity, and a rule no preset activates would turn every
/// upstream finding into a false negative — so drift has to fail here rather
/// than quietly skew a divergence report.
#[test]
fn eslint_vue_rule_map_records_current_severity_and_ecosystem_membership() {
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
    let ecosystem: BTreeSet<&'static str> = ecosystem_rule_names().into_iter().collect();

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
        assert_eq!(
            recorded.contains(&"ecosystem"),
            ecosystem.contains(target),
            "{eslint_rule} records stale ecosystem membership for {target}"
        );
    }
}

const fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    }
}

fn rule_names(preset: LintPreset) -> Vec<&'static str> {
    let mut rules: Vec<_> = RuleRegistry::with_preset(preset)
        .rules()
        .iter()
        .map(|rule| rule.meta().name)
        .collect();
    rules.extend_from_slice(super::builtin_script_rule_names(preset));
    rules.extend_from_slice(super::builtin_css_rule_names(preset));
    rules
}

fn opt_in_rule_names() -> Vec<&'static str> {
    let mut rules: Vec<_> = RuleRegistry::with_opt_in_rules()
        .rules()
        .iter()
        .map(|rule| rule.meta().name)
        .collect();
    rules.extend_from_slice(crate::linter::script_rules::opt_in_script_rule_names());
    rules.extend_from_slice(crate::linter::musea_rules::all_builtin_musea_rule_names());
    rules
}

fn ecosystem_rule_names() -> Vec<&'static str> {
    let mut rules: Vec<_> = RuleRegistry::with_ecosystem()
        .rules()
        .iter()
        .map(|rule| rule.meta().name)
        .collect();
    rules.extend_from_slice(super::ecosystem_builtin_script_rule_names());
    rules
}
