//! Markdown report for comments, issues, and generated artifacts.

use crate::linter::LintResult;
use crate::output::format_summary;
use crate::output::shared::{diagnostic_views, result_counts};
use vize_carton::{String, append};

pub(super) fn format_markdown(results: &[LintResult], sources: &[(String, String)]) -> String {
    let views = diagnostic_views(results, sources);
    let (errors, warnings) = result_counts(results);
    let mut output = String::from("# Patina Lint Report\n\n");
    append!(
        output,
        "Summary: {}.\n",
        format_summary(errors, warnings, results.len())
    );

    if views.is_empty() {
        return output;
    }

    let mut current_file = "";
    for view in &views {
        if current_file != view.file {
            current_file = view.file;
            output.push_str("\n## ");
            output.push_str(current_file);
            output.push_str("\n");
        }

        append!(
            output,
            "\n### {} `{}` at {}:{}\n\n{}\n\nReference: `{}`\n",
            view.severity_name(),
            view.rule_id,
            view.line,
            view.column,
            view.message,
            view.rule_docs_path
        );

        if let Some(help) = view.help_markdown {
            output.push_str("\nHelp:\n\n");
            output.push_str(help);
            output.push('\n');
        }
    }

    output
}
