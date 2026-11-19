//! css/no-id-selectors
//!
//! Discourage use of ID selectors in CSS.
//!
//! ID selectors have very high specificity which makes styles hard to override.
//! They also can't be reused since IDs must be unique in a document.

use vize_l1::style::StyleSheet;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::style_rule::{key_compound_ids, rule_preludes};
use super::{CssLintResult, CssRuleMeta, StyleRule};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/no-id-selectors",
    description: "Discourage use of ID selectors in CSS",
    default_severity: Severity::Warning,
};

/// It reads every rule's selector in the L1 structure, so it checks SCSS,
/// Sass, Less and Stylus blocks as written.
pub struct NoIdSelectors;

impl StyleRule for NoIdSelectors {
    fn meta(&self) -> &'static CssRuleMeta {
        &META
    }

    fn check(&self, sheet: &StyleSheet<'_>, result: &mut CssLintResult) {
        if !sheet.source().as_bytes().contains(&b'#') {
            return;
        }
        for prelude in rule_preludes(sheet) {
            for id in key_compound_ids(sheet, prelude) {
                result.add_diagnostic(
                    LintDiagnostic::warn(
                        META.name,
                        "Avoid ID selectors - use class selectors for better reusability",
                        id.start,
                        id.end,
                    )
                    .with_help("Replace with a class selector for lower specificity"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoIdSelectors;
    use crate::rules::css::CssLinter;

    fn create_linter() -> CssLinter {
        let mut linter = CssLinter::new();
        linter.add_style_rule(Box::new(NoIdSelectors));
        linter
    }

    #[test]
    fn test_valid_class_selector() {
        let linter = create_linter();
        let result = linter.lint(".button { color: red; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_id_selector() {
        let linter = create_linter();
        let result = linter.lint("#header { color: red; }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_nested_id_selector() {
        let linter = create_linter();
        let result = linter.lint(
            ".card { .title { color: red; } #header { color: blue; } }",
            0,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_color_hex() {
        let linter = create_linter();
        let result = linter.lint(".button { color: #ff0000; }", 0);
        assert_eq!(result.warning_count, 0);
    }
}
