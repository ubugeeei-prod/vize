//! `.tsrx` modules through the shared markup linter (#6858, #6848).
//!
//! The module parses once (L1) and lowers once (L2) into one sealed artifact
//! per markup region. Every region goes through [`Linter::lint_markup`], the
//! entry JSX/TSX render roots share, and the results merge in source order,
//! one diagnostic per rule and range.
//! Syntax errors and constructs the lowering refuses are `parser/tsrx`
//! diagnostics at their authored spans, so no region is skipped silently.

use std::path::Path;

use oxc_diagnostics::{OxcDiagnostic, Severity as OxcSeverity};
use vize_l0::dialect::FileKind;
use vize_l0::{Allocator, ToCompactString, cstr};
use vize_l1::markup::tsrx;
use vize_l2::lang::tsrx::{TsrxIssue, lower};
use vize_patina::markup::L2Markup;
use vize_patina::{LintDiagnostic, LintResult, Linter, Severity};

const TSRX_PARSE_RULE: &str = "parser/tsrx";

pub(super) fn is_tsrx_path(path: &Path) -> bool {
    FileKind::of_path(path) == Some(FileKind::Tsrx)
}

/// Lint every markup region of one `.tsrx` module.
pub(super) fn lint_tsrx(linter: &Linter, source: &str, filename: &str) -> LintResult {
    let allocator = Allocator::new();
    let module = tsrx::parse(&allocator, source);
    let mut diagnostics: Vec<_> = module.diagnostics().iter().map(syntax).collect();
    if module.is_clean() {
        match lower(&allocator, &module) {
            Err(issues) => diagnostics.extend(issues.iter().map(unsupported)),
            Ok(artifacts) => {
                for region in artifacts.regions() {
                    let markup = L2Markup::from_artifact(region.artifact());
                    let result = linter.lint_markup(source, filename, &markup);
                    diagnostics.extend(result.diagnostics);
                }
            }
        }
    }
    // A nested region's markup is also text of the enclosing region's
    // expression, so a rule can report it from both: keep one per rule and range.
    diagnostics.sort_by_key(|diagnostic| (diagnostic.start, diagnostic.end, diagnostic.rule_name));
    diagnostics.dedup_by(|later, earlier| {
        (later.start, later.end, later.rule_name) == (earlier.start, earlier.end, earlier.rule_name)
    });
    let error_count = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    LintResult {
        filename: filename.to_compact_string(),
        warning_count: diagnostics.len() - error_count,
        error_count,
        diagnostics,
    }
}

/// A host or grammar diagnostic, with its specification code when it has one.
fn syntax(diagnostic: &OxcDiagnostic) -> LintDiagnostic {
    let (start, end) = diagnostic.labels.iter().next().map_or((0, 0), |label| {
        (label.offset(), label.offset().saturating_add(label.len()))
    });
    let message = match (&diagnostic.code.scope, &diagnostic.code.number) {
        (Some(scope), Some(number)) => cstr!("{scope}{number}: {}", diagnostic.message),
        _ => diagnostic.message.to_compact_string(),
    };
    match diagnostic.severity {
        OxcSeverity::Error => LintDiagnostic::error(TSRX_PARSE_RULE, message, start, end),
        OxcSeverity::Warning | OxcSeverity::Advice => {
            LintDiagnostic::warn(TSRX_PARSE_RULE, message, start, end)
        }
    }
}

/// A construct the L2 lowering does not support yet: the module's markup is
/// not linted, and this says so where the construct is.
fn unsupported(issue: &TsrxIssue) -> LintDiagnostic {
    LintDiagnostic::error(
        TSRX_PARSE_RULE,
        cstr!(
            "{:?} is not supported in TSRX modules yet, so this module's markup is not linted",
            issue.kind
        ),
        issue.span.start,
        issue.span.end,
    )
}
