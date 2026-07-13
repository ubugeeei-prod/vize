//! Atlas-backed WASM lint entry points.

use vize_atlas::Shared;
use vize_carton::i18n::{Locale as CartonLocale, t_fmt};
use vize_patina::{LintResult, Locale, LspEmitter};
use wasm_bindgen::prelude::*;

use super::{create_linter, to_js_value};
use crate::lint_artifact::{LintGraphSource, LintSourceKind, PatinaLintGraph};

fn filename(options: &JsValue) -> String {
    js_sys::Reflect::get(options, &JsValue::from_str("filename"))
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_else(|| "anonymous.vue".to_string())
}

fn locale(options: &JsValue) -> Locale {
    js_sys::Reflect::get(options, &JsValue::from_str("locale"))
        .ok()
        .and_then(|value| value.as_string())
        .and_then(|value| Locale::parse(&value))
        .unwrap_or_default()
}

fn query(
    source: &str,
    filename: &str,
    options: &JsValue,
    locale: Locale,
    kind: LintSourceKind,
) -> Result<LintResult, JsValue> {
    let graph = PatinaLintGraph::new(
        Shared::new(create_linter(locale, options)),
        [LintGraphSource {
            name: filename,
            text: source,
            kind,
        }],
    )
    .map_err(|error| JsValue::from_str(&error))?;
    graph
        .query(0)
        .map(|outcome| outcome.result)
        .map_err(|error| JsValue::from_str(&error))
}

/// Lint independently supplied Vue template content.
#[wasm_bindgen(js_name = "lintTemplate")]
pub fn lint_template_wasm(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    let filename = filename(&options);
    let result = query(
        source,
        &filename,
        &options,
        locale(&options),
        LintSourceKind::VueTemplate,
    )?;
    output(&result, source, None)
}

/// Lint a complete Vue SFC.
#[wasm_bindgen(js_name = "lintSfc")]
pub fn lint_sfc_wasm(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    let filename = filename(&options);
    let locale = locale(&options);
    let carton_locale = match locale {
        Locale::En => CartonLocale::En,
        Locale::Ja => CartonLocale::Ja,
        Locale::Zh => CartonLocale::Zh,
    };
    let result = query(source, &filename, &options, locale, LintSourceKind::Sfc)?;
    output(&result, source, Some(carton_locale))
}

fn output(
    result: &LintResult,
    source: &str,
    locale: Option<CartonLocale>,
) -> Result<JsValue, JsValue> {
    let lsp_diagnostics = LspEmitter::to_lsp_diagnostics_with_source(result, source);
    let diagnostics: Vec<serde_json::Value> = result
        .diagnostics
        .iter()
        .zip(lsp_diagnostics.iter())
        .map(|(diagnostic, lsp)| {
            let message = locale.map_or_else(
                || diagnostic.message.clone(),
                |locale| {
                    t_fmt(
                        locale,
                        "diagnostic.format",
                        &[
                            ("rule", diagnostic.rule_name),
                            ("message", diagnostic.message.as_ref()),
                        ],
                    )
                    .into()
                },
            );
            serde_json::json!({
                "rule": diagnostic.rule_name,
                "severity": match diagnostic.severity {
                    vize_patina::Severity::Error => "error",
                    vize_patina::Severity::Warning => "warning",
                },
                "message": message,
                "location": {
                    "start": {
                        "line": lsp.range.start.line + 1,
                        "column": lsp.range.start.character + 1,
                        "offset": diagnostic.start,
                    },
                    "end": {
                        "line": lsp.range.end.line + 1,
                        "column": lsp.range.end.character + 1,
                        "offset": diagnostic.end,
                    },
                },
                "help": diagnostic.help,
            })
        })
        .collect();
    to_js_value(&serde_json::json!({
        "filename": result.filename,
        "errorCount": result.error_count,
        "warningCount": result.warning_count,
        "diagnostics": diagnostics,
    }))
}
