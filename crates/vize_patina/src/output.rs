//! Output formatters for lint diagnostics.

mod text;

pub use text::*;

use crate::diagnostic::{HelpRenderTarget, render_help};
use crate::linter::LintResult;
use serde::Serialize;
use vize_carton::{FxHashMap, SmallVec, String, ToCompactString, append};

/// Output format for lint results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Rich terminal output with colors and code snippets
    #[default]
    Text,
    /// Full ANSI report with colors, code snippets, and formatted help
    Ansi,
    /// Plain text report without ANSI escape codes or code frames
    Plain,
    /// ESLint-style compact grouped terminal output
    Stylish,
    /// JSON output for tooling integration
    Json,
    /// Markdown report for comments, issues, and generated artifacts
    Markdown,
    /// Self-contained HTML report
    Html,
    /// Plain, line-oriented output optimized for commit hooks and coding agents
    Agent,
}

impl OutputFormat {
    /// Parse a user-facing output format name.
    pub fn parse(format: &str) -> Option<Self> {
        match format {
            "text" | "codeframe" | "code-frame" => Some(Self::Text),
            "ansi" | "anssi" | "rich" | "rich-text" => Some(Self::Ansi),
            "plain" | "plain-text" => Some(Self::Plain),
            "stylish" => Some(Self::Stylish),
            "json" => Some(Self::Json),
            "markdown" | "md" => Some(Self::Markdown),
            "html" => Some(Self::Html),
            "agent" | "telegraph" => Some(Self::Agent),
            _ => None,
        }
    }

    /// User-facing format name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Ansi => "ansi",
            Self::Plain => "plain",
            Self::Stylish => "stylish",
            Self::Json => "json",
            Self::Markdown => "markdown",
            Self::Html => "html",
            Self::Agent => "agent",
        }
    }

    /// Non-text formats are whole-report transforms, so they should render even with `--quiet`.
    pub const fn renders_details_when_quiet(self) -> bool {
        !matches!(self, Self::Text)
    }
}

/// Format lint results according to the specified format
pub fn format_results(
    results: &[LintResult],
    sources: &[(String, String)],
    format: OutputFormat,
) -> String {
    match format {
        OutputFormat::Text => format_text(results, sources),
        OutputFormat::Ansi => format_ansi(results, sources),
        OutputFormat::Plain => format_plain(results, sources),
        OutputFormat::Stylish => format_stylish(results, sources),
        OutputFormat::Json => format_json(results, sources),
        OutputFormat::Markdown => format_markdown(results, sources),
        OutputFormat::Html => format_html(results, sources),
        OutputFormat::Agent => format_agent(results, sources),
    }
}

/// JSON output structure for a single file
#[derive(Debug, Serialize)]
pub struct JsonFileResult {
    pub file: String,
    pub messages: Vec<JsonMessage>,
    #[serde(rename = "errorCount")]
    pub error_count: usize,
    #[serde(rename = "warningCount")]
    pub warning_count: usize,
}

/// JSON output structure for a single message
#[derive(Debug, Serialize)]
pub struct JsonMessage {
    #[serde(rename = "ruleId")]
    pub rule_id: &'static str,
    #[serde(rename = "ruleDocsPath")]
    pub rule_docs_path: &'static str,
    pub severity: u8,
    pub message: String,
    pub line: u32,
    pub column: u32,
    #[serde(rename = "endLine")]
    pub end_line: u32,
    #[serde(rename = "endColumn")]
    pub end_column: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

/// Format results as JSON
fn format_json(results: &[LintResult], sources: &[(String, String)]) -> String {
    let source_indices = source_indices(sources);
    let json_results: Vec<JsonFileResult> = results
        .iter()
        .map(|r| JsonFileResult {
            file: r.filename.clone(),
            messages: r
                .diagnostics
                .iter()
                .map(|d| {
                    let location =
                        diagnostic_location(r.filename.as_str(), d.start, d.end, &source_indices);

                    JsonMessage {
                        rule_id: d.rule_name,
                        rule_docs_path: rule_docs_path(d.rule_name),
                        severity: match d.severity {
                            crate::diagnostic::Severity::Error => 2,
                            crate::diagnostic::Severity::Warning => 1,
                        },
                        // Use formatted message with [vize:RULE] prefix
                        message: d.formatted_message(),
                        line: location.line,
                        column: location.column,
                        end_line: location.end_line,
                        end_column: location.end_column,
                        help: d
                            .help
                            .as_ref()
                            .map(|h| render_help(h, HelpRenderTarget::PlainText)),
                    }
                })
                .collect(),
            error_count: r.error_count,
            warning_count: r.warning_count,
        })
        .collect();

    serde_json::to_string_pretty(&json_results)
        .unwrap_or_else(|_| "[]".to_owned())
        .into()
}

/// Return the local documentation page that explains a rule namespace.
pub fn rule_docs_path(rule_name: &str) -> &'static str {
    let namespace = rule_name
        .split_once('/')
        .map(|(namespace, _)| namespace)
        .unwrap_or(rule_name);

