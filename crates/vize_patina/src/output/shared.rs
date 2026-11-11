//! Shared helpers used across the report-style output formatters.

use crate::diagnostic::{HelpRenderTarget, render_help};
use crate::linter::LintResult;
use vize_l0::cstr;
use vize_l0::{FxHashMap, SmallVec, String};

/// Cross-file rules whose pages the docs generator writes under `rules/project/`.
/// `vize:croquis/*` comes from the cross-file catalogue; these ids are the
/// composed project pages (`project-render.ts`), not reference rules.
const PROJECT_RULES: &[&str] = &[
    "ecosystem/vue-router-extra-param",
    "ecosystem/vue-router-missing-param",
    "ecosystem/vue-router-param-type",
    "ecosystem/vue-router-unknown-route",
    "html/cross-component-nesting",
    "vue/cross-file-attrs-fallthrough",
];

/// Return the rule's own documentation page.
///
/// The slug matches `ruleSlug`: non-alphanumeric runs become a single `-`,
/// ASCII letters are lowercased, and leading or trailing hyphens are dropped.
/// `vue/require-v-for-key` is `docs/content/rules/reference/vue-require-v-for-key.md`.
pub fn rule_docs_path(rule_name: &str) -> String {
    // Operational diagnostics are outside the generated rule catalogue.
    match rule_name {
        "cross-file" => return String::from("docs/content/rules/cross-file.md"),
        "type/corsa-runtime" => return String::from("docs/content/rules/type-and-script.md"),
        "lint/unreadable" | "vue/comment-directive" => {
            return String::from("docs/content/guide/lint-execution.md");
        }
        _ => {}
    }
    let directory = if is_project_rule(rule_name) {
        "project"
    } else {
        "reference"
    };
    let slug = rule_slug(rule_name);
    cstr!("docs/content/rules/{directory}/{slug}.md")
}

#[cfg(test)]
mod documentation_authority_tests {
    #[test]
    fn operational_diagnostics_use_existing_pages_outside_the_rule_catalogue() {
        let registry = crate::rule::RuleRegistry::with_all();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for (rule, page) in [
            ("cross-file", "docs/content/rules/cross-file.md"),
            ("lint/unreadable", "docs/content/guide/lint-execution.md"),
            (
                "type/corsa-runtime",
                "docs/content/rules/type-and-script.md",
            ),
            (
                "vue/comment-directive",
                "docs/content/guide/lint-execution.md",
            ),
        ] {
            assert_eq!(
                registry.has_rule(rule),
                rule == "vue/comment-directive",
                "operational diagnostic: {rule}"
            );
            assert_eq!(super::rule_docs_path(rule), page);
            assert!(root.join(page).is_file(), "English page: {page}");
            assert!(
                root.join(page.replacen("docs/content/", "docs/content/ja/", 1))
                    .is_file(),
                "Japanese page: {page}"
            );
        }
        assert_eq!(
            super::rule_docs_path("vue/comment-directive-host-control"),
            "docs/content/rules/reference/vue-comment-directive-host-control.md"
        );
    }
}

fn is_project_rule(rule_name: &str) -> bool {
    rule_name.starts_with("vize:croquis/") || PROJECT_RULES.contains(&rule_name)
}

fn rule_slug(id: &str) -> String {
    let mut slug = String::with_capacity(id.len());
    let mut hyphen = false;
    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() {
            if hyphen && !slug.is_empty() {
                slug.push('-');
            }
            hyphen = false;
            slug.push(ch.to_ascii_lowercase());
        } else {
            hyphen = true;
        }
    }
    slug
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
    pub rule_docs_path: String,
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
    utf16_adjustments: SmallVec<[(usize, u32); 16]>,
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
        let mut utf16_adjustments = SmallVec::new();
        if !is_ascii {
            let mut extra_bytes_over_utf16 = 0u32;
            for (index, ch) in source.char_indices() {
                let width = ch.len_utf8();
                if width > 1 {
                    extra_bytes_over_utf16 += (width - ch.len_utf16()) as u32;
                    utf16_adjustments.push((index + width, extra_bytes_over_utf16));
                }
            }
        }

        Self {
            is_ascii,
            source_len: bytes.len(),
            line_starts,
            utf16_adjustments,
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
            .extra_bytes_over_utf16_before(offset)
            .saturating_sub(self.extra_bytes_over_utf16_before(line_start));
        let column = byte_column.saturating_sub(extra_bytes_before_column);

        (line, column)
    }

    fn extra_bytes_over_utf16_before(&self, offset: usize) -> u32 {
        let adjustment_index = self
            .utf16_adjustments
            .partition_point(|&(byte_offset, _)| byte_offset <= offset);
        adjustment_index
            .checked_sub(1)
            .and_then(|index| self.utf16_adjustments.get(index))
            .map(|&(_, extra)| extra)
            .unwrap_or(0)
    }
}
