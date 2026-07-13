//! Result retention and summary aggregation for parallel lint runs.

use super::pipeline::LintedFile;
use vize_patina::OutputFormat;

pub(super) struct LintRunAccumulator {
    error_count: usize,
    warning_count: usize,
    write_failures: usize,
    results: Option<Vec<LintedFile>>,
}

impl LintRunAccumulator {
    pub(super) fn new(retain_file_results: bool) -> Self {
        Self {
            error_count: 0,
            warning_count: 0,
            write_failures: 0,
            results: retain_file_results.then(Vec::new),
        }
    }

    pub(super) fn push(mut self, file_result: LintedFile) -> Self {
        self.error_count += file_result.result.error_count;
        self.warning_count += file_result.result.warning_count;
        self.write_failures += usize::from(file_result.write_failed);
        if let Some(results) = self.results.as_mut() {
            results.push(file_result);
        }
        self
    }

    pub(super) fn merge(mut self, other: Self) -> Self {
        self.error_count += other.error_count;
        self.warning_count += other.warning_count;
        self.write_failures += other.write_failures;
        match (self.results.as_mut(), other.results) {
            (Some(results), Some(other_results)) => results.extend(other_results),
            (None, None) => {}
            _ => unreachable!("lint accumulators must use the same retention mode"),
        }
        self
    }

    pub(super) fn into_parts(self) -> (Vec<LintedFile>, (usize, usize, usize)) {
        let mut results = self.results.unwrap_or_default();
        results.sort_unstable_by_key(|file| file.source_index);
        (
            results,
            (self.error_count, self.warning_count, self.write_failures),
        )
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
    use super::{LintRunAccumulator, should_retain_file_results};
    use crate::commands::lint::pipeline::LintedFile;
    use std::path::PathBuf;
    use vize_patina::LintResult;

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

        assert_eq!(totals, (7, 10, 0));
        assert!(results.is_empty());
    }

    fn file_result(filename: &str, error_count: usize, warning_count: usize) -> LintedFile {
        LintedFile {
            source_index: 0,
            path: PathBuf::from(filename),
            filename: filename.into(),
            source: "<template />".into(),
            result: LintResult {
                filename: filename.into(),
                diagnostics: Vec::new(),
                error_count,
                warning_count,
            },
            semantics: None,
            read_time: std::time::Duration::ZERO,
            lint_time: std::time::Duration::ZERO,
            fixed: false,
            write_failed: false,
            artifact_backed: false,
        }
    }
}
