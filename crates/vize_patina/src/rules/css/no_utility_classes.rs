//! css/no-utility-classes
//!
//! Warn against implementing utility classes in component styles.
//!
//! Utility classes (like Tailwind-style) should be defined globally,
//! not in component scoped styles. Implementing them in component
//! styles leads to:
//! - Duplication across components
//! - Inconsistency
//! - Larger bundle sizes
//!
//! ## Examples
//!
//! ### Invalid
//! ```css
//! .flex { display: flex; }
//! .mt-4 { margin-top: 1rem; }
//! .text-center { text-align: center; }
//! ```
//!
//! ### Valid
//! ```css
//! .my-component { display: flex; margin-top: 1rem; }
//! ```

use memchr::memmem;

use lightningcss::stylesheet::StyleSheet;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{CssLintResult, CssRule, CssRuleMeta};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/no-utility-classes",
    description: "Warn against implementing utility classes in component styles",
    default_severity: Severity::Warning,
};

/// Exact utility class patterns (must match exactly)
static EXACT_UTILITY_PATTERNS: &[&str] = &[
    // Display
    ".flex",
    ".block",
    ".inline",
    ".hidden",
    ".grid",
    // Flexbox
    ".items-center",
    ".justify-center",
    ".justify-between",
    ".flex-col",
    ".flex-row",
    ".flex-wrap",
    // Text
    ".text-center",
    ".text-left",
    ".text-right",
    ".font-bold",
    ".font-semibold",
    ".italic",
    ".underline",
    // Sizing
    ".w-full",
    ".h-full",
    ".w-screen",
    ".h-screen",
    // Position
    ".absolute",
    ".relative",
    ".fixed",
    ".sticky",
];

/// Prefix utility patterns (must be followed by a digit)
/// e.g., .mt-4, .p-2, .gap-1
static PREFIX_UTILITY_PATTERNS: &[&str] = &[
    // Spacing
    ".m-",
    ".p-",
    ".mt-",
    ".mb-",
    ".ml-",
    ".mr-",
    ".mx-",
    ".my-",
    ".pt-",
    ".pb-",
    ".pl-",
    ".pr-",
    ".px-",
    ".py-",
    // Colors/sizing with numbers
    ".bg-",
    ".text-",
    ".w-",
    ".h-",
    // Gap
    ".gap-",
    // Border radius
    ".rounded-",
    // Border width
    ".border-",
];

/// No utility classes rule
pub struct NoUtilityClasses;

impl CssRule for NoUtilityClasses {
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

        // Check exact patterns (must match exactly, no more characters)
        for &pattern in EXACT_UTILITY_PATTERNS {
            let finder = memmem::Finder::new(pattern.as_bytes());

            let mut search_start = 0;
            while let Some(pos) = finder.find(&bytes[search_start..]) {
                let absolute_pos = search_start + pos;

                // Check if this is at the start of a selector
                let is_selector_start = absolute_pos == 0
                    || matches!(
                        bytes.get(absolute_pos - 1),
                        Some(b' ' | b'\n' | b'\r' | b'\t' | b'{' | b'}' | b',')
                    );

                // Check if this is an exact match (followed by space, {, or ,)
                let end_pos = absolute_pos + pattern.len();
                let is_exact_match = end_pos >= bytes.len()
                    || matches!(
                        bytes.get(end_pos),
                        Some(b' ' | b'{' | b',' | b'\n' | b'\r' | b'\t')
                    );

                if is_selector_start && is_exact_match {
                    result.add_diagnostic(
                        LintDiagnostic::warn(
                            META.name,
                            "Utility class should be in global styles, not component styles",
                            (offset + absolute_pos) as u32,
                            (offset + end_pos) as u32,
                        )
                        .with_help(
                            "Use semantic class names in components, or import utility classes from a global stylesheet",
                        ),
                    );
                }

                search_start = absolute_pos + 1;
            }
        }

        // Check prefix patterns (must be followed by a digit)
        for &pattern in PREFIX_UTILITY_PATTERNS {
            let finder = memmem::Finder::new(pattern.as_bytes());

            let mut search_start = 0;
            while let Some(pos) = finder.find(&bytes[search_start..]) {
                let absolute_pos = search_start + pos;

                // Check if this is at the start of a selector
                let is_selector_start = absolute_pos == 0
                    || matches!(
                        bytes.get(absolute_pos - 1),
                        Some(b' ' | b'\n' | b'\r' | b'\t' | b'{' | b'}' | b',')
                    );

                // Check if followed by a digit (utility class pattern)
                let next_pos = absolute_pos + pattern.len();
                let is_followed_by_digit =
                    next_pos < bytes.len() && bytes[next_pos].is_ascii_digit();

                if is_selector_start && is_followed_by_digit {
                    // Find the end of the class name
                    let mut end = next_pos;
                    while end < bytes.len()
                        && (bytes[end].is_ascii_alphanumeric()
                            || bytes[end] == b'-'
                            || bytes[end] == b'_')
                    {
                        end += 1;
                    }

                    result.add_diagnostic(
                        LintDiagnostic::warn(
                            META.name,
                            "Utility class should be in global styles, not component styles",
                            (offset + absolute_pos) as u32,
                            (offset + end) as u32,
                        )
                        .with_help(
                            "Use semantic class names in components, or import utility classes from a global stylesheet",
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
    use super::NoUtilityClasses;
    use crate::rules::css::CssLinter;

    fn create_linter() -> CssLinter {
        let mut linter = CssLinter::new();
        linter.add_rule(Box::new(NoUtilityClasses));
        linter
    }

    #[test]
    fn test_valid_semantic_class() {
        let linter = create_linter();
        let result = linter.lint(".my-component { display: flex; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warns_flex_utility() {
        let linter = create_linter();
        let result = linter.lint(".flex { display: flex; }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_warns_margin_utility() {
        let linter = create_linter();
        let result = linter.lint(".mt-4 { margin-top: 1rem; }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_warns_text_center() {
        let linter = create_linter();
        let result = linter.lint(".text-center { text-align: center; }", 0);
        assert_eq!(result.warning_count, 1);
    }
}
