//! Output formatters for lint diagnostics.

mod agent;
mod html;
mod json;
mod markdown;
mod plain;
mod shared;
mod stylish;
mod text;

#[cfg(test)]
mod tests;

pub use json::{JsonFileResult, JsonMessage};
pub use shared::rule_docs_path;
pub use text::*;

use crate::linter::LintResult;
use vize_carton::String;

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
        OutputFormat::Plain => plain::format_plain(results, sources),
        OutputFormat::Stylish => stylish::format_stylish(results, sources),
        OutputFormat::Json => json::format_json(results, sources),
        OutputFormat::Markdown => markdown::format_markdown(results, sources),
        OutputFormat::Html => html::format_html(results, sources),
        OutputFormat::Agent => agent::format_agent(results, sources),
    }
}
