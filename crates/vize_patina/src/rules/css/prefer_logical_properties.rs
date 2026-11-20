//! css/prefer-logical-properties
//!
//! Recommend using CSS logical properties for better internationalization.

use lightningcss::stylesheet::StyleSheet as LightningStyleSheet;
use vize_l1::css::{StyleNodeKind, StyleSheet};

use crate::diagnostic::Severity;

use super::style_rule::{identifier_eq, in_style_scope, nested_declaration_name};
use super::{CssLintResult, CssRule, CssRuleMeta, StyleRule};

mod lightning;

static META: CssRuleMeta = CssRuleMeta {
    name: "css/prefer-logical-properties",
    description: "Recommend CSS logical properties for better i18n support",
    default_severity: Severity::Warning,
};

// The built-in path is `StyleRule`; retain the public `CssRule` implementation
// for existing callers of `CssLinter::add_rule`.
const _CSS_RULE_COMPATIBILITY: () = ();

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

impl CssRule for PreferLogicalProperties {
    fn meta(&self) -> &'static CssRuleMeta {
        &META
    }

    fn check<'i>(
        &self,
        source: &'i str,
        stylesheet: &LightningStyleSheet<'i>,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        for rule in &stylesheet.rules.0 {
            self.check_lightning_rule(rule, source, offset, result);
        }
    }
}

impl StyleRule for PreferLogicalProperties {
    fn meta(&self) -> &'static CssRuleMeta {
        &META
    }

    fn check(&self, sheet: &StyleSheet<'_>, result: &mut CssLintResult) {
        for node in sheet.nodes() {
            let StyleNodeKind::Declaration { name, .. } = sheet.kind(node) else {
                continue;
            };
            let text = sheet.text(name);
            let nested = node.parent().is_some_and(|parent| {
                sheet.nodes().get(parent).is_some_and(|parent| {
                    matches!(sheet.kind(parent), StyleNodeKind::Declaration { .. })
                })
            });
            let physical = if nested {
                nested_declaration_name(sheet, node)
                    .is_some_and(|(name, _)| physical_property(&name).is_some())
            } else {
                PHYSICAL
                    .iter()
                    .any(|(physical, _)| identifier_eq(text, physical))
            };
            if !physical || !in_style_scope(sheet, node) {
                continue;
            }
            result.warn(
                META.name,
                "Consider using logical properties for better RTL support",
                "Use logical properties like margin-inline-start instead of margin-left",
                name.start,
                name.end,
            );
        }
    }
}

fn physical_property(name: &str) -> Option<(&'static str, &'static str)> {
    PHYSICAL
        .iter()
        .copied()
        .find(|(physical, _)| name.eq_ignore_ascii_case(physical))
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

    #[test]
    fn test_warns_escaped_and_nested_physical_properties() {
        use vize_l1::css::lang::StyleLanguage;

        let linter = create_linter();
        let scss = ".a { m\\61 rgin-left: 1px; border: { left-width: 2px; } }";
        let result = linter.lint_block(scss, 0, StyleLanguage::Scss, true);
        let reported: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| &scss[diagnostic.start as usize..diagnostic.end as usize])
            .collect();
        assert_eq!(reported, ["m\\61 rgin-left", "left-width"]);
    }

    #[test]
    fn test_matches_only_the_effective_nested_property_name() {
        use vize_l1::css::lang::StyleLanguage;

        let linter = create_linter();
        let scss = ".a { border: { left: { width: 1px; } } #{$side}: { left: 2px; } }";
        let result = linter.lint_block(scss, 0, StyleLanguage::Scss, true);
        let reported: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| &scss[diagnostic.start as usize..diagnostic.end as usize])
            .collect();
        assert_eq!(reported, ["width"]);
    }

    #[test]
    fn test_remains_compatible_with_add_rule() {
        let mut linter = CssLinter::new();
        linter.add_rule(Box::new(PreferLogicalProperties));
        assert_eq!(
            linter
                .lint(".button { margin-left: 10px; }", 0)
                .warning_count,
            1
        );
    }
}
