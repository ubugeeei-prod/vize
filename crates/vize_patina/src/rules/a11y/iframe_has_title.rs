//! a11y/iframe-has-title
//!
//! Require iframe elements to have a title attribute.
//!
//! Screen readers use the title attribute to describe the iframe content.
//!
//! Based on eslint-plugin-vuejs-accessibility iframe-has-title rule.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "a11y/iframe-has-title",
    description: "Require iframe elements to have a title attribute",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require iframe elements to have a title attribute
#[derive(Default)]
pub struct IframeHasTitle;

impl IframeHasTitle {
    fn has_title(element: &MarkupElement<'_>) -> bool {
        let mut has_title = false;
        element.walk_bindings(&mut |binding| {
            if !binding.is_unqualified_arg_exact("title") {
                return;
            }

            match binding.kind() {
                MarkupBindingKind::Attribute => {
                    if binding
                        .static_value()
                        .is_some_and(|value| !value.trim().is_empty())
                    {
                        has_title = true;
                    }
                }
                MarkupBindingKind::Bind => {
                    has_title = true;
                }
                MarkupBindingKind::On | MarkupBindingKind::Model | MarkupBindingKind::Custom => {}
            }
        });
        has_title
    }

    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        if !element.is_unqualified_tag_exact("iframe") {
            return;
        }

        if !Self::has_title(element) {
            ctx.warn_at_with_help(
                ctx.t("a11y/iframe-has-title.message"),
                element.range(),
                ctx.t("a11y/iframe-has-title.help"),
            );
        }
    }
}

impl MarkupRule for IframeHasTitle {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check_element(ctx.lint(), element);
    }
}

impl Rule for IframeHasTitle {
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
    use super::IframeHasTitle;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(IframeHasTitle));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_with_title() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<iframe src="https://example.com" title="Example website"></iframe>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_dynamic_title() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<iframe src="https://example.com" :title="frameTitle"></iframe>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_no_title() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<iframe src="https://example.com"></iframe>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_empty_title() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<iframe src="https://example.com" title=""></iframe>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }
}
