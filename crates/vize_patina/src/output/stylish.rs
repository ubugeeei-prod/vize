//! ESLint-style compact grouped terminal output.

use crate::linter::LintResult;
use crate::output::format_summary;
use crate::output::shared::{diagnostic_views, result_counts};
use vize_carton::{String, append};

pub(super) fn format_stylish(results: &[LintResult], sources: &[(String, String)]) -> String {
    let views = diagnostic_views(results, sources);
    if views.is_empty() {
        let mut output = String::default();
        output.push_str(&format_summary(0, 0, results.len()));
        output.push('\n');
        return output;
    }

    let rule_width = views
        .iter()
        .map(|view| view.rule_id.len())
        .max()
        .unwrap_or("rule".len());
    let mut output = String::default();
    let mut current_file = "";

    for view in &views {
        if current_file != view.file {
            if !output.is_empty() {
                output.push('\n');
            }
            current_file = view.file;
            output.push_str(current_file);
            output.push('\n');
        }

        append!(
            output,
            "  {:>4}:{:<3}  {:<7}  {:<rule_width$}  {}  {}\n",
            view.line,
            view.column,
            view.severity_name(),
            view.rule_id,
            view.message,
            view.rule_docs_path,
            rule_width = rule_width,
        );
    }

    let (errors, warnings) = result_counts(results);
    output.push('\n');
    output.push_str(&format_summary(errors, warnings, results.len()));
    output.push('\n');
    output
}
