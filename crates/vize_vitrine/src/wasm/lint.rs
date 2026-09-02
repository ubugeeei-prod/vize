//! Patina (Linter) WASM bindings.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use super::to_js_value;
use vize_patina::LintPreset;
use wasm_bindgen::prelude::*;

mod rule_metadata;
pub use rule_metadata::get_lint_rules_wasm;

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

fn parse_enabled_rules(options: &JsValue) -> Option<Vec<vize_s0::CompactString>> {
    js_sys::Reflect::get(options, &JsValue::from_str("enabledRules"))
        .ok()
        .and_then(|v| {
            if v.is_undefined() || v.is_null() {
                return None;
            }
            js_sys::Array::from(&v)
                .iter()
                .map(|item| item.as_string().map(Into::into))
                .collect::<Option<Vec<vize_s0::CompactString>>>()
        })
}

fn create_linter(locale: vize_patina::Locale, options: &JsValue) -> vize_patina::Linter {
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
    use vize_patina::{Locale, LspEmitter};
    use vize_s0::i18n::{Locale as S0Locale, t_fmt};

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

    // Convert to S0 locale for i18n.
    let s0_locale = match locale {
        Locale::En => S0Locale::En,
        Locale::Ja => S0Locale::Ja,
        Locale::Zh => S0Locale::Zh,
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
                s0_locale,
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
