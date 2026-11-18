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

use lightningcss::stylesheet::StyleSheet;
use memchr::memmem;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{CssLintResult, CssRule, CssRuleMeta, value_tokens::ValueTokens};

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
        _stylesheet: &StyleSheet<'i>,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        if memmem::find(source.as_bytes(), b"v-bind(").is_none() {
            return;
        }
        for (start, end) in ValueTokens::new(source).bindings {
            result.add_diagnostic(
                LintDiagnostic::warn(
                    META.name,
                    "v-bind() installs runtime CSS variable updates",
                    (offset + start) as u32,
                    (offset + end) as u32,
                )
                .with_help(
                    "Vue lowers CSS v-bind() to per-instance reactive CSS custom property updates. Prefer static CSS, computed classes for finite variants, or a template :style binding when a prop-driven value is unavoidable.",
                ),
            );
        }
    }
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
        assert_eq!(
            result.diagnostics[0].message.as_str(),
            "v-bind() installs runtime CSS variable updates"
        );
        assert_eq!(
            result.diagnostics[0]
                .help
                .as_ref()
                .map(|help| help.as_str()),
            Some(
                "Vue lowers CSS v-bind() to per-instance reactive CSS custom property updates. Prefer static CSS, computed classes for finite variants, or a template :style binding when a prop-driven value is unavoidable."
            )
        );
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
