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
use crate::diagnostic::{HelpLevel, Severity};
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
    fn is_hidden_element(element: &MarkupElement<'_>) -> bool {
        !element.is_component()
            && markup_helpers::get_static_markup_attribute_value(element, "aria-hidden")
                == Some("true")
    }

    fn check_hidden_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>, inert: bool) {
        if inert
            || markup_helpers::is_statically_non_focusable(element)
            || !markup_helpers::is_focusable_markup_element(element)
        {
            return;
        }

        let full_help = ctx.t("a11y/no-aria-hidden-on-focusable.help");
        let help = if markup_helpers::get_static_markup_attribute_value(element, "tabindex")
            == Some("-1")
        {
            HelpLevel::Full.process(&full_help)
        } else {
            HelpLevel::Short.process(&full_help)
        };
        ctx.error_at_with_help(
            ctx.t("a11y/no-aria-hidden-on-focusable.message"),
            element.range(),
            help.unwrap_or_default(),
        );
    }
}

impl MarkupRule for NoAriaHiddenOnFocusable {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if !Self::is_hidden_element(element) {
            return;
        }
        let mut inert = markup_helpers::has_static_inert(element);
        if !inert && !markup_helpers::blocks_inert_inheritance(element) {
            for ancestor in ctx.ancestor_elements().rev() {
                if markup_helpers::has_static_inert(&ancestor) {
                    inert = true;
                    break;
                }
                if markup_helpers::blocks_inert_inheritance(&ancestor) {
                    break;
                }
            }
        }
        Self::check_hidden_element(ctx.lint(), element, inert);
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
        let element = MarkupElement::new(element);
        if !Self::is_hidden_element(&element) {
            return;
        }
        let inert = ctx.aria_hidden_inert || markup_helpers::has_static_inert(&element);
        Self::check_hidden_element(ctx, &element, inert);
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
    fn test_invalid_aria_hidden_on_input_with_tabindex_minus_one() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<input class="sizer" readonly tabindex="-1" aria-hidden="true" />"#,
            "test.vue",
        );
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
