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
        _stylesheet: &StyleSheet<'i>,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        let bytes = source.as_bytes();
        let comments = comment_ranges(bytes);

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
            while let Some(pos) = bytes.get(search_start..).and_then(|rest| finder.find(rest)) {
                let absolute_pos = search_start + pos;
                if in_comment(&comments, absolute_pos) {
                    search_start = absolute_pos + 1;
                    continue;
                }

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
        if contains_outside(bytes, b"slot", &comments)
            && !contains_outside(bytes, b"::v-slotted", &comments)
            && !contains_outside(bytes, b":slotted", &comments)
        {
            // Only warn if there's actual styling around "slot"
            let finder = memmem::Finder::new(b"slot");
            let mut search_start = 0;

            while let Some(pos) = bytes.get(search_start..).and_then(|rest| finder.find(rest)) {
                let absolute_pos = search_start + pos;

                // Check if "slot" is part of a selector (not inside a value or comment)
                // Look for preceding characters that indicate selector context
                let is_selector = absolute_pos
                    .checked_sub(1)
                    .and_then(|prev| bytes.get(prev))
                    .is_none_or(|prev| matches!(prev, b' ' | b'\n' | b'{' | b',' | b'>'));

                // Check if followed by selector-like characters
                let after_pos = absolute_pos + 4;
                let is_followed_by_selector = bytes
                    .get(after_pos)
                    .is_some_and(|next| matches!(next, b' ' | b'{' | b'.' | b'[' | b'>'));

                if is_selector && is_followed_by_selector && !in_comment(&comments, absolute_pos) {
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

fn comment_ranges(bytes: &[u8]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut index = 0;
    while index + 1 < bytes.len() {
        if bytes.get(index) == Some(&b'/') && bytes.get(index + 1) == Some(&b'*') {
            let start = index;
            index += 2;
            while index + 1 < bytes.len()
                && !(bytes.get(index) == Some(&b'*') && bytes.get(index + 1) == Some(&b'/'))
            {
                index += 1;
            }
            let end = if index + 1 < bytes.len() {
                index + 2
            } else {
                bytes.len()
            };
            ranges.push((start, end));
            index = end;
        } else {
            index += 1;
        }
    }
    ranges
}

fn in_comment(ranges: &[(usize, usize)], pos: usize) -> bool {
    ranges
        .iter()
        .any(|(start, end)| pos >= *start && pos < *end)
}

fn contains_outside(bytes: &[u8], needle: &[u8], comments: &[(usize, usize)]) -> bool {
    let finder = memmem::Finder::new(needle);
    let mut search_start = 0;
    while let Some(pos) = bytes.get(search_start..).and_then(|rest| finder.find(rest)) {
        let absolute = search_start + pos;
        if !in_comment(comments, absolute) {
            return true;
        }
        search_start = absolute + 1;
    }
    false
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
    fn test_ignores_slot_and_deep_selectors_inside_comments() {
        let linter = create_linter();
        let source = r#"
/*
 * Layout notes:
 *   ├ slot     40px
 *   └ details  14px
 *
 * The old `.legacy::v-deep .inner` rule was removed.
 * 旧 `.a .b >>> .c` は無効だった
 */
.field-box {
  display: block;
  & :deep(.child-control) {
    padding: 0 12px;
  }
}
"#;
        let result = linter.lint(source, 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warns_slot_selector() {
        let linter = create_linter();
        // Valid CSS that uses slot element directly
        let result = linter.lint("slot .content { color: red; }", 0);
        assert!(result.warning_count >= 1);
    }
}
