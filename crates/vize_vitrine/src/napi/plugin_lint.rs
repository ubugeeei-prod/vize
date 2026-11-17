//! Plugin lint rules for per-file hosts such as the Oxlint integration (#13074).
//!
//! One session owns the shared per-document host of one `vite.config.*`: its
//! sidecar starts on the first lint, every later file reuses it, and each file
//! crosses into JS once with every job. Diagnostics use the shape of
//! `lintPatinaSfc`, with fixes.
#![expect(clippy::disallowed_types, reason = "N-API strings are std String")]
#![expect(clippy::disallowed_macros, reason = "N-API strings are std String")]

use std::path::Path;

use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use serde_json::{Value, json};
use vize_patina::plugin_batch::lint::host::{PluginLintHost, PluginLintOutcome};
use vize_patina::{LintResult, LspEmitter, Severity};

/// The plugin lint rules of one `vite.config.*`, linted one file at a time.
#[napi(js_name = "PluginLintSession")]
pub struct PluginLintSessionNapi {
    host: Option<PluginLintHost>,
}

#[napi]
impl PluginLintSessionNapi {
    /// Load `configFile` and plan its enabled plugin rules. Throws when the
    /// config or its plugin plan is invalid.
    #[napi(constructor)]
    pub fn new(config_file: String) -> Result<Self> {
        let loaded =
            vize_carton::config::try_load_project_config_with_source(Some(Path::new(&config_file)))
                .map_err(|error| Error::new(Status::InvalidArg, error))?;
        let severities = loaded
            .document
            .linter_plan_with_config_rule_options()
            .plan
            .base
            .rules;
        let host = PluginLintHost::plan(
            &loaded.document,
            loaded.source_path.as_deref(),
            &severities,
            "lint",
        );
        Ok(Self { host })
    }

    /// Whether the config enables any plugin rule.
    #[napi(getter)]
    pub fn enabled(&self) -> bool {
        self.host.is_some()
    }

    /// Lint one SFC with every enabled plugin rule. Throws on a planning,
    /// sidecar or rule failure.
    #[napi]
    pub fn lint(&self, source: String, filename: String) -> Result<Value> {
        let Some(host) = &self.host else {
            return Ok(result(&filename, &source, Vec::new()));
        };
        match host.lint(&source, &filename, &|| false) {
            PluginLintOutcome::Diagnostics(diagnostics) => {
                Ok(result(&filename, &source, diagnostics))
            }
            PluginLintOutcome::Cancelled => Ok(result(&filename, &source, Vec::new())),
            PluginLintOutcome::Failed(error) => Err(Error::new(Status::GenericFailure, error)),
        }
    }
}

fn result(filename: &str, source: &str, diagnostics: Vec<vize_patina::LintDiagnostic>) -> Value {
    let error_count = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let result = LintResult {
        filename: filename.into(),
        warning_count: diagnostics.len() - error_count,
        error_count,
        diagnostics,
    };
    let ranges = LspEmitter::to_lsp_diagnostics_with_source(&result, source);
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .zip(&ranges)
        .map(|(diagnostic, lsp)| {
            let position = |line: u32, character: u32, offset: u32| {
                json!({ "line": line + 1, "column": character + 1, "offset": offset })
            };
            json!({
                "rule": diagnostic.rule_name,
                "severity": match diagnostic.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                },
                "message": diagnostic.message.as_str(),
                "location": {
                    "start": position(lsp.range.start.line, lsp.range.start.character, diagnostic.start),
                    "end": position(lsp.range.end.line, lsp.range.end.character, diagnostic.end),
                },
                "help": diagnostic.help.as_ref().map_or(Value::Null, |help| json!(help.as_ref() as &str)),
                "fix": diagnostic.fix.as_ref().map_or(Value::Null, |fix| json!({
                    "message": fix.message.as_str(),
                    "edits": fix.edits.iter().map(|edit| json!({
                        "start": edit.start,
                        "end": edit.end,
                        "newText": edit.new_text.as_str(),
                    })).collect::<Vec<_>>(),
                })),
            })
        })
        .collect();
    json!({
        "filename": filename,
        "errorCount": result.error_count,
        "warningCount": result.warning_count,
        "diagnostics": diagnostics,
    })
}
