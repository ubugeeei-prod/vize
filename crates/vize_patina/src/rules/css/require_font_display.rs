//! css/require-font-display
//!
//! Require font-display property in @font-face rules.

use lightningcss::rules::CssRule as LightningRule;
use lightningcss::rules::font_face::FontFaceProperty;
use lightningcss::stylesheet::StyleSheet as LightningStyleSheet;
use vize_l1::css::{StyleNodeKind, StyleSheet};

use crate::diagnostic::{LintDiagnostic, Severity};

use super::style_rule::{identifier_eq, in_style_scope, nested_declaration_name};
use super::{CssLintResult, CssRule, CssRuleMeta, StyleRule};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/require-font-display",
    description: "Require font-display in @font-face rules",
    default_severity: Severity::Warning,
};

// The built-in path is `StyleRule`; retain the public `CssRule` implementation
// for existing callers of `CssLinter::add_rule`.
const _CSS_RULE_COMPATIBILITY: () = ();

/// It reads every `@font-face` in the L1 structure, in every style language,
/// and reports it at its `@font-face` keyword.
pub struct RequireFontDisplay;

impl CssRule for RequireFontDisplay {
    fn meta(&self) -> &'static CssRuleMeta {
        &META
    }

    fn check<'i>(
        &self,
        _source: &'i str,
        stylesheet: &LightningStyleSheet<'i>,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        for rule in &stylesheet.rules.0 {
            self.check_lightning_rule(rule, offset, result);
        }
    }
}

impl RequireFontDisplay {
    fn check_lightning_rule(
        &self,
        rule: &LightningRule,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        match rule {
            LightningRule::FontFace(font_face) => {
                let has_font_display = font_face.properties.iter().any(|property| {
                    matches!(property, FontFaceProperty::Custom(custom)
                        if custom.name.as_ref().eq_ignore_ascii_case("font-display"))
                });
                if !has_font_display {
                    result.add_diagnostic(
                        LintDiagnostic::warn(
                            META.name,
                            "@font-face rule is missing font-display property",
                            offset as u32,
                            (offset + 10) as u32,
                        )
                        .with_help(
                            "Add `font-display: swap;` to prevent FOIT (Flash of Invisible Text)",
                        ),
                    );
                }
            }
            LightningRule::Media(media) => {
                for rule in &media.rules.0 {
                    self.check_lightning_rule(rule, offset, result);
                }
            }
            LightningRule::Supports(supports) => {
                for rule in &supports.rules.0 {
                    self.check_lightning_rule(rule, offset, result);
                }
            }
            LightningRule::LayerBlock(layer) => {
                for rule in &layer.rules.0 {
                    self.check_lightning_rule(rule, offset, result);
                }
            }
            _ => {}
        }
    }
}

impl StyleRule for RequireFontDisplay {
    fn meta(&self) -> &'static CssRuleMeta {
        &META
    }

    fn check(&self, sheet: &StyleSheet<'_>, result: &mut CssLintResult) {
        let nodes = sheet.nodes();
        for (index, node) in nodes.iter().enumerate() {
            let StyleNodeKind::AtRule { name } = sheet.kind(node) else {
                continue;
            };
            if node.block().is_none()
                || !identifier_eq(sheet.text(name), "font-face")
                || !in_style_scope(sheet, node)
            {
                continue;
            }
            // Descendants follow their at-rule in document order.
            let end = node.span().end;
            let has_font_display = nodes
                .get(index + 1..)
                .unwrap_or_default()
                .iter()
                .take_while(|child| child.span().start < end)
                .any(|child| {
                    let StyleNodeKind::Declaration { name, .. } = sheet.kind(child) else {
                        return false;
                    };
                    if child.parent() == Some(index) {
                        return identifier_eq(sheet.text(name), "font-display");
                    }
                    nested_declaration_name(sheet, child).is_some_and(|(name, owner)| {
                        owner == Some(index) && name.eq_ignore_ascii_case("font-display")
                    })
                });
            if !has_font_display {
                result.warn(
                    META.name,
                    "@font-face rule is missing font-display property",
                    "Add `font-display: swap;` to prevent FOIT (Flash of Invisible Text)",
                    node.prelude().start,
                    name.end,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RequireFontDisplay;
    use crate::rules::css::CssLinter;

    fn create_linter() -> CssLinter {
        let mut linter = CssLinter::new();
        linter.add_style_rule(Box::new(RequireFontDisplay));
        linter
    }

    #[test]
    fn test_valid_with_font_display() {
        let linter = create_linter();
        let result = linter.lint(
            "@font-face { font-family: 'MyFont'; font-display: swap; src: url('f.woff2'); }",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_missing_font_display() {
        let linter = create_linter();
        let result = linter.lint(
            "@font-face { font-family: 'MyFont'; src: url('font.woff2'); }",
            0,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_no_font_face() {
        let linter = create_linter();
        let result = linter.lint(".button { color: red; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_accepts_escaped_font_display() {
        let linter = create_linter();
        let result = linter.lint("@font-face { font-\\64 isplay: swap; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_accepts_nested_sass_font_display_only_in_the_font_face() {
        use vize_l1::css::lang::StyleLanguage;

        let linter = create_linter();
        let valid = "@font-face { font: { display: swap; } }";
        assert_eq!(
            linter
                .lint_block(valid, 0, StyleLanguage::Scss, true)
                .warning_count,
            0
        );
        let nested_rule = "@font-face { .unrelated { font-display: swap; } }";
        assert_eq!(
            linter
                .lint_block(nested_rule, 0, StyleLanguage::Scss, true)
                .warning_count,
            1
        );
    }

    #[test]
    fn test_remains_compatible_with_add_rule() {
        let mut linter = CssLinter::new();
        linter.add_rule(Box::new(RequireFontDisplay));
        assert_eq!(
            linter
                .lint("@font-face { font-family: Example; }", 0)
                .warning_count,
            1
        );
    }
}
