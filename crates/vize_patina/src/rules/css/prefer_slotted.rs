//! css/prefer-slotted
//!
//! Recommend using ::v-slotted() selector for styling slot content.
//!
//! When using scoped styles, slot content from parent components
//! is not affected by the child's scoped styles. Use ::v-slotted()
//! to explicitly target slot content.
//!
//! ## Vue 3.3+ Features
//!
//! - `::v-slotted(.class)` - Style slot content
//! - `::v-deep(.class)` - Style deep child components
//! - `::v-global(.class)` - Escape scoped styles
//!
//! ## Examples
//!
//! ### Using ::v-slotted()
//! ```css
//! <style scoped>
//! ::v-slotted(.content) {
//!   color: red;
//! }
//! </style>
//! ```

use memchr::memmem;

use lightningcss::stylesheet::StyleSheet;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{CssLintResult, CssRule, CssRuleMeta};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/prefer-slotted",
    description: "Recommend ::v-slotted() for styling slot content",
    default_severity: Severity::Warning,
};

/// Prefer slotted rule
pub struct PreferSlotted;

impl CssRule for PreferSlotted {
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
        let bytes = source.as_bytes();

        // Check for deprecated ::v-deep without parentheses (Vue 2 style)
        // In Vue 3, should use :deep() or ::v-deep()
        let deprecated_patterns = [
            (">>> ", "deep selector"),
            ("/deep/ ", "deep selector"),
            ("::v-deep ", "::v-deep without parentheses"),
        ];

        for (pattern, _desc) in deprecated_patterns {
            let finder = memmem::Finder::new(pattern.as_bytes());

            let mut search_start = 0;
            while let Some(pos) = finder.find(&bytes[search_start..]) {
                let absolute_pos = search_start + pos;

                result.add_diagnostic(
                    LintDiagnostic::warn(
                        META.name,
                        "Deprecated deep selector syntax",
                        (offset + absolute_pos) as u32,
                        (offset + absolute_pos + pattern.len()) as u32,
                    )
                    .with_help("Use :deep(.class) or ::v-deep(.class) with parentheses in Vue 3"),
                );

                search_start = absolute_pos + 1;
            }
        }

        // Check for slot element selector that might need ::v-slotted
        // Pattern: direct styling of slot element without ::v-slotted
        if source.contains("slot")
            && !source.contains("::v-slotted")
            && !source.contains(":slotted")
        {
            // Only warn if there's actual styling around "slot"
            let finder = memmem::Finder::new(b"slot");
            let mut search_start = 0;

            while let Some(pos) = finder.find(&bytes[search_start..]) {
                let absolute_pos = search_start + pos;

                // Check if "slot" is part of a selector (not inside a value or comment)
                // Look for preceding characters that indicate selector context
                let is_selector = if absolute_pos > 0 {
                    let prev = bytes[absolute_pos - 1];
                    prev == b' ' || prev == b'\n' || prev == b'{' || prev == b',' || prev == b'>'
                } else {
                    true
                };

                // Check if followed by selector-like characters
                let after_pos = absolute_pos + 4;
                let is_followed_by_selector = after_pos < bytes.len()
                    && (bytes[after_pos] == b' '
                        || bytes[after_pos] == b'{'
                        || bytes[after_pos] == b'.'
                        || bytes[after_pos] == b'['
                        || bytes[after_pos] == b'>');

                if is_selector && is_followed_by_selector {
                    result.add_diagnostic(
                        LintDiagnostic::warn(
                            META.name,
                            "Consider using ::v-slotted() to style slot content in scoped styles",
                            (offset + absolute_pos) as u32,
                            (offset + absolute_pos + 4) as u32,
                        )
                        .with_help(
                            "Use `::v-slotted(selector)` to explicitly target content passed to slots",
                        ),
                    );
                }

                search_start = absolute_pos + 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PreferSlotted;
    use crate::rules::css::CssLinter;

    fn create_linter() -> CssLinter {
        let mut linter = CssLinter::new();
        linter.add_rule(Box::new(PreferSlotted));
        linter
    }

    #[test]
    fn test_valid_v_slotted() {
        let linter = create_linter();
        let result = linter.lint("::v-slotted(.content) { color: red; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_normal_selector() {
        let linter = create_linter();
        let result = linter.lint(".button { color: red; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    // Note: Tests for deprecated >>> and /deep/ patterns are not included here
    // because they are invalid CSS syntax and fail to parse.
    // These patterns should be detected at the SFC level before CSS parsing,
    // or by a raw text scanner that runs before the CSS linter.

    #[test]
    fn test_warns_slot_selector() {
        let linter = create_linter();
        // Valid CSS that uses slot element directly
        let result = linter.lint("slot .content { color: red; }", 0);
        assert!(result.warning_count >= 1);
    }
}
