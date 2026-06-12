//! a11y/no-distracting-elements
//!
//! Disallow distracting elements like <marquee> and <blink>.
//!
//! These elements can cause accessibility issues, particularly for users
//! with attention disorders or vestibular motion disorders.
//!
//! Based on eslint-plugin-vuejs-accessibility no-distracting-elements rule.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "a11y/no-distracting-elements",
    description: "Disallow distracting elements like <marquee> and <blink>",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow distracting elements
#[derive(Default)]
pub struct NoDistractingElements;

impl NoDistractingElements {
    fn is_distracting(tag: &str) -> bool {
        matches!(tag, "marquee" | "blink")
    }
}

/// Markup-IR entry point for `a11y/no-distracting-elements`.
///
/// A pure tag-name rule, so it maps one-to-one across backends: a Vue
/// `<marquee>` and a JSX `<marquee />` both reach the same intrinsic tag check,
/// and the diagnostic range addresses the original syntax. Components are exempt
/// on both sides ([`MarkupElement::is_component`]), so a JSX `<Marquee/>` — a
/// user component, not the intrinsic element — is never flagged.
impl MarkupRule for NoDistractingElements {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if element.is_component() {
            return;
        }
        let tag = element.tag();
        if Self::is_distracting(tag) {
            let message = ctx
                .lint()
                .t_fmt("a11y/no-distracting-elements.message", &[("tag", tag)]);
            let help = ctx.lint().t("a11y/no-distracting-elements.help");
            ctx.lint().warn_at_with_help(message, element.range(), help);
        }
    }
}

impl Rule for NoDistractingElements {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if Self::is_distracting(&element.tag) {
            ctx.warn_with_help(
                ctx.t_fmt(
                    "a11y/no-distracting-elements.message",
                    &[("tag", element.tag.as_str())],
                ),
                &element.loc,
                ctx.t("a11y/no-distracting-elements.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoDistractingElements;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDistractingElements));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_normal_elements() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>Hello</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_marquee() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<marquee>Scrolling text</marquee>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_blink() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<blink>Blinking text</blink>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }
}
