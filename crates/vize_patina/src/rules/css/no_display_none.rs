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
use lightningcss::properties::Property;
use lightningcss::properties::display::{Display, DisplayKeyword};
use lightningcss::rules::CssRule as LCssRule;
use lightningcss::selector::{Combinator, Component, PseudoClass, PseudoElement, Selector};
use lightningcss::stylesheet::StyleSheet;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::declaration_positions::DeclarationPositions;
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
        source: &'i str,
        stylesheet: &StyleSheet<'i>,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        for rule in &stylesheet.rules.0 {
            self.check_rule(rule, source, offset, false, result);
        }
    }
}

impl NoDisplayNone {
    #[inline]
    fn check_rule(
        &self,
        rule: &LCssRule,
        source: &str,
        offset: usize,
        inherited_external: bool,
        result: &mut CssLintResult,
    ) {
        match rule {
            LCssRule::Style(style_rule) => {
                let is_pseudo = style_rule.selectors.0.iter().any(|selector| {
                    selector
                        .iter()
                        .any(|component| matches!(component, Component::PseudoElement(_)))
                });
                let external = style_rule
                    .selectors
                    .0
                    .iter()
                    .all(|selector| external_target(selector, inherited_external));
                if !is_pseudo && !external {
                    let mut positions = DeclarationPositions::new(source, style_rule);
                    self.check_declarations(
                        &style_rule.declarations,
                        &mut positions,
                        offset,
                        result,
                    );
                }
                for rule in &style_rule.rules.0 {
                    self.check_rule(rule, source, offset, external, result);
                }
            }
            LCssRule::LayerBlock(layer) => {
                for rule in &layer.rules.0 {
                    self.check_rule(rule, source, offset, inherited_external, result);
                }
            }
            _ => {}
        }
    }

    #[inline]
    fn check_declarations(
        &self,
        declarations: &DeclarationBlock,
        positions: &mut DeclarationPositions,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        // Check all declarations
        for decl in declarations.declarations.iter() {
            self.check_property(decl, positions, offset, result);
        }
        for decl in declarations.important_declarations.iter() {
            self.check_property(decl, positions, offset, result);
        }
    }

    #[inline]
    fn check_property(
        &self,
        property: &Property,
        positions: &mut DeclarationPositions,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        if let Property::Display(display) = property {
            let is_none = matches!(display, Display::Keyword(DisplayKeyword::None));

            if is_none {
                let Some((start, end)) = positions.take("display", offset) else {
                    return;
                };
                result.add_diagnostic(
                    LintDiagnostic::warn(
                        META.name,
                        "Consider using v-show directive instead of display: none",
                        start,
                        end,
                    )
                    .with_help(
                        "v-show toggles visibility without removing from DOM, improving performance for frequent toggles",
                    ),
                );
            }
        }
    }
}

/// Match the selected subject, not names in strings or a :has()/:not() filter.
/// Mixed selector lists still warn for their local target. A sibling step does
/// not establish that the selected element belongs to the preceding deep target.
fn external_target(selector: &Selector<'_>, inherited: bool) -> bool {
    for component in selector.iter_raw_match_order() {
        match component {
            Component::NonTSPseudoClass(PseudoClass::CustomFunction { name, arguments })
                if matches!(name.as_ref(), "deep" | "slotted") && !arguments.0.is_empty() =>
            {
                return true;
            }
            Component::PseudoElement(
                PseudoElement::Custom { name } | PseudoElement::CustomFunction { name, .. },
            ) if matches!(name.as_ref(), "v-deep" | "v-slotted") => return true,
            Component::Slotted(_) => return true,
            Component::Is(selectors) | Component::Where(selectors)
                if selectors
                    .iter()
                    .all(|selector| external_target(selector, false)) =>
            {
                return true;
            }
            Component::Combinator(Combinator::NextSibling | Combinator::LaterSibling) => {
                return false;
            }
            Component::Nesting => return inherited,
            _ => {}
        }
    }
    inherited
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
    fn test_warns_nested_display_none() {
        let linter = create_linter();
        let result = linter.lint(".card { .title { display: none; } }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_visibility_hidden() {
        let linter = create_linter();
        let result = linter.lint(".hidden { visibility: hidden; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_skips_pseudo_elements_and_conditional_rules() {
        let source = ".marker::after { display: none; }\n@media print { .button { display: none; } }\n@supports (display: grid) { .button { display: none; } }";
        let result = create_linter().lint(source, 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_reports_plain_element_at_declaration() {
        let source = ".marker { display: block; }\n.button { display: none; }";
        let result = create_linter().lint(source, 50);
        assert_eq!(result.warning_count, 1);
        assert_eq!(
            result.diagnostics[0].start as usize,
            50 + source.rfind("display").unwrap()
        );
    }
}
