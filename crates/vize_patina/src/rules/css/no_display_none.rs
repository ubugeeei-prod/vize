//! css/no-display-none
//!
//! Warn about `display: none` usage and suggest `v-show` directive.
//!
//! In Vue.js, using `v-show` is often more performant and semantic
//! for toggling visibility, as the element remains in the DOM and
//! Vue can optimize the reactivity.
//!
//! Note: This is a suggestion, not an error. There are valid cases
//! for `display: none` (e.g., print styles, initial hidden state).

use lightningcss::declaration::DeclarationBlock;
use lightningcss::properties::display::{Display, DisplayKeyword};
use lightningcss::properties::Property;
use lightningcss::rules::CssRule as LCssRule;
use lightningcss::stylesheet::StyleSheet;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{CssLintResult, CssRule, CssRuleMeta};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/no-display-none",
    description: "Suggest using v-show instead of display: none",
    default_severity: Severity::Warning,
};

/// No display: none rule
pub struct NoDisplayNone;

impl CssRule for NoDisplayNone {
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

impl NoDisplayNone {
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
        // Check all declarations
        for decl in declarations.declarations.iter() {
            self.check_property(decl, offset, result);
        }
        for decl in declarations.important_declarations.iter() {
            self.check_property(decl, offset, result);
        }
    }

    #[inline]
    fn check_property(&self, property: &Property, offset: usize, result: &mut CssLintResult) {
        if let Property::Display(display) = property {
            let is_none = matches!(display, Display::Keyword(DisplayKeyword::None));

            if is_none {
                result.add_diagnostic(
                    LintDiagnostic::warn(
                        META.name,
                        "Consider using v-show directive instead of display: none",
                        offset as u32,
                        (offset + 13) as u32, // "display: none".len()
                    )
                    .with_help(
                        "v-show toggles visibility without removing from DOM, improving performance for frequent toggles",
                    ),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoDisplayNone;
    use crate::rules::css::CssLinter;

    fn create_linter() -> CssLinter {
        let mut linter = CssLinter::new();
        linter.add_rule(Box::new(NoDisplayNone));
        linter
    }

    #[test]
    fn test_valid_display_block() {
        let linter = create_linter();
        let result = linter.lint(".button { display: block; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warns_display_none() {
        let linter = create_linter();
        let result = linter.lint(".hidden { display: none; }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_visibility_hidden() {
        let linter = create_linter();
        let result = linter.lint(".hidden { visibility: hidden; }", 0);
        assert_eq!(result.warning_count, 0);
    }
}
