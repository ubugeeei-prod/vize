use super::super::should_render_lint_details;
use vize_patina::OutputFormat;

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
