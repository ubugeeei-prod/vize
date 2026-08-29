//! a11y/placeholder-label-option
//!
//! Validate `<select>` placeholder label requirements.
//! The first `<option>` with an empty `value` attribute should have
//! `disabled` or `hidden` to be a proper placeholder.
//! Based on markuplint's `placeholder-label-option` rule.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <select>
//!     <option value="">Choose one</option>
//!     <option value="a">A</option>
//!   </select>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <select>
//!     <option value="" disabled>Choose one</option>
//!     <option value="a">A</option>
//!   </select>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupNode, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "a11y/placeholder-label-option",
    description: "Require disabled or hidden on select placeholder option",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
pub struct PlaceholderLabelOption;

impl PlaceholderLabelOption {
    fn first_option_child<'a>(
        element: &MarkupElement<'a>,
        transparent_fragments: bool,
    ) -> Option<MarkupElement<'a>> {
        let mut first_option = None;
        element.walk_children(&mut |child| {
            if first_option.is_none()
                && let MarkupNode::Element(element) = child
                && element.is_unqualified_tag_exact("option")
            {
                first_option = Some(element);
            }
            if first_option.is_none()
                && transparent_fragments
                && let MarkupNode::Element(element) = child
                && element.tag().is_empty()
            {
                first_option = Self::first_option_child(&element, transparent_fragments);
            }
        });
        first_option
    }

    fn has_empty_static_attribute(element: &MarkupElement<'_>, name: &str) -> bool {
        let mut found = false;
        element.walk_bindings(&mut |binding| {
            if binding.kind() == MarkupBindingKind::Attribute
                && binding.is_unqualified_arg_exact(name)
                && binding.static_value().is_none_or(str::is_empty)
            {
                found = true;
            }
        });
        found
    }

    fn has_exact_static_attribute(element: &MarkupElement<'_>, name: &str) -> bool {
        let mut found = false;
        element.walk_bindings(&mut |binding| {
            if binding.kind() == MarkupBindingKind::Attribute
                && binding.is_unqualified_arg_exact(name)
            {
                found = true;
            }
        });
        found
    }

    fn check_element(
        ctx: &mut LintContext<'_>,
        element: &MarkupElement<'_>,
        transparent_fragments: bool,
    ) {
        if element.is_component() || !element.is_unqualified_tag_exact("select") {
            return;
        }

        let Some(option) = Self::first_option_child(element, transparent_fragments) else {
            return;
        };

        if !Self::has_empty_static_attribute(&option, "value") {
            return;
        }

        if Self::has_exact_static_attribute(&option, "disabled")
            || Self::has_exact_static_attribute(&option, "hidden")
        {
            return;
        }

        let message = ctx.t("a11y/placeholder-label-option.message");
        let help = ctx.t("a11y/placeholder-label-option.help");
        ctx.warn_at_with_help(message, option.range(), help);
    }
}

impl MarkupRule for PlaceholderLabelOption {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        let transparent_fragments = ctx.is_jsx();
        Self::check_element(ctx.lint(), element, transparent_fragments);
    }
}

impl Rule for PlaceholderLabelOption {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check_element(ctx, &MarkupElement::new(element), false);
    }
}

#[cfg(test)]
mod tests {
    use super::PlaceholderLabelOption;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(PlaceholderLabelOption));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_disabled_placeholder() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<select><option value="" disabled>Choose</option><option value="a">A</option></select>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_hidden_placeholder() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<select><option value="" hidden>Choose</option><option value="a">A</option></select>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_placeholder() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<select><option value="a">A</option><option value="b">B</option></select>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_options() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<select></select>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_not_select() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>text</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_no_disabled_or_hidden() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<select><option value="">Choose</option><option value="a">A</option></select>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }
}
