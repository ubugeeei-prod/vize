//! CSS rule inputs for a file-wide tool request.
use super::{CssLintResult, CssLinter, DisabledRules, Severity, StyleLanguage};
use vize_l1::css::lang::OriginalStyleStructure;

/// An original module structure, or a style read only by the linter.
pub enum StyleRuleInput<'s, 'a> {
    /// Reuse a module's parsed structure, dialect and absolute start.
    Observed(OriginalStyleStructure<'s, 'a>),
    /// Parse an ordinary inline style when a rule needs its structure.
    Inline {
        /// The original contents of the style element.
        source: &'a str,
        /// Byte offset of the contents within the SFC.
        offset: usize,
        /// Dialect declared by this style.
        language: StyleLanguage,
    },
}

impl CssLinter {
    /// Run this registry over one authored style with its file-absolute spans.
    #[expect(
        clippy::needless_borrow,
        reason = "the same rule loop accepts owned inline sheets and borrowed module sheets"
    )]
    pub fn lint_file_style(
        &self,
        style: StyleRuleInput<'_, '_>,
        respect_comments: bool,
    ) -> CssLintResult {
        let source = match &style {
            StyleRuleInput::Observed(style) => style.sheet().source(),
            StyleRuleInput::Inline { source, .. } => source,
        };
        // Every disable syntax contains this literal. Without one the filter
        // keeps all rows, even when a rule emits diagnostics directly.
        let respect_disable_comments =
            respect_comments && memchr::memmem::find(source.as_bytes(), b"vize-disable").is_some();
        let mut result = match style {
            StyleRuleInput::Inline {
                source,
                offset,
                language,
            } => self.lint_block(source, offset, language, respect_disable_comments),
            StyleRuleInput::Observed(style) => {
                let sheet = style.sheet();
                let offset = sheet.source_start() as usize;
                let language = style.language();
                let read_structure = |_, _: &str, _| sheet;
                include!("block.rs")
            }
        };
        if respect_comments && !respect_disable_comments {
            result.error_count = result
                .diagnostics
                .iter()
                .filter(|row| row.severity == Severity::Error)
                .count();
            result.warning_count = result.diagnostics.len() - result.error_count;
        }
        result
    }
}