    match namespace {
        "vue" => "docs/content/rules/vue.md",
        "a11y" => "docs/content/rules/accessibility.md",
        "html" => "docs/content/rules/html.md",
        "ssr" => "docs/content/rules/ssr.md",
        "vapor" => "docs/content/rules/vapor.md",
        "musea" | "css" => "docs/content/rules/musea-and-css.md",
        "type" | "script" => "docs/content/rules/type-and-script.md",
        "ecosystem" => "docs/content/rules/ecosystem.md",
        "cross-file" | "vize:croquis" => "docs/content/rules/cross-file.md",
        _ => "docs/content/rules/index.md",
    }
}

#[derive(Debug, Clone, Copy)]
struct DiagnosticLocation {
    line: u32,
    column: u32,
    end_line: u32,
    end_column: u32,
}

#[derive(Debug)]
struct DiagnosticView<'a> {
    file: &'a str,
    rule_id: &'static str,
    rule_docs_path: &'static str,
    severity: crate::diagnostic::Severity,
    message: &'a str,
    line: u32,
    column: u32,
    end_line: u32,
    end_column: u32,
    help_markdown: Option<&'a str>,
    help_text: Option<String>,
}

impl DiagnosticView<'_> {
    const fn severity_name(&self) -> &'static str {
        match self.severity {
            crate::diagnostic::Severity::Error => "error",
            crate::diagnostic::Severity::Warning => "warning",
        }
    }
}

fn diagnostic_views<'a>(
    results: &'a [LintResult],
    sources: &'a [(String, String)],
) -> Vec<DiagnosticView<'a>> {
    let source_indices = source_indices(sources);
    let mut views = Vec::new();

    for result in results {
        for diagnostic in &result.diagnostics {
            let location = diagnostic_location(
                result.filename.as_str(),
                diagnostic.start,
                diagnostic.end,
                &source_indices,
            );
            views.push(DiagnosticView {
                file: result.filename.as_str(),
                rule_id: diagnostic.rule_name,
                rule_docs_path: rule_docs_path(diagnostic.rule_name),
                severity: diagnostic.severity,
                message: diagnostic.message.as_ref(),
                line: location.line,
                column: location.column,
                end_line: location.end_line,
                end_column: location.end_column,
                help_markdown: diagnostic.help.as_deref(),
                help_text: diagnostic
                    .help
                    .as_ref()
                    .map(|help| render_help(help, HelpRenderTarget::PlainText)),
            });
        }
    }

    views
}

fn diagnostic_location(
    filename: &str,
    start: u32,
    end: u32,
    source_indices: &FxHashMap<&str, SourceLineIndex>,
) -> DiagnosticLocation {
    source_indices
        .get(filename)
        .map(|source| {
            let (line, column) = source.offset_to_line_col(start);
            let (end_line, end_column) = source.offset_to_line_col(end);
            DiagnosticLocation {
                line,
                column,
                end_line,
                end_column,
            }
        })
        .unwrap_or(DiagnosticLocation {
            line: 1,
            column: start + 1,
            end_line: 1,
            end_column: end + 1,
        })
}

fn source_indices(sources: &[(String, String)]) -> FxHashMap<&str, SourceLineIndex> {
    sources
        .iter()
        .map(|(filename, source)| (filename.as_str(), SourceLineIndex::new(source.as_str())))
        .collect()
}

