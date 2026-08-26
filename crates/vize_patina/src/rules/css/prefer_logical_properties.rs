//! css/prefer-logical-properties
//!
//! Recommend using CSS logical properties for better internationalization.

use lightningcss::declaration::DeclarationBlock;
use lightningcss::properties::PropertyId;
use lightningcss::rules::CssRule as LCssRule;
use lightningcss::stylesheet::StyleSheet;
use vize_s0::FxHashSet;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{CssLintResult, CssRule, CssRuleMeta};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/prefer-logical-properties",
    description: "Recommend CSS logical properties for better i18n support",
    default_severity: Severity::Warning,
};

/// Prefer logical properties rule
pub struct PreferLogicalProperties;

impl CssRule for PreferLogicalProperties {
    fn meta(&self) -> &'static CssRuleMeta {
        &META
    }

    fn check<'i>(
        &self,
        _source: &'i str,
        stylesheet: &StyleSheet<'i>,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        let mut reported = FxHashSet::default();
        for rule in &stylesheet.rules.0 {
            self.check_rule(rule, offset, result, &mut reported);
        }
    }
}

impl PreferLogicalProperties {
    fn check_rule(
        &self,
        rule: &LCssRule,
        offset: usize,
        result: &mut CssLintResult,
        reported: &mut FxHashSet<u32>,
    ) {
        match rule {
            LCssRule::Style(style_rule) => {
                self.check_declarations(&style_rule.declarations, offset, result, reported);
            }
            LCssRule::Media(media) => {
                for rule in &media.rules.0 {
                    self.check_rule(rule, offset, result, reported);
                }
            }
            LCssRule::Supports(supports) => {
                for rule in &supports.rules.0 {
                    self.check_rule(rule, offset, result, reported);
                }
            }
            LCssRule::LayerBlock(layer) => {
                for rule in &layer.rules.0 {
                    self.check_rule(rule, offset, result, reported);
                }
            }
            _ => {}
        }
    }

    fn check_declarations(
        &self,
        declarations: &DeclarationBlock,
        offset: usize,
        result: &mut CssLintResult,
        reported: &mut FxHashSet<u32>,
    ) {
        // Check all declarations (both regular and important)
        for decl in declarations.declarations.iter() {
            self.check_property(decl.property_id(), offset, result, reported);
        }
        for decl in declarations.important_declarations.iter() {
            self.check_property(decl.property_id(), offset, result, reported);
        }
    }

    #[inline]
    fn check_property(
        &self,
        prop_id: PropertyId,
        offset: usize,
        result: &mut CssLintResult,
        reported: &mut FxHashSet<u32>,
    ) {
        let (physical, logical) = match prop_id {
            PropertyId::MarginLeft => ("margin-left", "margin-inline-start"),
            PropertyId::MarginRight => ("margin-right", "margin-inline-end"),
            PropertyId::PaddingLeft => ("padding-left", "padding-inline-start"),
            PropertyId::PaddingRight => ("padding-right", "padding-inline-end"),
            PropertyId::BorderLeftWidth => ("border-left-width", "border-inline-start-width"),
            PropertyId::BorderRightWidth => ("border-right-width", "border-inline-end-width"),
            PropertyId::Left => ("left", "inset-inline-start"),
            PropertyId::Right => ("right", "inset-inline-end"),
            _ => return,
        };

        // Report at offset since PropertyId doesn't provide precise location
        let start = offset as u32;
        let end = (offset + physical.len()) as u32;
        if !reported.insert(start) {
            return;
        }
        let _ = logical; // suppress unused warnings
        result.add_diagnostic(
            LintDiagnostic::warn(
                META.name,
                "Consider using logical properties for better RTL support",
                start,
                end,
            )
            .with_help("Use logical properties like margin-inline-start instead of margin-left"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::PreferLogicalProperties;
    use crate::rules::css::CssLinter;

    fn create_linter() -> CssLinter {
        let mut linter = CssLinter::new();
        linter.add_rule(Box::new(PreferLogicalProperties));
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
    fn test_deduplicates_same_fallback_range() {
        let linter = create_linter();
        let result = linter.lint(
            ".button { margin-left: 10px; margin-left: 20px !important; }",
            0,
        );
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
