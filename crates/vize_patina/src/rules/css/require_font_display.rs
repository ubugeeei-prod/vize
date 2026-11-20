//! css/require-font-display
//!
//! Require font-display property in @font-face rules.

use vize_l1::css::{StyleNodeKind, StyleSheet};

use crate::diagnostic::Severity;

use super::style_rule::{identifier_eq, in_style_scope};
use super::{CssLintResult, CssRuleMeta, StyleRule};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/require-font-display",
    description: "Require font-display in @font-face rules",
    default_severity: Severity::Warning,
};

/// It reads every `@font-face` in the L1 structure, in every style language,
/// and reports it at its `@font-face` keyword.
pub struct RequireFontDisplay;

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
                    child.parent() == Some(index)
                        && matches!(sheet.kind(child), StyleNodeKind::Declaration { name, .. }
                        if identifier_eq(sheet.text(name), "font-display"))
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
        let result = linter.lint("@font-face { font-\\64isplay: swap; }", 0);
        assert_eq!(result.warning_count, 0);
    }
}
