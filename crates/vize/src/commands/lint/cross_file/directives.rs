//! Admission for the concrete project producers enabled by this CLI lane.
//! Rule IDs stay semantic; rendered "cross-file" messages never enter matching.

use std::path::Path;
use vize_patina::{LintDiagnostic, LintResult, ProjectLintDirectives};

use super::super::routes::config::{CrossFileRuleSettings, RuleSetting, report_unused, setting};
use super::{component, fallthrough};

/// These codes are emitted by the provide/inject, unique-id, boundary,
/// reactivity and race producers selected by patina_cross_file_options().
/// Other published Croquis groups do not execute in this lane and cannot
/// make a directive unused. Unknown names may belong to another linter.
const CROQUIS_RULES: &[&str] = &[
    "croquis/cf/async-boundary",
    "croquis/cf/async-no-suspense",
    "croquis/cf/browser-api-ssr",
    "croquis/cf/circular-reactive-dependency",
    "croquis/cf/destructuring-breaks-reactivity",
    "croquis/cf/duplicate-id",
    "croquis/cf/hydration-risk",
    "croquis/cf/inject-without-symbol",
    "croquis/cf/injected-async-mutation-race",
    "croquis/cf/non-reactive-provide",
    "croquis/cf/non-unique-id",
    "croquis/cf/provide-inject-type",
    "croquis/cf/provide-without-symbol",
    "croquis/cf/reassignment-breaks-reactivity",
    "croquis/cf/spread-breaks-reactivity",
    "croquis/cf/uncaught-error",
    "croquis/cf/unmatched-inject",
    "croquis/cf/unused-provide",
    "croquis/cf/value-extraction-breaks-reactivity",
    "croquis/cf/watcheffect-async",
];

pub(super) fn admit(
    states: &[Option<ProjectLintDirectives>],
    index: usize,
    rule: &str,
    diagnostic: &mut LintDiagnostic,
) -> bool {
    states
        .get(index)
        .and_then(Option::as_ref)
        .is_none_or(|state| state.admit(rule, diagnostic))
}

pub(super) fn finalize(
    state: Option<&ProjectLintDirectives>,
    path: &Path,
    result: &mut LintResult,
    settings: Option<&CrossFileRuleSettings<'_>>,
) {
    let Some(state) = state else {
        return;
    };
    if report_unused(settings, path) {
        result.diagnostics.extend(state.unused_named(|rule| {
            (CROQUIS_RULES.contains(&rule) || rule == component::RULE || rule == fallthrough::RULE)
                && !matches!(setting(settings, path, rule), RuleSetting::Off)
        }));
    }
}

pub(super) fn reconcile(state: Option<&ProjectLintDirectives>, result: &mut LintResult) {
    if let Some(state) = state {
        state.reconcile_template_unused(&mut result.diagnostics);
        result.error_count = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == vize_patina::Severity::Error)
            .count();
        result.warning_count = result.diagnostics.len() - result.error_count;
    }
}
