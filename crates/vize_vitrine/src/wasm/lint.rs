//! Patina (Linter) WASM bindings.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use super::{to_js_value, to_json_js_value};
use serde::Serialize;
use vize_patina::{LintPreset, RuleRegistry, builtin_script_rules};
use wasm_bindgen::prelude::*;

mod run;
pub use run::{lint_sfc_wasm, lint_template_wasm};

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
const fn rule_category_name(category: vize_patina::RuleCategory) -> &'static str {
    match category {
        vize_patina::RuleCategory::Essential => "Essential",
        vize_patina::RuleCategory::StronglyRecommended => "StronglyRecommended",
        vize_patina::RuleCategory::Recommended => "Recommended",
        vize_patina::RuleCategory::Vapor => "Vapor",
        vize_patina::RuleCategory::Musea => "Musea",
        vize_patina::RuleCategory::Accessibility => "Accessibility",
        vize_patina::RuleCategory::HtmlConformance => "HtmlConformance",
        vize_patina::RuleCategory::TypeAware => "TypeAware",
        vize_patina::RuleCategory::Ecosystem => "Ecosystem",
    }
}

#[inline]
const fn severity_name(severity: vize_patina::Severity) -> &'static str {
    match severity {
        vize_patina::Severity::Error => "error",
        vize_patina::Severity::Warning => "warning",
    }
}

enum WasmPresetSelection {
    Builtin(LintPreset),
    Ecosystem,
}

fn parse_lint_preset(options: &JsValue) -> WasmPresetSelection {
    js_sys::Reflect::get(options, &JsValue::from_str("preset"))
        .ok()
        .and_then(|v| v.as_string())
        .as_deref()
        .and_then(|value| match value {
            "general-recommended" | "GeneralRecommended" | "generalRecommended" => {
                Some(WasmPresetSelection::Builtin(LintPreset::HappyPath))
            }
            "essential" | "Essential" => Some(WasmPresetSelection::Builtin(LintPreset::Essential)),
            "incremental" | "Incremental" => {
                Some(WasmPresetSelection::Builtin(LintPreset::Incremental))
            }
            "opinionated" | "Opinionated" | "Opnionated" | "opnionated" => {
                Some(WasmPresetSelection::Builtin(LintPreset::Opinionated))
            }
            "ecosystem" | "Ecosystem" | "eco" | "Eco" => Some(WasmPresetSelection::Ecosystem),
            "nuxt" | "Nuxt" => Some(WasmPresetSelection::Builtin(LintPreset::Nuxt)),
            _ => LintPreset::parse(value).map(WasmPresetSelection::Builtin),
        })
        .unwrap_or(WasmPresetSelection::Builtin(LintPreset::default()))
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

fn parse_enabled_rules(options: &JsValue) -> Option<Vec<vize_carton::CompactString>> {
    js_sys::Reflect::get(options, &JsValue::from_str("enabledRules"))
        .ok()
        .and_then(|v| {
            if v.is_undefined() || v.is_null() {
                return None;
            }
            js_sys::Array::from(&v)
                .iter()
                .map(|item| item.as_string().map(Into::into))
                .collect::<Option<Vec<vize_carton::CompactString>>>()
        })
}

pub(super) fn create_linter(locale: vize_patina::Locale, options: &JsValue) -> vize_patina::Linter {
    let enabled_rules = parse_enabled_rules(options);
    let preset = if enabled_rules.is_some() {
        WasmPresetSelection::Builtin(LintPreset::Opinionated)
    } else {
        parse_lint_preset(options)
    };

    let linter = match preset {
        WasmPresetSelection::Builtin(preset) => vize_patina::Linter::with_preset(preset),
        WasmPresetSelection::Ecosystem => vize_patina::Linter::with_ecosystem(),
    };

    linter.with_locale(locale).with_enabled_rules(enabled_rules)
}

/// Get available lint rules
#[wasm_bindgen(js_name = "getLintRules")]
#[allow(clippy::disallowed_macros)]
pub fn get_lint_rules_wasm() -> Result<JsValue, JsValue> {
    use vize_carton::FxHashSet;
    let template_rule_registries = [
        RuleRegistry::with_preset(LintPreset::Opinionated),
        RuleRegistry::with_ecosystem(),
        RuleRegistry::with_opt_in_rules(),
    ];
    let happy_path_rules: FxHashSet<&'static str> =
        RuleRegistry::with_preset(LintPreset::HappyPath)
            .rules()
            .iter()
            .map(|rule| rule.meta().name)
            .collect();
    let essential_rules: FxHashSet<&'static str> = RuleRegistry::with_preset(LintPreset::Essential)
        .rules()
        .iter()
        .map(|rule| rule.meta().name)
        .collect();
    let nuxt_rules: FxHashSet<&'static str> = RuleRegistry::with_preset(LintPreset::Nuxt)
        .rules()
        .iter()
        .map(|rule| rule.meta().name)
        .collect();
    let opinionated_rules: FxHashSet<&'static str> =
        RuleRegistry::with_preset(LintPreset::Opinionated)
            .rules()
            .iter()
            .map(|rule| rule.meta().name)
            .collect();
    let ecosystem_rules: FxHashSet<&'static str> = RuleRegistry::with_ecosystem()
        .rules()
        .iter()
        .map(|rule| rule.meta().name)
        .collect();

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

    to_json_js_value(&rules)
}

/// Get available locales for i18n
#[wasm_bindgen(js_name = "getLocales")]
pub fn get_locales_wasm() -> Result<JsValue, JsValue> {
    use vize_patina::Locale;

    let locales: Vec<serde_json::Value> = Locale::ALL
        .iter()
        .map(|l| {
            serde_json::json!({
                "code": l.code(),
                "name": l.display_name(),
            })
        })
        .collect();

    to_js_value(&locales)
}
