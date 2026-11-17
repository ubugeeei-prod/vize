//! `.tsrx` modules through the shared markup linter (#6858, #6848).
//!
//! The module parses once (L1) and lowers once (L2) into one sealed artifact
//! per markup region. Every region goes through the markup walk JSX/TSX render
//! roots share, in one arena for the module, and the results merge in source
//! order, one diagnostic per rule and range. Syntax errors and constructs the
//! lowering refuses are `parser/tsrx` diagnostics at their authored spans, so
//! no region is skipped silently. Host loops (`@for`) are list scopes: a `key`
//! clause keys their items, and their `@empty` region is content outside the
//! list.
//!
//! This is the one TSRX lint entry: `vize lint`, the Node API and the language
//! server all call [`Linter::lint_tsrx`].

use crate::diagnostic::{LintDiagnostic, Severity};
use crate::linter::config::{LintResult, Linter};
use crate::markup::{L2Markup, MarkupRules};
use oxc_diagnostics::{OxcDiagnostic, Severity as OxcSeverity};
use vize_l0::{Allocator, ToCompactString, cstr, profile};
use vize_l1::markup::tsrx;
use vize_l1_to_l2::framework::vue::tsrx::{TsrxIssue, lower};

/// The operational rule of TSRX parse and lowering diagnostics.
pub const TSRX_PARSE_RULE: &str = "parser/tsrx";

impl Linter {
    /// Lint every markup region of one `.tsrx` module with every enabled
    /// markup rule. Spans address `source`.
    pub fn lint_tsrx(&self, source: &str, filename: &str) -> LintResult {
        let allocator = Allocator::with_capacity((source.len() * 4).max(self.initial_capacity));
        let module = profile!("patina.tsrx.parse", tsrx::parse(&allocator, source));
        let mut diagnostics: Vec<_> = module.diagnostics().iter().map(syntax).collect();
        if module.is_clean() {
            match profile!("patina.tsrx.lower", lower(&allocator, &module)) {
                Err(issues) => diagnostics.extend(issues.iter().map(unsupported)),
                Ok(artifacts) => {
                    let rules = self.markup_rules(usize::MAX, |_, _| true);
                    for region in artifacts.regions() {
                        let markup = L2Markup::from_artifact(region.artifact());
                        let result = self.lint_markup_l2(
                            &allocator,
                            source,
                            filename,
                            &markup,
                            None,
                            MarkupRules::new(&rules),
                        );
                        diagnostics.extend(result.diagnostics);
                    }
                }
            }
        }
        // A nested region's markup is also text of the enclosing region's
        // expression, so a rule can report it from both: keep one per rule
        // and range.
        diagnostics
            .sort_by_key(|diagnostic| (diagnostic.start, diagnostic.end, diagnostic.rule_name));
        diagnostics.dedup_by(|later, earlier| {
            (later.start, later.end, later.rule_name)
                == (earlier.start, earlier.end, earlier.rule_name)
        });
        let error_count = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .count();
        self.apply_diagnostic_policy(LintResult {
            filename: filename.to_compact_string(),
            warning_count: diagnostics.len() - error_count,
            error_count,
            diagnostics,
        })
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

#[cfg(test)]
mod tests;
