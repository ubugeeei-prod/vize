//! a11y/heading-has-content
//!
//! Require heading elements (h1-h6) to have accessible content.
//!
//! Empty headings are not accessible to screen reader users.
//!
//! Based on eslint-plugin-vuejs-accessibility heading-has-content rule.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{
    MarkupBindingKind, MarkupContext, MarkupElement, MarkupElementKind, MarkupNode, MarkupRule,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "a11y/heading-has-content",
    description: "Require heading elements to have accessible content",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require heading elements to have accessible content
#[derive(Default)]
pub struct HeadingHasContent;

impl HeadingHasContent {
    fn is_heading(element: &MarkupElement<'_>) -> bool {
        ["h1", "h2", "h3", "h4", "h5", "h6"]
            .iter()
            .any(|tag| element.is_unqualified_tag_exact(tag))
    }

    fn has_accessible_name(element: &MarkupElement<'_>) -> bool {
        let mut found = false;
        element.walk_bindings(&mut |binding| {
            if found {
                return;
            }

            if matches!(
                binding.kind(),
                MarkupBindingKind::Attribute | MarkupBindingKind::Bind
            ) && (binding.is_unqualified_arg_exact("aria-label")
                || binding.is_unqualified_arg_exact("aria-labelledby"))
            {
                found = true;
            }
        });
        found
    }

    fn is_hidden_from_accessibility_tree(element: &MarkupElement<'_>) -> bool {
        let mut hidden = false;
        element.walk_bindings(&mut |binding| {
            if hidden {
                return;
            }

            if binding.kind() == MarkupBindingKind::Attribute
                && binding.is_unqualified_arg_exact("aria-hidden")
                && binding.static_value() == Some("true")
            {
                hidden = true;
            }
        });
        hidden
    }

    fn has_accessible_content(element: &MarkupElement<'_>) -> bool {
        if Self::has_accessible_name(element) {
            return true;
        }

        let mut has_content = false;
        element.walk_children(&mut |child| match child {
            MarkupNode::Text(text) if text.is_significant() => {
                has_content = true;
            }
            MarkupNode::Interpolation(_) => {
                has_content = true;
            }
            MarkupNode::Element(child_element)
                if child_element.kind() == MarkupElementKind::Slot
                    || child_element.is_unqualified_tag_exact("slot") =>
            {
                has_content = true;
            }
            MarkupNode::Element(child_element) if Self::has_accessible_content(&child_element) => {
                has_content = true;
            }
            MarkupNode::Text(_)
            | MarkupNode::Element(_)
            | MarkupNode::Comment(_)
            | MarkupNode::If(_)
            | MarkupNode::For(_)
            | MarkupNode::Other(_) => {}
        });

        has_content
    }

    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        if !Self::is_heading(element) || Self::is_hidden_from_accessibility_tree(element) {
            return;
        }

        if !Self::has_accessible_content(element) {
            ctx.warn_at_with_help(
                ctx.t_fmt(
                    "a11y/heading-has-content.message",
                    &[("tag", element.tag())],
                ),
                element.range(),
                ctx.t("a11y/heading-has-content.help"),
            );
        }
    }
}

impl MarkupRule for HeadingHasContent {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if ctx.is_jsx_attribute_value() {
            return;
        }

        Self::check_element(ctx.lint(), element);
    }
}

impl Rule for HeadingHasContent {
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
    use super::HeadingHasContent;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(HeadingHasContent));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_with_text() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1>Hello World</h1>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_interpolation() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h2>{{ title }}</h2>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_aria_hidden() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1 aria-hidden="true"></h1>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_static_aria_label() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1 aria-label="Dashboard"></h1>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_bound_aria_label() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1 :aria-label="title"></h1>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_bound_aria_labelledby() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1 :aria-labelledby="labelId"></h1>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_empty() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1></h1>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_with_default_slot() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1><slot></slot></h1>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
