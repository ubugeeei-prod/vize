//! Result retention and summary aggregation for parallel lint runs.

use super::cross_file::CliLintFileResult;
use std::cmp::Ordering;
use vize_patina::{LintDiagnostic, OutputFormat, Severity};

pub(super) struct LintRunAccumulator {
    error_count: usize,
    warning_count: usize,
    results: Option<Vec<CliLintFileResult>>,
}

impl LintRunAccumulator {
    pub(super) fn new(retain_file_results: bool) -> Self {
        Self {
            error_count: 0,
            warning_count: 0,
            results: retain_file_results.then(Vec::new),
        }
    }

    pub(super) fn push(mut self, file_result: CliLintFileResult) -> Self {
        self.error_count += file_result.3.error_count;
        self.warning_count += file_result.3.warning_count;
        if let Some(results) = self.results.as_mut() {
            results.push(file_result);
        }
        self
    }

    pub(super) fn merge(mut self, other: Self) -> Self {
        self.error_count += other.error_count;
        self.warning_count += other.warning_count;
        match (self.results.as_mut(), other.results) {
            (Some(results), Some(other_results)) => results.extend(other_results),
            (None, None) => {}
            _ => unreachable!("lint accumulators must use the same retention mode"),
        }
        self
    }

    pub(super) fn into_parts(self) -> (Vec<CliLintFileResult>, Option<(usize, usize)>) {
        let quiet_totals = self
            .results
            .is_none()
            .then_some((self.error_count, self.warning_count));
        (self.results.unwrap_or_default(), quiet_totals)
    }
}

pub(super) fn totals(
    quiet_totals: Option<(usize, usize)>,
    results: &[CliLintFileResult],
) -> (usize, usize) {
    quiet_totals.unwrap_or_else(|| {
        results
            .iter()
            .fold((0, 0), |(errors, warnings), (_, _, _, result)| {
                (errors + result.error_count, warnings + result.warning_count)
            })
    })
}

pub(super) fn sort_details_for_output(results: &mut [CliLintFileResult]) {
    results.sort_by(compare_file_result);
    for (_, _, _, result) in results {
        result.diagnostics.sort_by(compare_diagnostic);
    }
}

fn compare_file_result(left: &CliLintFileResult, right: &CliLintFileResult) -> Ordering {
    left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0))
}

fn compare_diagnostic(left: &LintDiagnostic, right: &LintDiagnostic) -> Ordering {
    left.start
        .cmp(&right.start)
        .then_with(|| left.end.cmp(&right.end))
        .then_with(|| severity_rank(left.severity).cmp(&severity_rank(right.severity)))
        .then_with(|| left.rule_name.cmp(right.rule_name))
        .then_with(|| left.message.cmp(&right.message))
        .then_with(|| left.help.cmp(&right.help))
}

fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 0,
        Severity::Warning => 1,
    }
}

#[inline]
pub(super) fn should_retain_file_results(render_details: bool, cross_file_enabled: bool) -> bool {
    render_details || cross_file_enabled
}

#[inline]
pub(super) fn should_render_details(format: OutputFormat, quiet: bool) -> bool {
    format.renders_details_when_quiet() || !quiet
}

#[cfg(test)]
mod tests {
    use super::{
        CliLintFileResult, LintRunAccumulator, should_retain_file_results, sort_details_for_output,
    };
    use std::path::PathBuf;
    use vize_patina::{LintDiagnostic, LintResult};

    #[test]
    fn quiet_text_uses_summary_only_collection_without_cross_file_analysis() {
        assert!(!should_retain_file_results(false, false));
        assert!(should_retain_file_results(true, false));
        assert!(should_retain_file_results(false, true));
    }

    #[test]
    fn quiet_accumulator_drops_file_payloads_and_reduces_totals() {
        let first = LintRunAccumulator::new(false).push(file_result("First.vue", 2, 3));
        let second = LintRunAccumulator::new(false).push(file_result("Second.vue", 5, 7));
        let (results, totals) = first.merge(second).into_parts();

        assert_eq!(totals, Some((7, 10)));
        assert!(results.is_empty());
    }

    #[test]
    fn output_details_are_sorted_by_report_filename() {
        let mut results = vec![
            file_result("src/zeta.vue", 0, 1),
            file_result("src/app.vue", 1, 0),
            file_result("src/components/button.vue", 0, 1),
        ];

        sort_details_for_output(&mut results);

        let filenames: Vec<_> = results
            .iter()
            .map(|(_, filename, _, _)| filename.as_str())
            .collect();
        assert_eq!(
            filenames,
            vec!["src/app.vue", "src/components/button.vue", "src/zeta.vue"]
        );
    }

    #[test]
    fn output_details_are_sorted_by_diagnostic_location_then_identity() {
        let mut result = file_result("App.vue", 2, 3);
        result.3.diagnostics = vec![
            LintDiagnostic::warn("vue/later", "later", 20, 24),
            LintDiagnostic::warn("vue/tie-warning", "warning", 10, 12),
            LintDiagnostic::error("vue/tie-error", "error", 10, 12),
            LintDiagnostic::warn("a11y/earliest", "earliest", 5, 8),
        ];
        let mut results = vec![result];

        sort_details_for_output(&mut results);

        let rule_names: Vec<_> = results[0]
            .3
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.rule_name)
            .collect();
        assert_eq!(
            rule_names,
            vec![
                "a11y/earliest",
                "vue/tie-error",
                "vue/tie-warning",
                "vue/later"
            ]
        );
    }

    fn file_result(filename: &str, error_count: usize, warning_count: usize) -> CliLintFileResult {
        (
            PathBuf::from(filename),
            filename.into(),
            "<template />".into(),
            LintResult {
                filename: filename.into(),
                diagnostics: Vec::new(),
                error_count,
                warning_count,
            },
        )
    }
}
