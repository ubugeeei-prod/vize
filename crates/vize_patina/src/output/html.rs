//! Self-contained HTML report.

use crate::linter::LintResult;
use crate::output::format_summary;
use crate::output::shared::{diagnostic_views, escape_html, result_counts};
use vize_carton::{String, append};

pub(super) fn format_html(results: &[LintResult], sources: &[(String, String)]) -> String {
    let views = diagnostic_views(results, sources);
    let (errors, warnings) = result_counts(results);
    let mut output = String::from(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>Patina Lint Report</title>\n<style>\nbody{font-family:system-ui,-apple-system,BlinkMacSystemFont,\"Segoe UI\",sans-serif;margin:2rem;line-height:1.5;color:#191919;background:#fff}main{max-width:960px}.summary{color:#555}.file{margin-top:2rem}.diagnostic{border:1px solid #ddd;border-left-width:4px;border-radius:6px;padding:1rem;margin:1rem 0}.diagnostic.error{border-left-color:#c62828}.diagnostic.warning{border-left-color:#ad6b00}.meta{display:flex;gap:.75rem;flex-wrap:wrap;align-items:center}.severity{text-transform:uppercase;font-size:.78rem;font-weight:700}.location,.docs{color:#666}code,pre{font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}pre{white-space:pre-wrap;background:#f7f7f7;padding:.75rem;border-radius:4px;overflow:auto}\n</style>\n</head>\n<body>\n<main>\n<h1>Patina Lint Report</h1>\n",
    );
    output.push_str("<p class=\"summary\">");
    output.push_str(&escape_html(&format_summary(
        errors,
        warnings,
        results.len(),
    )));
    output.push_str("</p>\n");

    if views.is_empty() {
        output.push_str("</main>\n</body>\n</html>\n");
        return output;
    }

    let mut current_file = "";
    for view in &views {
        if current_file != view.file {
            if !current_file.is_empty() {
                output.push_str("</section>\n");
            }
            current_file = view.file;
            output.push_str("<section class=\"file\">\n<h2>");
            output.push_str(&escape_html(current_file));
            output.push_str("</h2>\n");
        }

        output.push_str("<article class=\"diagnostic ");
        output.push_str(view.severity_name());
        output.push_str("\">\n<header class=\"meta\"><span class=\"severity\">");
        output.push_str(view.severity_name());
        output.push_str("</span><code>");
        output.push_str(&escape_html(view.rule_id));
        output.push_str("</code><span class=\"location\">");
        append!(
            output,
            "{}:{}-{}:{}",
            view.line,
            view.column,
            view.end_line,
            view.end_column
        );
        output.push_str("</span></header>\n<p>");
        output.push_str(&escape_html(view.message));
        output.push_str("</p>\n<p class=\"docs\">Reference: <code>");
        output.push_str(&escape_html(view.rule_docs_path));
        output.push_str("</code></p>\n");

        if let Some(help) = view.help_text.as_deref() {
            output.push_str("<pre>");
            output.push_str(&escape_html(help));
            output.push_str("</pre>\n");
        }

        output.push_str("</article>\n");
    }

    output.push_str("</section>\n</main>\n</body>\n</html>\n");
    output
}
