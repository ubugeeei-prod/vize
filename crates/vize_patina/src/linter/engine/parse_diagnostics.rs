//! Translation of parser errors (template and SFC) into lint diagnostics.

use crate::diagnostic::LintDiagnostic;
use crate::markup::L2Template;
use vize_atelier_sfc::SfcError;
use vize_l0::ToCompactString;

use super::super::config::{LintResult, Linter};

const TEMPLATE_PARSE_RULE: &str = "parser/template";
const SFC_PARSE_RULE: &str = "parser/sfc";

mod compatibility;

fn sfc_parse_span(parse_error: &SfcError, source_len: usize) -> (u32, u32) {
    if source_len == 0 {
        return (0, 0);
    }

    let source_len = source_len as u32;
    let (raw_start, raw_end) = parse_error
        .loc
        .as_ref()
        .map(|loc| (loc.start as u32, loc.end as u32))
        .unwrap_or((0, 0));
    let start = raw_start.min(source_len.saturating_sub(1));
    let end = raw_end
        .max(raw_start.saturating_add(1))
        .max(start.saturating_add(1))
        .min(source_len);

    (start, end)
}

impl Linter {
    pub(crate) fn l1_template_parse_lint_result(
        filename: &str,
        source: &str,
        template: &L2Template<'_>,
    ) -> LintResult {
        let parse_errors = template.surface_errors();
        let mut diagnostics = Vec::with_capacity(parse_errors.len());
        let mut error_count = 0;
        let mut warning_count = 0;

        for parse_error in parse_errors {
            let span = parse_error.span(source.len() as u32);
            let diagnostic = if parse_error.is_recoverable() {
                warning_count += 1;
                LintDiagnostic::warn(
                    TEMPLATE_PARSE_RULE,
                    parse_error.message(source),
                    span.start,
                    span.end,
                )
            } else {
                error_count += 1;
                LintDiagnostic::error(
                    TEMPLATE_PARSE_RULE,
                    parse_error.message(source),
                    span.start,
                    span.end,
                )
            };
            diagnostics.push(diagnostic);
        }

        LintResult {
            filename: filename.to_compact_string(),
            diagnostics,
            error_count,
            warning_count,
        }
    }

    pub(crate) fn has_fatal_l1_template_parse_errors(template: &L2Template<'_>) -> bool {
        template
            .surface_errors()
            .iter()
            .any(|error| !error.has_documented_recovery())
    }

    pub(crate) fn has_fatal_template_parse_errors(&self, template: &L2Template<'_>) -> bool {
        Self::has_fatal_l1_template_parse_errors(template)
    }

    pub(super) fn sfc_parse_lint_result(
        filename: &str,
        source_len: usize,
        parse_error: &SfcError,
    ) -> LintResult {
        let (start, end) = sfc_parse_span(parse_error, source_len);
        LintResult {
            filename: filename.to_compact_string(),
            diagnostics: vec![LintDiagnostic::error(
                SFC_PARSE_RULE,
                parse_error.message.clone(),
                start,
                end,
            )],
            error_count: 1,
            warning_count: 0,
        }
    }
}

#[cfg(test)]
mod tests;
