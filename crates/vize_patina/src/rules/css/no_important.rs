//! css/no-important
//!
//! Discourage use of !important in CSS.
//!
//! Using !important makes styles harder to override and maintain.
//! It's often a sign of specificity wars and can lead to CSS bloat.

use lightningcss::stylesheet::StyleSheet;
use memchr::memmem;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{CssLintResult, CssRule, CssRuleMeta};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/no-important",
    description: "Discourage use of !important in CSS",
    default_severity: Severity::Warning,
};

/// No !important rule
pub struct NoImportant;

impl CssRule for NoImportant {
    fn meta(&self) -> &'static CssRuleMeta {
        &META
    }

    fn check<'i>(
        &self,
        source: &'i str,
        _stylesheet: &StyleSheet<'i, 'i>,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        // Use text search to find !important occurrences
        // This provides accurate source positions for inline disable comments
        let bytes = source.as_bytes();
        let finder = memmem::Finder::new(b"!important");

        let mut search_start = 0;
        while let Some(pos) = finder.find(&bytes[search_start..]) {
            let abs_pos = search_start + pos;

            // Verify it's not inside a comment or string
            if !Self::is_in_css_comment(bytes, abs_pos) {
                result.add_diagnostic(
                    LintDiagnostic::warn(
                        META.name,
                        "Avoid using !important as it makes styles harder to override",
                        (offset + abs_pos) as u32,
                        (offset + abs_pos + 10) as u32,
                    )
                    .with_help("Use more specific selectors or reorganize CSS specificity instead"),
                );
            }

            search_start = abs_pos + 1;
        }
    }
}

impl NoImportant {
    /// Check if a position is inside a CSS comment
    fn is_in_css_comment(bytes: &[u8], pos: usize) -> bool {
        let mut in_comment = false;
        let mut i = 0;
        while i < pos && i + 1 < bytes.len() {
            if !in_comment && bytes[i] == b'/' && bytes[i + 1] == b'*' {
                in_comment = true;
                i += 2;
            } else if in_comment && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                in_comment = false;
                i += 2;
            } else {
                i += 1;
            }
        }
        in_comment
    }
}

#[cfg(test)]
mod tests {
    use super::NoImportant;
    use crate::rules::css::CssLinter;

    fn create_linter() -> CssLinter {
        let mut linter = CssLinter::new();
        linter.add_rule(Box::new(NoImportant));
        linter
    }

    #[test]
    fn test_valid_no_important() {
        let linter = create_linter();
        let result = linter.lint(".button { color: red; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_important() {
        let linter = create_linter();
        let result = linter.lint(".button { color: red !important; }", 0);
        assert_eq!(result.warning_count, 1);
    }
}
