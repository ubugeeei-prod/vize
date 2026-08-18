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

pub(super) fn sorted_totals(
    quiet_totals: Option<(usize, usize)>,
    results: &mut [CliLintFileResult],
) -> (usize, usize) {
    sort_details_for_output(results);
    totals(quiet_totals, results)
}

pub(super) fn sort_details_for_output(results: &mut [CliLintFileResult]) {
    results.sort_by(compare_file_result);
    for (_, _, _, result) in results {
        for diagnostic in &mut result.diagnostics {
            diagnostic.labels.sort_by(|left, right| {
                left.start
                    .cmp(&right.start)
                    .then_with(|| left.end.cmp(&right.end))
                    .then_with(|| left.message.cmp(&right.message))
            });
        }
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
        .then_with(|| compare_label_sequences(left, right))
}

fn compare_label_sequences(left: &LintDiagnostic, right: &LintDiagnostic) -> Ordering {
    left.labels
        .iter()
        .map(|label| (label.start, label.end, label.message.as_str()))
        .cmp(
            right
                .labels
                .iter()
                .map(|label| (label.start, label.end, label.message.as_str())),
        )
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
    fn output_details_are_sorted_by_diagnostic_primary_keys() {
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

    #[test]
    fn output_details_are_sorted_by_diagnostic_identity_tiebreakers() {
        let mut result = file_result("App.vue", 0, 7);
        result.3.diagnostics = vec![
            LintDiagnostic::warn("vue/end", "same", 30, 34),
            LintDiagnostic::warn("vue/end", "same", 30, 32),
            LintDiagnostic::warn("vue/rule-b", "same", 40, 42),
            LintDiagnostic::warn("vue/rule-a", "same", 40, 42),
            LintDiagnostic::warn("vue/message", "bravo", 50, 52),
            LintDiagnostic::warn("vue/message", "alpha", 50, 52),
            LintDiagnostic::warn("vue/help", "same", 60, 62).with_help("bravo"),
            LintDiagnostic::warn("vue/help", "same", 60, 62).with_help("alpha"),
            LintDiagnostic::warn("vue/labels", "same", 70, 72)
                .with_help("same")
                .with_label("bravo", 5, 6),
            LintDiagnostic::warn("vue/labels", "same", 70, 72)
                .with_help("same")
                .with_label("alpha", 5, 6),
        ];
        let mut results = vec![result];

        sort_details_for_output(&mut results);

        assert_eq!(
            diagnostic_signatures(&results[0].3.diagnostics),
            vec![
                "30..32|Warning|vue/end|same|None|[]",
                "30..34|Warning|vue/end|same|None|[]",
                "40..42|Warning|vue/rule-a|same|None|[]",
                "40..42|Warning|vue/rule-b|same|None|[]",
                "50..52|Warning|vue/message|alpha|None|[]",
                "50..52|Warning|vue/message|bravo|None|[]",
                "60..62|Warning|vue/help|same|Some(\"alpha\")|[]",
                "60..62|Warning|vue/help|same|Some(\"bravo\")|[]",
                "70..72|Warning|vue/labels|same|Some(\"same\")|[5..6:alpha]",
                "70..72|Warning|vue/labels|same|Some(\"same\")|[5..6:bravo]",
            ]
        );
    }

    #[test]
    fn output_details_sort_related_labels_inside_each_diagnostic() {
        let mut result = file_result("App.vue", 0, 1);
        result.3.diagnostics = vec![
            LintDiagnostic::warn("vue/labels", "same", 10, 12)
                .with_label("last", 20, 22)
                .with_label("first", 1, 3)
                .with_label("middle", 10, 11),
        ];
        let mut results = vec![result];

        sort_details_for_output(&mut results);

        let labels: Vec<_> = results[0].3.diagnostics[0]
            .labels
            .iter()
            .map(|label| format!("{}..{}:{}", label.start, label.end, label.message))
            .collect();
        assert_eq!(labels, vec!["1..3:first", "10..11:middle", "20..22:last"]);
    }

    fn diagnostic_signatures(diagnostics: &[LintDiagnostic]) -> Vec<String> {
        diagnostics
            .iter()
            .map(|diagnostic| {
                let labels = diagnostic
                    .labels
                    .iter()
                    .map(|label| format!("{}..{}:{}", label.start, label.end, label.message))
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "{}..{}|{:?}|{}|{}|{:?}|[{}]",
                    diagnostic.start,
                    diagnostic.end,
                    diagnostic.severity,
                    diagnostic.rule_name,
                    diagnostic.message,
                    diagnostic.help,
                    labels
                )
            })
            .collect()
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
