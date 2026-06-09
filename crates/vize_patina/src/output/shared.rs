//! Shared helpers used across the report-style output formatters.

use crate::diagnostic::{HelpRenderTarget, render_help};
use crate::linter::LintResult;
use vize_carton::{FxHashMap, SmallVec, String};

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
pub(super) struct DiagnosticLocation {
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

#[derive(Debug)]
pub(super) struct DiagnosticView<'a> {
    pub file: &'a str,
    pub rule_id: &'static str,
    pub rule_docs_path: &'static str,
    pub severity: crate::diagnostic::Severity,
    pub message: &'a str,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub help_markdown: Option<&'a str>,
    pub help_text: Option<String>,
}

impl DiagnosticView<'_> {
    pub(super) const fn severity_name(&self) -> &'static str {
        match self.severity {
            crate::diagnostic::Severity::Error => "error",
            crate::diagnostic::Severity::Warning => "warning",
        }
    }
}

pub(super) fn diagnostic_views<'a>(
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

pub(super) fn diagnostic_location(
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

pub(super) fn source_indices(sources: &[(String, String)]) -> FxHashMap<&str, SourceLineIndex> {
    sources
        .iter()
        .map(|(filename, source)| (filename.as_str(), SourceLineIndex::new(source.as_str())))
        .collect()
}

pub(super) fn result_counts(results: &[LintResult]) -> (usize, usize) {
    let errors = results.iter().map(|result| result.error_count).sum();
    let warnings = results.iter().map(|result| result.warning_count).sum();
    (errors, warnings)
}

pub(super) fn escape_html(input: &str) -> String {
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

pub(super) fn json_quote(input: &str) -> String {
    serde_json::to_string(input)
        .unwrap_or_else(|_| "\"\"".to_owned())
        .into()
}

pub(super) fn push_indented_lines(output: &mut String, text: &str, indent: &str) {
    for line in text.lines() {
        output.push_str(indent);
        output.push_str(line);
        output.push('\n');
    }
}

pub(super) struct SourceLineIndex {
    is_ascii: bool,
    source_len: usize,
    line_starts: SmallVec<[usize; 64]>,
    multibyte_adjustments: SmallVec<[(usize, u32); 16]>,
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

        let is_ascii = source.is_ascii();
        let mut multibyte_adjustments = SmallVec::new();
        if !is_ascii {
            let mut extra_bytes = 0u32;
            for (index, ch) in source.char_indices() {
                let width = ch.len_utf8();
                if width > 1 {
                    extra_bytes += (width - 1) as u32;
                    multibyte_adjustments.push((index + width, extra_bytes));
                }
            }
        }

        Self {
            is_ascii,
            source_len: bytes.len(),
            line_starts,
            multibyte_adjustments,
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
        if self.is_ascii {
            return (line, offset.saturating_sub(line_start) as u32 + 1);
        }
        let byte_column = offset.saturating_sub(line_start) as u32 + 1;
        let extra_bytes_before_column = self
            .extra_bytes_before(offset)
            .saturating_sub(self.extra_bytes_before(line_start));
        let column = byte_column.saturating_sub(extra_bytes_before_column);

        (line, column)
    }

    fn extra_bytes_before(&self, offset: usize) -> u32 {
        let adjustment_index = self
            .multibyte_adjustments
            .partition_point(|&(byte_offset, _)| byte_offset <= offset);
        adjustment_index
            .checked_sub(1)
            .map(|index| self.multibyte_adjustments[index].1)
            .unwrap_or(0)
    }
}
