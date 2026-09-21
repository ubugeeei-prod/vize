//! Patina diagnostics on the unified channel (P4-6c).
//!
//! [`to_unified`] converts one [`LintDiagnostic`] — whose ranges are
//! file-absolute by the time a result leaves the linter — into the
//! `vize_davinci::diagnostic::Diagnostic` every renderer reads, under the
//! witness law:
//!
//! - the rule's [`RuleContract`](vize_davinci::diagnostic::RuleContract)
//!   clamps the reported severity, so a heuristic rule can never reach the
//!   channel as an error, whatever the configuration says;
//! - an error carries the rule's counted [`Exemption`] (no Patina rule
//!   produces a witness yet — the P4-8 waves do), or the one
//!   `configured-error` exemption when configuration raised a warning-default
//!   rule; a warning is an [`Advisory`];
//! - every range is checked against the authored file through
//!   [`SourceRoot`], so a mis-framed range (FP-1's class) is refused, never
//!   rendered on the wrong line.
//!
//! Labels become secondary parts, help a help part at the primary span, and
//! each fix edit a suggestion part whose message is the replacement text.

use vize_davinci::diagnostic::{
    Advisory, Diagnostic, DiagnosticPart, Exemption, PartKind, Severity, Stage,
};
use vize_s0::{SourceRoot, Span};

use crate::diagnostic::LintDiagnostic;
use crate::rule_contracts::contract_for;

/// Template parse errors, reported by the lint engine without a witness.
static PARSER_TEMPLATE: Exemption = Exemption::new("vize_patina", "parser/template");
/// SFC parse errors, reported by the lint engine without a witness.
static PARSER_SFC: Exemption = Exemption::new("vize_patina", "parser/sfc");
/// Errors that exist because configuration raised a warning-default rule to
/// error: the rule's own row is not in the inventory (its default claims no
/// error), so the raise is exempt under this one explicit row instead.
static CONFIGURED: Exemption = Exemption::new("vize_patina", "configured-error");

/// Why a Patina diagnostic cannot join the unified channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnifiedError {
    /// The diagnostic names neither a registered rule nor a parser.
    UnknownRule { rule: &'static str },
    /// A range is not a valid span of the authored file.
    SpanOutsideSource { rule: &'static str, span: Span },
}

/// `diagnostic` on the unified channel, measured against `source`.
///
/// # Errors
///
/// [`UnifiedError::UnknownRule`] for a rule name outside
/// [`RULE_CONTRACTS`](crate::rule_contracts::RULE_CONTRACTS) and the two
/// parsers; [`UnifiedError::SpanOutsideSource`] for the first range that is
/// not a span of `source`.
pub fn to_unified(
    diagnostic: &LintDiagnostic,
    source: SourceRoot<'_>,
) -> Result<Diagnostic, UnifiedError> {
    let rule = diagnostic.rule_name;
    let (stage, exemption, severity) = match rule {
        "parser/template" => (Stage::Surface, &PARSER_TEMPLATE, unified(diagnostic)),
        "parser/sfc" => (Stage::Surface, &PARSER_SFC, unified(diagnostic)),
        _ => {
            let entry = contract_for(rule).ok_or(UnifiedError::UnknownRule { rule })?;
            let severity = entry.contract.clamp(unified(diagnostic));
            (
                Stage::Semantic,
                entry.exemption().unwrap_or(&CONFIGURED),
                severity,
            )
        }
    };
    let span = checked(rule, source, diagnostic.start, diagnostic.end)?;
    let message = diagnostic.message.as_str();
    let mut out = match Advisory::from_severity(severity) {
        Some(advisory) => Diagnostic::new(advisory, stage, span, message),
        None => Diagnostic::legacy_error(exemption, stage, span, message),
    };
    for label in &diagnostic.labels {
        let at = checked(rule, source, label.start, label.end)?;
        out = out.with_part(DiagnosticPart::new(
            PartKind::Secondary,
            at,
            label.message.as_str(),
        ));
    }
    if let Some(help) = &diagnostic.help {
        out = out.with_part(DiagnosticPart::new(PartKind::Help, span, help.as_str()));
    }
    for edit in diagnostic.fix.iter().flat_map(|fix| &fix.edits) {
        let at = checked(rule, source, edit.start, edit.end)?;
        out = out.with_part(DiagnosticPart::new(
            PartKind::Suggestion,
            at,
            edit.new_text.as_str(),
        ));
    }
    Ok(out)
}

fn unified(diagnostic: &LintDiagnostic) -> Severity {
    match diagnostic.severity {
        crate::diagnostic::Severity::Error => Severity::Error,
        crate::diagnostic::Severity::Warning => Severity::Warning,
    }
}

fn checked(
    rule: &'static str,
    source: SourceRoot<'_>,
    start: u32,
    end: u32,
) -> Result<Span, UnifiedError> {
    let span = Span::new(start, end);
    if start <= end && source.contains_span(span) {
        Ok(span)
    } else {
        Err(UnifiedError::SpanOutsideSource { rule, span })
    }
}
