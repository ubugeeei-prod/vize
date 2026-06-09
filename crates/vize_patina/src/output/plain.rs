//! Plain text report without ANSI escape codes or code frames.

use crate::linter::LintResult;
use crate::output::format_summary;
use crate::output::shared::{diagnostic_views, push_indented_lines, result_counts};
use vize_carton::{String, append};

pub(super) fn format_plain(results: &[LintResult], sources: &[(String, String)]) -> String {
    let views = diagnostic_views(results, sources);
    let (errors, warnings) = result_counts(results);
    let mut output = String::default();

    append!(
        output,
        "Patina lint report: {}\n",
        format_summary(errors, warnings, results.len())
    );

    if views.is_empty() {
        return output;
    }

    let mut current_file = "";
    for view in &views {
        if current_file != view.file {
            current_file = view.file;
            output.push('\n');
            output.push_str(current_file);
            output.push('\n');
        }

        append!(
            output,
            "  {}:{}:{} {} {} {}\n",
            view.file,
            view.line,
            view.column,
            view.severity_name(),
            view.rule_id,
            view.message
        );
        output.push_str("    Reference: ");
        output.push_str(view.rule_docs_path);
        output.push('\n');

        if let Some(help) = view.help_text.as_deref() {
            output.push_str("    Help:\n");
            push_indented_lines(&mut output, help, "      ");
        }
    }

    output
}