fn format_stylish(results: &[LintResult], sources: &[(String, String)]) -> String {
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

fn format_plain(results: &[LintResult], sources: &[(String, String)]) -> String {
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

fn format_markdown(results: &[LintResult], sources: &[(String, String)]) -> String {
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

fn format_html(results: &[LintResult], sources: &[(String, String)]) -> String {
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

fn format_agent(results: &[LintResult], sources: &[(String, String)]) -> String {
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

fn result_counts(results: &[LintResult]) -> (usize, usize) {
    let errors = results.iter().map(|result| result.error_count).sum();
    let warnings = results.iter().map(|result| result.warning_count).sum();
    (errors, warnings)
}

fn escape_html(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn json_quote(input: &str) -> String {
    serde_json::to_string(input)
        .unwrap_or_else(|_| "\"\"".to_owned())
        .into()
}

fn push_indented_lines(output: &mut String, text: &str, indent: &str) {
    for line in text.lines() {
        output.push_str(indent);
        output.push_str(line);
        output.push('\n');
    }
}

struct SourceLineIndex {
    source_len: usize,
    line_starts: SmallVec<[usize; 64]>,
}

impl SourceLineIndex {
    fn new(source: &str) -> Self {
        let bytes = source.as_bytes();
        let mut line_starts = SmallVec::new();
        line_starts.push(0);

        for (index, &byte) in bytes.iter().enumerate() {
            if byte == b'\n' {
                line_starts.push(index + 1);
            }
        }

        Self {
            source_len: bytes.len(),
            line_starts,
        }
    }

    fn offset_to_line_col(&self, offset: u32) -> (u32, u32) {
        let offset = (offset as usize).min(self.source_len);
        let line_index = self
            .line_starts
            .partition_point(|&line_start| line_start <= offset)
            .saturating_sub(1);
        let line_start = self.line_starts.get(line_index).copied().unwrap_or(0);
        let line = line_index as u32 + 1;
        let column = offset.saturating_sub(line_start) as u32 + 1;

        (line, column)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        LintDiagnostic, LintPreset, LintResult, Linter, OutputFormat, format_results,
        rule_docs_path,
    };
    use vize_carton::ToCompactString;

    #[test]
    fn json_output_uses_source_line_columns() {
        let source = r#"<script setup lang="ts">
const items = [1]
</script>

<template>
  <div v-for="item in items">{{ item }}</div>
</template>
"#;
        let filename = vize_carton::String::from("Component.vue");
        let result = Linter::new().lint_sfc(source, &filename);
        let output = format_results(
            &[result],
            &[(filename, vize_carton::String::from(source))],
            OutputFormat::Json,
        );

        assert!(output.contains(r#""line": 6"#), "{output}");
        assert!(output.contains(r#""column": 8"#), "{output}");
        assert!(
            output.contains(r#""ruleDocsPath": "docs/content/rules/vue.md""#),
            "{output}"
        );
    }

    #[test]
    fn output_format_parses_report_formats() {
        assert_eq!(OutputFormat::parse("stylish"), Some(OutputFormat::Stylish));
        assert_eq!(OutputFormat::parse("ansi"), Some(OutputFormat::Ansi));
        assert_eq!(OutputFormat::parse("anssi"), Some(OutputFormat::Ansi));
        assert_eq!(OutputFormat::parse("plain-text"), Some(OutputFormat::Plain));
        assert_eq!(OutputFormat::parse("md"), Some(OutputFormat::Markdown));
        assert_eq!(OutputFormat::parse("telegraph"), Some(OutputFormat::Agent));
        assert_eq!(OutputFormat::parse("unknown"), None);
    }

    #[test]
    fn rendered_outputs_interpolate_lint_message_placeholders() {
        let source = r#"<script setup lang="ts">
defineOptions({
  name: "Label",
})

const url = ""
</script>

<template>
  <a :href="url" />
</template>
"#;
        let filename = vize_carton::String::from("Label.vue");
        let result = Linter::with_preset(LintPreset::Essential).lint_sfc(source, &filename);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("Component name \"Label\"")),
            "{:?}",
            result.diagnostics
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("Dynamic :href binding")),
            "{:?}",
            result.diagnostics
        );

        let sources = [(filename.clone(), vize_carton::String::from(source))];
        for format in [
            OutputFormat::Text,
            OutputFormat::Ansi,
            OutputFormat::Plain,
            OutputFormat::Stylish,
            OutputFormat::Json,
            OutputFormat::Markdown,
            OutputFormat::Html,
            OutputFormat::Agent,
        ] {
            let output = format_results(std::slice::from_ref(&result), &sources, format);
            assert!(!output.contains("{name}"), "{format:?}\n{output}");
            assert!(!output.contains("{attr}"), "{format:?}\n{output}");
            assert!(output.contains("Label"), "{format:?}\n{output}");
            assert!(output.contains("href"), "{format:?}\n{output}");
        }
    }

    #[test]
    fn rule_docs_path_maps_namespaces_to_reference_pages() {
        assert_eq!(
            rule_docs_path("vue/require-v-for-key"),
            "docs/content/rules/vue.md"
        );
        assert_eq!(
            rule_docs_path("a11y/img-alt"),
            "docs/content/rules/accessibility.md"
        );
        assert_eq!(
            rule_docs_path("script/no-options-api"),
            "docs/content/rules/type-and-script.md"
        );
        assert_eq!(
            rule_docs_path("cross-file"),
            "docs/content/rules/cross-file.md"
        );
    }

    #[test]
    fn stylish_output_includes_reference_paths() {
        let source = "<template><div v-for=\"item in items\"></div></template>";
        let filename = vize_carton::String::from("Component.vue");
        let result = Linter::new().lint_sfc(source, &filename);
        let output = format_results(
            &[result],
            &[(filename, vize_carton::String::from(source))],
            OutputFormat::Stylish,
        );

        assert!(output.contains("Component.vue"), "{output}");
        assert!(output.contains("vue/require-v-for-key"), "{output}");
        assert!(output.contains("docs/content/rules/vue.md"), "{output}");
    }

    #[test]
    fn text_output_includes_reference_paths() {
        let result = LintResult {
            filename: "Component.vue".to_compact_string(),
            diagnostics: vec![LintDiagnostic::warn(
                "vue/no-v-html",
                "Avoid raw HTML",
                0,
                3,
            )],
            error_count: 0,
            warning_count: 1,
        };
        let output = format_results(
            &[result],
            &[(
                vize_carton::String::from("Component.vue"),
                vize_carton::String::from("abc"),
            )],
            OutputFormat::Text,
        );

        assert!(output.contains("docs/content/rules/vue.md"), "{output}");
    }

    #[test]
    fn ansi_output_includes_summary_and_ansi_help() {
        let result = LintResult {
            filename: "Component.vue".to_compact_string(),
            diagnostics: vec![
                LintDiagnostic::warn("vue/no-v-html", "Avoid raw HTML", 0, 3)
                    .with_help("Use **text interpolation** instead."),
            ],
            error_count: 0,
            warning_count: 1,
        };
        let output = format_results(
            &[result],
            &[(
                vize_carton::String::from("Component.vue"),
                vize_carton::String::from("abc"),
            )],
            OutputFormat::Ansi,
        );

        assert!(output.contains("docs/content/rules/vue.md"), "{output}");
        assert!(output.contains("1 warning in 1 file"), "{output}");
        assert!(output.contains("\x1b["), "{output}");
    }

    #[test]
    fn plain_output_includes_reference_paths_without_ansi() {
        let result = LintResult {
            filename: "Component.vue".to_compact_string(),
            diagnostics: vec![
                LintDiagnostic::error("a11y/img-alt", "Missing alt text", 0, 3)
                    .with_help("Add an `alt` attribute."),
            ],
            error_count: 1,
            warning_count: 0,
        };
        let output = format_results(
            &[result],
            &[(
                vize_carton::String::from("Component.vue"),
                vize_carton::String::from("abc"),
            )],
            OutputFormat::Plain,
        );

        assert!(
            output.contains("Patina lint report: 1 error in 1 file"),
            "{output}"
        );
        assert!(
            output.contains("docs/content/rules/accessibility.md"),
            "{output}"
        );
        assert!(output.contains("Add an alt attribute."), "{output}");
        assert!(!output.contains("\x1b["), "{output}");
    }

    #[test]
    fn markdown_output_keeps_help_and_reference_paths() {
        let result = LintResult {
            filename: "Component.vue".to_compact_string(),
            diagnostics: vec![
                LintDiagnostic::warn("vue/no-v-html", "Avoid raw HTML", 10, 16)
                    .with_help("Use text interpolation instead."),
            ],
            error_count: 0,
            warning_count: 1,
        };
        let output = format_results(
            &[result],
            &[(
                vize_carton::String::from("Component.vue"),
                vize_carton::String::from(""),
            )],
            OutputFormat::Markdown,
        );

        assert!(output.contains("# Patina Lint Report"), "{output}");
        assert!(
            output.contains("Reference: `docs/content/rules/vue.md`"),
            "{output}"
        );
        assert!(
            output.contains("Use text interpolation instead."),
            "{output}"
        );
    }

    #[test]
    fn html_output_escapes_messages() {
        let result = LintResult {
            filename: "Component.vue".to_compact_string(),
            diagnostics: vec![LintDiagnostic::error(
                "html/deprecated-element",
                "Avoid <center> & friends",
                0,
                8,
            )],
            error_count: 1,
            warning_count: 0,
        };
        let output = format_results(
            &[result],
            &[(
                vize_carton::String::from("Component.vue"),
                vize_carton::String::from("<center>"),
            )],
            OutputFormat::Html,
        );

        assert!(output.contains("&lt;center&gt; &amp; friends"), "{output}");
        assert!(output.contains("docs/content/rules/html.md"), "{output}");
    }

    #[test]
    fn agent_output_is_line_oriented() {
        let result = LintResult {
            filename: "Component.vue".to_compact_string(),
            diagnostics: vec![LintDiagnostic::warn(
                "ssr/no-hydration-mismatch",
                "SSR risk",
                0,
                3,
            )],
            error_count: 0,
            warning_count: 1,
        };
        let output = format_results(
            &[result],
            &[(
                vize_carton::String::from("Component.vue"),
                vize_carton::String::from("abc"),
            )],
            OutputFormat::Agent,
        );

        assert!(
            output.starts_with("patina report errors=0 warnings=1 files=1"),
            "{output}"
        );
        assert!(
            output.contains("docs=\"docs/content/rules/ssr.md\""),
            "{output}"
        );
        assert!(output.contains("message: SSR risk"), "{output}");
    }
}
