//! css/no-hardcoded-values
//!
//! Warn about hardcoded values that could use CSS variables.
//!
//! Design systems benefit from using CSS custom properties (variables)
//! instead of hardcoded values. This improves:
//! - Consistency across the codebase
//! - Theme-ability (dark mode, etc.)
//! - Maintainability
//!
//! This rule checks for:
//! - Hardcoded colors (hex, rgb, hsl)
//! - Hardcoded font sizes
//! - Hardcoded spacing values
//! - Hardcoded z-index values

use lightningcss::declaration::DeclarationBlock;
use lightningcss::properties::Property;
use lightningcss::rules::CssRule as LCssRule;
use lightningcss::stylesheet::StyleSheet;
use lightningcss::values::color::CssColor;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{CssLintResult, CssRule, CssRuleMeta};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/no-hardcoded-values",
    description: "Suggest using CSS variables instead of hardcoded values",
    default_severity: Severity::Warning,
};

/// Configuration for which value types to check
#[derive(Debug, Clone)]
pub struct NoHardcodedValuesConfig {
    /// Check hardcoded colors
    pub colors: bool,
    /// Check hardcoded font sizes
    pub font_sizes: bool,
    /// Check hardcoded spacing (margin, padding)
    pub spacing: bool,
    /// Check hardcoded z-index
    pub z_index: bool,
    /// Allowed hardcoded values (e.g., "0", "inherit", "auto")
    pub allowed: Vec<&'static str>,
}

impl Default for NoHardcodedValuesConfig {
    fn default() -> Self {
        Self {
            colors: true,
            font_sizes: true,
            spacing: false, // Many projects use hardcoded spacing
            z_index: true,
            allowed: vec![
                "0",
                "inherit",
                "initial",
                "unset",
                "auto",
                "none",
                "transparent",
            ],
        }
    }
}

/// No hardcoded values rule
#[derive(Default)]
pub struct NoHardcodedValues {
    config: NoHardcodedValuesConfig,
}

impl NoHardcodedValues {
    /// Create with custom configuration
    pub fn with_config(config: NoHardcodedValuesConfig) -> Self {
        Self { config }
    }
}

impl CssRule for NoHardcodedValues {
    fn meta(&self) -> &'static CssRuleMeta {
        &META
    }

    fn check<'i>(
        &self,
        _source: &'i str,
        stylesheet: &StyleSheet<'i, 'i>,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        for rule in &stylesheet.rules.0 {
            self.check_rule(rule, offset, result);
        }
    }
}

impl NoHardcodedValues {
    #[inline]
    fn check_rule(&self, rule: &LCssRule, offset: usize, result: &mut CssLintResult) {
        match rule {
            LCssRule::Style(style_rule) => {
                self.check_declarations(&style_rule.declarations, offset, result);
            }
            LCssRule::Media(media) => {
                for rule in &media.rules.0 {
                    self.check_rule(rule, offset, result);
                }
            }
            LCssRule::Supports(supports) => {
                for rule in &supports.rules.0 {
                    self.check_rule(rule, offset, result);
                }
            }
            LCssRule::LayerBlock(layer) => {
                for rule in &layer.rules.0 {
                    self.check_rule(rule, offset, result);
                }
            }
            _ => {}
        }
    }

    #[inline]
    fn check_declarations(
        &self,
        declarations: &DeclarationBlock,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        for decl in declarations.declarations.iter() {
            self.check_property(decl, offset, result);
        }
        for decl in declarations.important_declarations.iter() {
            self.check_property(decl, offset, result);
        }
    }

    #[inline]
    fn check_property(&self, property: &Property, offset: usize, result: &mut CssLintResult) {
        // Check colors
        if self.config.colors {
            self.check_color_property(property, offset, result);
        }

        // Check font sizes
        if self.config.font_sizes {
            self.check_font_size_property(property, offset, result);
        }

        // Check z-index
        if self.config.z_index {
            self.check_z_index_property(property, offset, result);
        }
    }

    fn check_color_property(&self, property: &Property, offset: usize, result: &mut CssLintResult) {
        let is_hardcoded = match property {
            Property::Color(color) => self.is_hardcoded_color(color),
            Property::BackgroundColor(color) => self.is_hardcoded_color(color),
            Property::BorderColor(border) => {
                self.is_hardcoded_color(&border.top)
                    || self.is_hardcoded_color(&border.right)
                    || self.is_hardcoded_color(&border.bottom)
                    || self.is_hardcoded_color(&border.left)
            }
            Property::BorderTopColor(color)
            | Property::BorderRightColor(color)
            | Property::BorderBottomColor(color)
            | Property::BorderLeftColor(color) => self.is_hardcoded_color(color),
            _ => false,
        };

        if is_hardcoded {
            result.add_diagnostic(
                LintDiagnostic::warn(
                    META.name,
                    "Consider using a CSS variable for this color value",
                    offset as u32,
                    (offset + 10) as u32,
                )
                .with_help("Use var(--color-name) for consistent theming"),
            );
        }
    }

    fn check_font_size_property(
        &self,
        property: &Property,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        if let Property::FontSize(size) = property {
            // Check if it's a Length type (hardcoded px values)
            let is_hardcoded = matches!(size, lightningcss::properties::font::FontSize::Length(_));

            if is_hardcoded {
                result.add_diagnostic(
                    LintDiagnostic::warn(
                        META.name,
                        "Consider using a CSS variable for font-size",
                        offset as u32,
                        (offset + 10) as u32,
                    )
                    .with_help("Use var(--font-size-name) or relative units (rem, em)"),
                );
            }
        }
    }

    fn check_z_index_property(
        &self,
        property: &Property,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        if let Property::ZIndex(z_index) = property {
            // Check if it's a hardcoded integer (not auto)
            let is_hardcoded = matches!(
                z_index,
                lightningcss::properties::position::ZIndex::Integer(_)
            );

            if is_hardcoded {
                result.add_diagnostic(
                    LintDiagnostic::warn(
                        META.name,
                        "Consider using a CSS variable for z-index",
                        offset as u32,
                        (offset + 8) as u32,
                    )
                    .with_help("Use var(--z-index-name) for consistent layering"),
                );
            }
        }
    }

    #[inline]
    fn is_hardcoded_color(&self, color: &CssColor) -> bool {
        // Check for non-variable colors (CurrentColor and var() are allowed)
        !matches!(color, CssColor::CurrentColor)
    }
}

#[cfg(test)]
mod tests {
    use super::NoHardcodedValues;
    use crate::rules::css::CssLinter;

    fn create_linter() -> CssLinter {
        let mut linter = CssLinter::new();
        linter.add_rule(Box::new(NoHardcodedValues::default()));
        linter
    }

    #[test]
    fn test_valid_css_variable() {
        let linter = create_linter();
        let result = linter.lint(".button { color: var(--primary); }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warns_hardcoded_hex() {
        let linter = create_linter();
        let result = linter.lint(".button { color: #ff0000; }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_warns_hardcoded_rgb() {
        let linter = create_linter();
        let result = linter.lint(".button { color: rgb(255, 0, 0); }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_inherit() {
        let linter = create_linter();
        let result = linter.lint(".button { color: inherit; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_current_color() {
        let linter = create_linter();
        let result = linter.lint(".button { color: currentColor; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warns_hardcoded_z_index() {
        let linter = create_linter();
        let result = linter.lint(".modal { z-index: 9999; }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_z_index_auto() {
        let linter = create_linter();
        let result = linter.lint(".modal { z-index: auto; }", 0);
        assert_eq!(result.warning_count, 0);
    }
}
