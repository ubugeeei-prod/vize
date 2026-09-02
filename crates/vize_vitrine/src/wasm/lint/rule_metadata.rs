use super::super::to_json_js_value;
use serde::Serialize;
use vize_patina::{
    LintPreset, RuleCategory, RuleRegistry, Severity, builtin_musea_rules, builtin_script_rules,
};
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct LintRuleWasm {
    name: &'static str,
    description: &'static str,
    category: &'static str,
    fixable: bool,
    #[serde(rename = "defaultSeverity")]
    default_severity: &'static str,
    presets: Vec<&'static str>,
}

#[inline]
const fn plugin_preset_name(preset: LintPreset) -> &'static str {
    match preset {
        LintPreset::HappyPath => "general-recommended",
        LintPreset::Opinionated => "opinionated",
        LintPreset::Essential => "essential",
        LintPreset::Incremental => "incremental",
        LintPreset::Ecosystem => "ecosystem",
        LintPreset::Nuxt => "nuxt",
    }
}

#[inline]
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

#[inline]
const fn rule_category_name(category: RuleCategory) -> &'static str {
    match category {
        RuleCategory::Essential => "Essential",
        RuleCategory::StronglyRecommended => "StronglyRecommended",
        RuleCategory::Recommended => "Recommended",
        RuleCategory::Vapor => "Vapor",
        RuleCategory::Musea => "Musea",
        RuleCategory::Accessibility => "Accessibility",
        RuleCategory::HtmlConformance => "HtmlConformance",
        RuleCategory::TypeAware => "TypeAware",
        RuleCategory::Ecosystem => "Ecosystem",
    }
}

#[inline]
const fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    }
}

/// Get available lint rules
#[wasm_bindgen(js_name = "getLintRules")]
#[allow(clippy::disallowed_macros)]
pub fn get_lint_rules_wasm() -> Result<JsValue, JsValue> {
    use vize_s0::FxHashSet;
    let template_rule_registries = [
        RuleRegistry::with_preset(LintPreset::Opinionated),
        RuleRegistry::with_ecosystem(),
        RuleRegistry::with_opt_in_rules(),
    ];
    let happy_path_rules = rule_name_set(RuleRegistry::with_preset(LintPreset::HappyPath));
    let essential_rules = rule_name_set(RuleRegistry::with_preset(LintPreset::Essential));
    let nuxt_rules = rule_name_set(RuleRegistry::with_preset(LintPreset::Nuxt));
    let opinionated_rules = rule_name_set(RuleRegistry::with_preset(LintPreset::Opinionated));
    let ecosystem_rules = rule_name_set(RuleRegistry::with_ecosystem());

    let mut seen = FxHashSet::default();
    let mut rules = Vec::new();
    for registry in &template_rule_registries {
        for rule in registry.rules() {
            let meta = rule.meta();
            if !seen.insert(meta.name) {
                continue;
            }
            let mut presets = Vec::with_capacity(5);
            if essential_rules.contains(meta.name) {
                presets.push(plugin_preset_name(LintPreset::Essential));
            }
            if happy_path_rules.contains(meta.name) {
                presets.push(plugin_preset_name(LintPreset::HappyPath));
            }
            if nuxt_rules.contains(meta.name) {
                presets.push(plugin_preset_name(LintPreset::Nuxt));
            }
            if ecosystem_rules.contains(meta.name) {
                presets.push("ecosystem");
            }
            if opinionated_rules.contains(meta.name) {
                presets.push(plugin_preset_name(LintPreset::Opinionated));
            }

            rules.push(LintRuleWasm {
                name: meta.name,
                description: meta.description,
                category: rule_category_name(meta.category),
                fixable: meta.fixable,
                default_severity: severity_name(meta.default_severity),
                presets,
            });
        }
    }

    for script_rule in builtin_script_rules() {
        rules.push(LintRuleWasm {
            name: script_rule.name,
            description: script_rule.description,
            category: script_rule.category,
            fixable: script_rule.fixable,
            default_severity: severity_name(script_rule.default_severity),
            presets: script_rule
                .presets
                .iter()
                .map(|preset| plugin_preset_name_from_raw(preset))
                .collect(),
        });
    }

    for musea_rule in builtin_musea_rules() {
        rules.push(LintRuleWasm {
            name: musea_rule.name,
            description: musea_rule.description,
            category: "Musea",
            fixable: false,
            default_severity: severity_name(musea_rule.default_severity),
            presets: Vec::new(),
        });
    }

    to_json_js_value(&rules)
}

fn rule_name_set(registry: RuleRegistry) -> vize_s0::FxHashSet<&'static str> {
    registry
        .rules()
        .iter()
        .map(|rule| rule.meta().name)
        .collect()
}
