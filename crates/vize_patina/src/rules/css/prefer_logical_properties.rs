//! css/prefer-logical-properties
//!
//! Recommend using CSS logical properties for better internationalization.

use vize_l1::css::{StyleNodeKind, StyleSheet};

use crate::diagnostic::{LintDiagnostic, Severity};

use super::style_rule::in_style_scope;
use super::{CssLintResult, CssRuleMeta, StyleRule};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/prefer-logical-properties",
    description: "Recommend CSS logical properties for better i18n support",
    default_severity: Severity::Warning,
};

/// Physical properties and the logical property each one maps to.
const PHYSICAL: [(&str, &str); 8] = [
    ("margin-left", "margin-inline-start"),
    ("margin-right", "margin-inline-end"),
    ("padding-left", "padding-inline-start"),
    ("padding-right", "padding-inline-end"),
    ("border-left-width", "border-inline-start-width"),
    ("border-right-width", "border-inline-end-width"),
    ("left", "inset-inline-start"),
    ("right", "inset-inline-end"),
];

/// It reads every declaration's name in the L1 structure, in every style
/// language, and reports a physical property at its name.
pub struct PreferLogicalProperties;

impl StyleRule for PreferLogicalProperties {
    fn meta(&self) -> &'static CssRuleMeta {
        &META
    }

    fn check(&self, sheet: &StyleSheet<'_>, result: &mut CssLintResult) {
        for node in sheet.nodes() {
            let StyleNodeKind::Declaration { name, .. } = sheet.kind(node) else {
                continue;
            };
            if !in_style_scope(sheet, node) {
                continue;
            }
            let text = sheet.text(name);
            if !PHYSICAL
                .iter()
                .any(|(physical, _)| text.eq_ignore_ascii_case(physical))
            {
                continue;
            }
            result.add_diagnostic(
                LintDiagnostic::warn(
                    META.name,
                    "Consider using logical properties for better RTL support",
                    name.start,
                    name.end,
                )
                .with_help(
                    "Use logical properties like margin-inline-start instead of margin-left",
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PreferLogicalProperties;
    use crate::rules::css::CssLinter;

    fn create_linter() -> CssLinter {
        let mut linter = CssLinter::new();
        linter.add_style_rule(Box::new(PreferLogicalProperties));
        linter
    }

    #[test]
    fn test_valid_logical_properties() {
        let linter = create_linter();
        let result = linter.lint(".button { margin-inline-start: 10px; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warns_physical_properties() {
        let linter = create_linter();
        let result = linter.lint(".button { margin-left: 10px; }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_reports_each_declaration_at_its_own_range() {
        let linter = create_linter();
        let result = linter.lint(
            ".button { margin-left: 10px; margin-left: 20px !important; }",
            0,
        );
        assert_eq!(result.warning_count, 2);
        assert!(result.diagnostics[0].start < result.diagnostics[1].start);
    }

    #[test]
    fn test_warns_nested_physical_property() {
        let linter = create_linter();
        let result = linter.lint(".card { .title { margin-left: 10px; } }", 0);
        assert_eq!(result.warning_count, 1);
    }
}
