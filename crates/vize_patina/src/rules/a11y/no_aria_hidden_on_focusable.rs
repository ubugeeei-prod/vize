//! a11y/no-aria-hidden-on-focusable
//!
//! Disallow `aria-hidden="true"` on focusable elements.
//!
//! Using `aria-hidden="true"` on a focusable element hides it from
//! assistive technologies while it remains focusable by keyboard,
//! creating a confusing experience for screen reader users.
//!
//! Based on eslint-plugin-vuejs-accessibility no-aria-hidden-on-focusable rule.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

use super::markup_helpers;

static META: RuleMeta = RuleMeta {
    name: "a11y/no-aria-hidden-on-focusable",
    description: "Disallow aria-hidden=\"true\" on focusable elements",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow aria-hidden="true" on focusable elements
#[derive(Default)]
pub struct NoAriaHiddenOnFocusable;

impl NoAriaHiddenOnFocusable {
    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        if element.is_component() {
            return;
        }

        if let Some(value) =
            markup_helpers::get_static_markup_attribute_value(element, "aria-hidden")
            && value == "true"
            && markup_helpers::is_focusable_markup_element(element)
        {
            ctx.error_at_with_help(
                ctx.t("a11y/no-aria-hidden-on-focusable.message"),
                element.range(),
                ctx.t("a11y/no-aria-hidden-on-focusable.help"),
            );
        }
    }
}

impl MarkupRule for NoAriaHiddenOnFocusable {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check_element(ctx.lint(), element);
    }
}

impl Rule for NoAriaHiddenOnFocusable {
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
    use super::NoAriaHiddenOnFocusable;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoAriaHiddenOnFocusable));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_aria_hidden_on_non_focusable() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div aria-hidden="true"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_aria_hidden_on_anchor_without_href() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<a aria-hidden="true">decorative</a>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_aria_hidden_on_anchor_with_href() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<a href="/" aria-hidden="true">Home</a>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_aria_hidden_on_anchor_with_bound_href() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<a :href="url" aria-hidden="true">Home</a>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_aria_hidden_on_anchor_with_dynamic_href_argument() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<a :[href]="url" aria-hidden="true">Home</a>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_aria_hidden_on_button() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<button aria-hidden="true">Click</button>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_aria_hidden_false_on_button() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<button aria-hidden="false">Click</button>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }
}
