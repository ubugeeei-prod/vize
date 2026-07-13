//! Patina diagnostics from Atlas-owned frontend products.

use tower_lsp::lsp_types::{
    CodeDescription, Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url,
};
use vize_patina::{HelpRenderTarget, render_help};

use crate::server::ServerState;

use super::super::{DiagnosticService, LineIndex, sources};

impl DiagnosticService {
    pub(in crate::ide::diagnostics) fn collect_lint_diagnostics(
        state: &ServerState,
        uri: &Url,
        content: &str,
        ecosystem_enabled: bool,
        line_index: &LineIndex<'_>,
    ) -> Vec<Diagnostic> {
        let config = state.get_linter_config();
        if !config.enabled {
            return vec![];
        }

        let Some(result) = state.lint_report_for(uri, content, ecosystem_enabled) else {
            return vec![];
        };
        let result = result.as_ref().clone();

        result
            .diagnostics
            .into_iter()
            .map(|diagnostic| {
                let (start_line, start_col) = line_index.line_col(diagnostic.start as usize);
                let (end_line, end_col) = line_index.line_col(diagnostic.end as usize);
                let message = diagnostic.help.as_ref().map_or_else(
                    || diagnostic.message.to_string(),
                    |help| {
                        format!(
                            "{}\n\nHelp: {}",
                            diagnostic.message,
                            render_help(help, HelpRenderTarget::PlainText)
                        )
                    },
                );
                Diagnostic {
                    range: Range {
                        start: Position {
                            line: start_line,
                            character: start_col,
                        },
                        end: Position {
                            line: end_line,
                            character: end_col,
                        },
                    },
                    severity: Some(match diagnostic.severity {
                        vize_patina::Severity::Error => DiagnosticSeverity::ERROR,
                        vize_patina::Severity::Warning => DiagnosticSeverity::WARNING,
                    }),
                    code: Some(NumberOrString::String(diagnostic.rule_name.to_string())),
                    code_description: lint_rule_description(diagnostic.rule_name),
                    source: Some(sources::LINTER.to_string()),
                    message,
                    ..Default::default()
                }
            })
            .collect()
    }
}

fn lint_rule_description(rule: &str) -> Option<CodeDescription> {
    let name = rule.strip_prefix("vue/").unwrap_or(rule);
    let url = format!("https://eslint.vuejs.org/rules/{name}.html");
    Url::parse(&url)
        .or_else(|_| Url::parse("https://eslint.vuejs.org/rules/"))
        .ok()
        .map(|href| CodeDescription { href })
}
