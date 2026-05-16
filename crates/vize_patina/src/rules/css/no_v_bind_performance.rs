//! css/no-v-bind-performance
//!
//! Warn about performance implications of CSS v-bind().
//!
//! CSS v-bind() creates reactive CSS custom properties at runtime,
//! which has a performance cost. Each v-bind() adds:
//! - A reactive dependency
//! - Runtime style updates on value change
//! - CSS custom property injection
//!
//! Consider using static CSS or computed styles for better performance.

use memchr::memmem;

use lightningcss::stylesheet::StyleSheet;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{CssLintResult, CssRule, CssRuleMeta};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/no-v-bind-performance",
    description: "Warn about performance cost of CSS v-bind()",
    default_severity: Severity::Warning,
};

/// v-bind performance warning rule
pub struct NoVBindPerformance;

impl CssRule for NoVBindPerformance {
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
        // Use SIMD-accelerated search for "v-bind("
        let finder = memmem::Finder::new(b"v-bind(");
        let bytes = source.as_bytes();

        let mut search_start = 0;
        while let Some(pos) = finder.find(&bytes[search_start..]) {
            let absolute_pos = search_start + pos;

            // Find the closing parenthesis
            let end_pos = find_closing_paren(source, absolute_pos + 7);

            let start = (offset + absolute_pos) as u32;
            let end = (offset + end_pos.unwrap_or(absolute_pos + 7)) as u32;

            result.add_diagnostic(
                LintDiagnostic::warn(
                    META.name,
                    "v-bind() has runtime performance cost",
                    start,
                    end,
                )
                .with_help(
                    "v-bind() creates reactive CSS custom properties. Consider static CSS or computed classes for better performance",
                ),
            );

            search_start = absolute_pos + 1;
        }
    }
}

/// Find the closing parenthesis, handling nested parentheses
#[inline]
fn find_closing_paren(source: &str, start: usize) -> Option<usize> {
    let mut depth = 1;
    let bytes = source.as_bytes();

    for (offset, &byte) in bytes[start..].iter().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + offset + 1);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::NoVBindPerformance;
    use crate::rules::css::CssLinter;

    fn create_linter() -> CssLinter {
        let mut linter = CssLinter::new();
        linter.add_rule(Box::new(NoVBindPerformance));
        linter
    }

    #[test]
    fn test_valid_static_css() {
        let linter = create_linter();
        let result = linter.lint(".button { color: red; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warns_v_bind() {
        let linter = create_linter();
        let result = linter.lint(".button { color: v-bind(color); }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_warns_multiple_v_bind() {
        let linter = create_linter();
        let result = linter.lint(
            ".button { color: v-bind(color); background: v-bind(bg); }",
            0,
        );
        assert_eq!(result.warning_count, 2);
    }

    #[test]
    fn test_valid_css_var() {
        let linter = create_linter();
        let result = linter.lint(".button { color: var(--color); }", 0);
        assert_eq!(result.warning_count, 0);
    }
}
