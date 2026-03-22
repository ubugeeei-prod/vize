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
use vize_patina::{builtin_script_rules, LintPreset, RuleRegistry};
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
    }
}

#[inline]
const fn severity_name(severity: vize_patina::Severity) -> &'static str {
    match severity {
        vize_patina::Severity::Error => "error",
        vize_patina::Severity::Warning => "warning",
    }
}

fn parse_lint_preset(options: &JsValue) -> LintPreset {
    js_sys::Reflect::get(options, &JsValue::from_str("preset"))
        .ok()
        .and_then(|v| v.as_string())
        .as_deref()
        .and_then(LintPreset::parse)
        .unwrap_or_default()
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

fn create_linter(locale: vize_patina::Locale, options: &JsValue) -> vize_patina::Linter {
    let enabled_rules = parse_enabled_rules(options);
    let preset = if enabled_rules.is_some() {
        LintPreset::Opinionated
    } else {
        parse_lint_preset(options)
    };

    vize_patina::Linter::with_preset(preset)
        .with_locale(locale)
        .with_enabled_rules(enabled_rules)
}

/// Lint Vue SFC template
#[wasm_bindgen(js_name = "lintTemplate")]
pub fn lint_template_wasm(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    use vize_patina::{Locale, LspEmitter};

    let filename: String = js_sys::Reflect::get(&options, &JsValue::from_str("filename"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| "anonymous.vue".to_string());

    // Parse locale from options
    let locale: Locale = js_sys::Reflect::get(&options, &JsValue::from_str("locale"))
        .ok()
        .and_then(|v| v.as_string())
        .and_then(|s| Locale::parse(&s))
        .unwrap_or_default();

    let linter = create_linter(locale, &options);
    let result = linter.lint_template(source, &filename);

    // Use LspEmitter for accurate line/column conversion
    let lsp_diagnostics = LspEmitter::to_lsp_diagnostics_with_source(&result, source);

    let diagnostics: Vec<serde_json::Value> = result
        .diagnostics
        .iter()
        .zip(lsp_diagnostics.iter())
        .map(|(d, lsp)| {
            serde_json::json!({
                "rule": d.rule_name,
                "severity": match d.severity {
                    vize_patina::Severity::Error => "error",
                    vize_patina::Severity::Warning => "warning",
                },
                "message": d.message,
                "location": {
                    "start": {
                        "line": lsp.range.start.line + 1, // 1-indexed for display
                        "column": lsp.range.start.character + 1,
                        "offset": d.start,
                    },
                    "end": {
                        "line": lsp.range.end.line + 1,
                        "column": lsp.range.end.character + 1,
                        "offset": d.end,
                    },
                },
                "help": d.help,
            })
        })
        .collect();

    let output = serde_json::json!({
        "filename": result.filename,
        "errorCount": result.error_count,
        "warningCount": result.warning_count,
        "diagnostics": diagnostics,
    });

    to_js_value(&output)
}

/// Lint Vue SFC file (full SFC including script)
#[wasm_bindgen(js_name = "lintSfc")]
pub fn lint_sfc_wasm(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    use vize_carton::i18n::{t_fmt, Locale as CartonLocale};
    use vize_patina::{Locale, LspEmitter};

    let filename: String = js_sys::Reflect::get(&options, &JsValue::from_str("filename"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| "anonymous.vue".to_string());

    // Parse locale from options
    let locale: Locale = js_sys::Reflect::get(&options, &JsValue::from_str("locale"))
        .ok()
        .and_then(|v| v.as_string())
        .and_then(|s| Locale::parse(&s))
        .unwrap_or_default();

    // Convert to carton locale for i18n
    let carton_locale = match locale {
        Locale::En => CartonLocale::En,
        Locale::Ja => CartonLocale::Ja,
        Locale::Zh => CartonLocale::Zh,
    };

    let linter = create_linter(locale, &options);
    let result = linter.lint_sfc(source, &filename);

    // Use LspEmitter for accurate line/column conversion
    let lsp_diagnostics = LspEmitter::to_lsp_diagnostics_with_source(&result, source);

    let diagnostics: Vec<serde_json::Value> = result
        .diagnostics
        .iter()
        .zip(lsp_diagnostics.iter())
        .map(|(d, lsp)| {
            // Format message with i18n format string
            let formatted_message = t_fmt(
                carton_locale,
                "diagnostic.format",
                &[("rule", d.rule_name), ("message", d.message.as_ref())],
            );

            serde_json::json!({
                "rule": d.rule_name,
                "severity": match d.severity {
                    vize_patina::Severity::Error => "error",
                    vize_patina::Severity::Warning => "warning",
                },
                "message": formatted_message,
                "location": {
                    "start": {
                        "line": lsp.range.start.line + 1, // 1-indexed for display
                        "column": lsp.range.start.character + 1,
                        "offset": d.start,
                    },
                    "end": {
                        "line": lsp.range.end.line + 1,
                        "column": lsp.range.end.character + 1,
                        "offset": d.end,
                    },
                },
                "help": d.help,
            })
        })
        .collect();

    let output = serde_json::json!({
        "filename": result.filename,
        "errorCount": result.error_count,
        "warningCount": result.warning_count,
        "diagnostics": diagnostics,
    });

    to_js_value(&output)
}

/// Get available lint rules
#[wasm_bindgen(js_name = "getLintRules")]
#[allow(clippy::disallowed_macros)]
pub fn get_lint_rules_wasm() -> Result<JsValue, JsValue> {
    use vize_carton::FxHashSet;
    use vize_patina::Linter;

    let linter = Linter::with_preset(LintPreset::Opinionated);
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

    let rules: Vec<LintRuleWasm> = linter
        .rules()
        .iter()
        .map(|r| {
            let meta = r.meta();
            let mut presets = Vec::with_capacity(4);
            if essential_rules.contains(meta.name) {
                presets.push(LintPreset::Essential.as_str());
            }
            if happy_path_rules.contains(meta.name) {
                presets.push(LintPreset::HappyPath.as_str());
            }
            if nuxt_rules.contains(meta.name) {
                presets.push(LintPreset::Nuxt.as_str());
            }
            presets.push(LintPreset::Opinionated.as_str());

            LintRuleWasm {
                name: meta.name,
                description: meta.description,
                category: rule_category_name(meta.category),
                fixable: meta.fixable,
                default_severity: severity_name(meta.default_severity),
                presets,
            }
        })
        .collect();
    let mut rules = rules;

    for script_rule in builtin_script_rules() {
        rules.push(LintRuleWasm {
            name: script_rule.name,
            description: script_rule.description,
            category: script_rule.category,
            fixable: script_rule.fixable,
            default_severity: severity_name(script_rule.default_severity),
            presets: script_rule.presets.to_vec(),
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
