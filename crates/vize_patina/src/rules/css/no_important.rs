//! css/no-important
//!
//! Discourage use of !important in CSS.
//!
//! Using !important makes styles harder to override and maintain.
//! It's often a sign of specificity wars and can lead to CSS bloat.

use vize_l1::style::StyleSheet;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::style_rule::{declaration_values, important_flags};
use super::{CssLintResult, CssRuleMeta, StyleRule};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/no-important",
    description: "Discourage use of !important in CSS",
    default_severity: Severity::Warning,
};

/// No !important rule. It reads every declaration's value in the L1
/// structure, so it checks SCSS, Sass, Less and Stylus blocks as written.
pub struct NoImportant;

impl StyleRule for NoImportant {
    fn meta(&self) -> &'static CssRuleMeta {
        &META
    }

    fn check(&self, sheet: &StyleSheet<'_>, result: &mut CssLintResult) {
        if !sheet.source().as_bytes().contains(&b'!') {
            return;
        }
        for value in declaration_values(sheet) {
            for flag in important_flags(sheet, value) {
                result.add_diagnostic(
                    LintDiagnostic::warn(
                        META.name,
                        "Avoid using !important as it makes styles harder to override",
                        flag.start,
                        flag.end,
                    )
                    .with_help("Use more specific selectors or reorganize CSS specificity instead"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoImportant;
    use crate::rules::css::CssLinter;

    fn create_linter() -> CssLinter {
        let mut linter = CssLinter::new();
        linter.add_style_rule(Box::new(NoImportant));
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
