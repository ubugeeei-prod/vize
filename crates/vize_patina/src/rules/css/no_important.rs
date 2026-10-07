//! css/no-important
//!
//! Discourage use of !important in CSS.
//!
//! Using !important makes styles harder to override and maintain.
//! It's often a sign of specificity wars and can lead to CSS bloat.

use lightningcss::stylesheet::StyleSheet;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{CssLintResult, CssRule, CssRuleMeta, value_tokens::ValueTokens};

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
        _stylesheet: &StyleSheet<'i>,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        if !source.as_bytes().contains(&b'!') {
            return;
        }
        for (start, end) in ValueTokens::new(source).important {
            result.add_diagnostic(
                LintDiagnostic::warn(
                    META.name,
                    "Avoid using !important as it makes styles harder to override",
                    (offset + start) as u32,
                    (offset + end) as u32,
                )
                .with_help("Use more specific selectors or reorganize CSS specificity instead"),
            );
        }
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
