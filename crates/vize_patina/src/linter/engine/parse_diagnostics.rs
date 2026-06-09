//! Translation of parser errors (template and SFC) into lint diagnostics.

use crate::diagnostic::{LintDiagnostic, Severity};
use vize_atelier_sfc::SfcError;
use vize_carton::ToCompactString;
use vize_relief::{CompilerError, ErrorCode};

use super::super::config::{LintResult, Linter};

const TEMPLATE_PARSE_RULE: &str = "parser/template";
const SFC_PARSE_RULE: &str = "parser/sfc";
const INVALID_SELF_CLOSING_HTML_MESSAGE: &str =
    "Invalid self-closing syntax on non-void HTML element";

fn template_parse_diagnostic(parse_error: &CompilerError, source_len: usize) -> LintDiagnostic {
    let (start, end) = template_parse_span(parse_error, source_len);
    if parse_error.is_recoverable() {
        LintDiagnostic::warn(TEMPLATE_PARSE_RULE, parse_error.message.clone(), start, end)
    } else {
        LintDiagnostic::error(TEMPLATE_PARSE_RULE, parse_error.message.clone(), start, end)
    }
}

fn should_report_template_parse_diagnostic(parse_error: &CompilerError) -> bool {
    // Standard mode rewrites invalid HTML self-closing syntax as a compatibility
    // warning. Lint surfaces parser errors, but should not turn this rewrite
    // notice into a project warning budget failure.
    !(parse_error.code == ErrorCode::ExtendPoint
        && parse_error
            .message
            .starts_with(INVALID_SELF_CLOSING_HTML_MESSAGE))
}

fn template_parse_span(parse_error: &CompilerError, source_len: usize) -> (u32, u32) {
    if source_len == 0 {
        return (0, 0);
    }

    let source_len = source_len as u32;
    let (raw_start, raw_end) = parse_error
        .loc
        .as_ref()
        .map(|loc| (loc.start.offset, loc.end.offset))
        .unwrap_or((0, 0));
    let start = raw_start.min(source_len.saturating_sub(1));
    let end = raw_end
        .max(raw_start.saturating_add(1))
        .max(start.saturating_add(1))
        .min(source_len);

    (start, end)
}

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
    pub(crate) fn template_parse_lint_result(
        filename: &str,
        source_len: usize,
        parse_errors: &[CompilerError],
    ) -> LintResult {
        let mut diagnostics = Vec::with_capacity(parse_errors.len());
        let mut error_count = 0;
        let mut warning_count = 0;

        for parse_error in parse_errors {
            if !should_report_template_parse_diagnostic(parse_error) {
                continue;
            }
            let diagnostic = template_parse_diagnostic(parse_error, source_len);
            match diagnostic.severity {
                Severity::Error => error_count += 1,
                Severity::Warning => warning_count += 1,
            }
            diagnostics.push(diagnostic);
        }

        LintResult {
            filename: filename.to_compact_string(),
            diagnostics,
            error_count,
            warning_count,
        }
    }

    pub(crate) fn has_fatal_template_parse_errors(parse_errors: &[CompilerError]) -> bool {
        parse_errors.iter().any(|error| !error.is_recoverable())
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
