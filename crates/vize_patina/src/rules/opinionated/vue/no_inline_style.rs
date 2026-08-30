//! vue/no-inline-style
//!
//! Discourage use of inline style attributes.
//!
//! Inline styles make it harder to maintain consistent styling,
//! can override CSS classes unexpectedly, and reduce code reusability.
//! Prefer using CSS classes, scoped styles, or CSS-in-JS solutions.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div style="color: red">text</div>
//! <span :style="{ color: 'red' }">text</span>
//! <p :style="dynamicStyles">text</p>
//! ```
//!
//! ### Valid
//! ```vue
//! <div class="text-red">text</div>
//! <span :class="{ 'text-red': isRed }">text</span>
//! ```
//!
//! ### Exceptions
//! Dynamic styles for animations, canvas-like positioning, or user-customizable
//! theming may be acceptable exceptions. This rule can be disabled with a comment.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-inline-style",
    description: "Discourage use of inline style attributes",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// No inline style rule
#[derive(Default)]
pub struct NoInlineStyle;

impl NoInlineStyle {
    fn check_binding(ctx: &mut LintContext<'_>, binding: &MarkupBinding<'_>) {
        if !matches!(
            binding.kind(),
            MarkupBindingKind::Attribute | MarkupBindingKind::Bind
        ) || !binding.is_unqualified_arg_exact("style")
        {
            return;
        }

        ctx.warn_at_with_help(
            ctx.t("vue/no-inline-style.message"),
            binding.range(),
            ctx.t("vue/no-inline-style.help"),
        );
    }

    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        element.walk_bindings(&mut |binding| Self::check_binding(ctx, &binding));
    }
}

impl MarkupRule for NoInlineStyle {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        _element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        Self::check_binding(ctx.lint(), binding);
    }
}

impl Rule for NoInlineStyle {
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
    use super::NoInlineStyle;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoInlineStyle));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_class() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="foo">text</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_dynamic_class() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :class="{ active: isActive }">text</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_static_style() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div style="color: red">text</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_dynamic_style() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div :style="{ color: 'red' }">text</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }
}
