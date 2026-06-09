//! Plain, line-oriented output optimized for commit hooks and coding agents.

use crate::linter::LintResult;
use crate::output::shared::{diagnostic_views, json_quote, result_counts};
use vize_carton::{String, ToCompactString, append};

pub(super) fn format_agent(results: &[LintResult], sources: &[(String, String)]) -> String {
    let views = diagnostic_views(results, sources);
    let (errors, warnings) = result_counts(results);
    let mut output = String::default();

    append!(
        output,
        "patina report errors={} warnings={} files={}\n",
        errors,
        warnings,
        results.len()
    );

    if views.is_empty() {
        output.push_str("patina ok: no problems found\n");
        return output;
    }

    for view in &views {
        output.push_str("patina diagnostic");
        output.push_str(" file=");
        output.push_str(&json_quote(view.file));
        output.push_str(" line=");
        output.push_str(&view.line.to_compact_string());
        output.push_str(" column=");
        output.push_str(&view.column.to_compact_string());
        output.push_str(" severity=");
        output.push_str(view.severity_name());
        output.push_str(" rule=");
        output.push_str(&json_quote(view.rule_id));
        output.push_str(" docs=");
        output.push_str(&json_quote(view.rule_docs_path));
        output.push('\n');
        output.push_str("message: ");
        output.push_str(view.message);
        output.push('\n');

        if let Some(help) = view.help_text.as_deref() {
            output.push_str("help: ");
            output.push_str(&help.replace('\n', "\n  "));
            output.push('\n');
        }
    }

    output
}
