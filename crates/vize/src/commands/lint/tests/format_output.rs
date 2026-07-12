use super::super::{
    LintRunAccumulator, should_render_lint_details, should_retain_lint_file_results,
};
use std::path::PathBuf;
use vize_patina::{LintResult, OutputFormat};

#[test]
fn quiet_text_output_skips_detailed_diagnostics() {
    assert!(!should_render_lint_details(OutputFormat::Text, true));
}

#[test]
fn json_output_remains_machine_readable_in_quiet_mode() {
    assert!(should_render_lint_details(OutputFormat::Json, true));
}

#[test]
fn report_formats_render_in_quiet_mode() {
    assert!(should_render_lint_details(OutputFormat::Ansi, true));
    assert!(should_render_lint_details(OutputFormat::Plain, true));
    assert!(should_render_lint_details(OutputFormat::Markdown, true));
    assert!(should_render_lint_details(OutputFormat::Html, true));
    assert!(should_render_lint_details(OutputFormat::Agent, true));
}

#[test]
fn quiet_text_uses_summary_only_collection_without_cross_file_analysis() {
    assert!(!should_retain_lint_file_results(false, false));
    assert!(should_retain_lint_file_results(true, false));
    assert!(should_retain_lint_file_results(false, true));
}

#[test]
fn quiet_accumulator_drops_file_payloads_and_reduces_totals() {
    let mut first = LintRunAccumulator::new(false);
    first.push(file_result("First.vue", 2, 3));
    let mut second = LintRunAccumulator::new(false);
    second.push(file_result("Second.vue", 5, 7));
    let accumulator = first.merge(second);

    assert_eq!(accumulator.error_count, 7);
    assert_eq!(accumulator.warning_count, 10);
    assert!(accumulator.results.is_none());
}

fn file_result(
    filename: &str,
    error_count: usize,
    warning_count: usize,
) -> super::super::cross_file::CliLintFileResult {
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
