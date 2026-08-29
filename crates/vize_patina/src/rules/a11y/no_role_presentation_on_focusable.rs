//! a11y/no-role-presentation-on-focusable
//!
//! Disallow `role="presentation"` or `role="none"` on focusable elements.
//!
//! Using `role="presentation"` or `role="none"` on a focusable element
//! removes its semantic meaning from the accessibility tree while it
//! remains focusable, creating a confusing experience for screen reader users.
//!
//! Based on eslint-plugin-vuejs-accessibility no-role-presentation-on-focusable rule.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

use super::markup_helpers;

static META: RuleMeta = RuleMeta {
    name: "a11y/no-role-presentation-on-focusable",
    description: "Disallow role=\"presentation\" or role=\"none\" on focusable elements",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow role="presentation" or role="none" on focusable elements
#[derive(Default)]
pub struct NoRolePresentationOnFocusable;

impl NoRolePresentationOnFocusable {
    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        if element.is_component() {
            return;
        }

        if let Some(role) = markup_helpers::get_static_markup_attribute_value(element, "role")
            && matches!(role, "presentation" | "none")
            && markup_helpers::is_focusable_markup_element(element)
        {
            ctx.error_at_with_help(
                ctx.t("a11y/no-role-presentation-on-focusable.message"),
                element.range(),
                ctx.t("a11y/no-role-presentation-on-focusable.help"),
            );
        }
    }
}

impl MarkupRule for NoRolePresentationOnFocusable {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check_element(ctx.lint(), element);
    }
}

impl Rule for NoRolePresentationOnFocusable {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check_element(ctx, &MarkupElement::new(element));
    }
}

#[cfg(test)]
mod tests {
    use super::NoRolePresentationOnFocusable;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoRolePresentationOnFocusable));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_role_presentation_on_non_focusable() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div role="presentation"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_role_presentation_on_anchor_without_href() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<a role="presentation">decorative</a>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_role_presentation_on_anchor_with_href() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<a href="/" role="presentation">Home</a>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_role_presentation_on_anchor_with_bound_href() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<a :href="url" role="presentation">Home</a>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_role_presentation_on_anchor_with_dynamic_href_argument() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<a :[href]="url" role="presentation">Home</a>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_role_presentation_on_button() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<button role="presentation">Click</button>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_role_none_on_input() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input role="none" type="text" />"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
