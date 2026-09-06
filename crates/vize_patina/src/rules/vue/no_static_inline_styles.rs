//! vue/no-static-inline-styles
//!
//! Disallow static inline `style` attributes.
//!
//! This follows eslint-plugin-vue's `vue/no-static-inline-styles` surface:
//! literal `style="..."` attributes are reported, while dynamic style bindings
//! such as `:style="..."` remain valid.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-static-inline-styles",
    description: "Disallow static inline style attributes",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
pub struct NoStaticInlineStyles;

impl NoStaticInlineStyles {
    fn check_binding(ctx: &mut LintContext<'_>, binding: &MarkupBinding<'_>) {
        if binding.kind() != MarkupBindingKind::Attribute
            || !binding.is_unqualified_arg_exact("style")
            || binding.static_value().is_none()
        {
            return;
        }

        ctx.warn_at_with_help(
            "Static inline style attributes are not allowed.",
            binding.range(),
            "Move the style to a class, CSS module, or dynamic style binding.",
        );
    }

    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        element.walk_bindings(&mut |binding| Self::check_binding(ctx, &binding));
    }
}

impl MarkupRule for NoStaticInlineStyles {
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

impl Rule for NoStaticInlineStyles {
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
    use super::NoStaticInlineStyles;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoStaticInlineStyles));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_static_style_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div style="color: red">text</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert_eq!(
            result.diagnostics[0].rule_name,
            "vue/no-static-inline-styles"
        );
    }

    #[test]
    fn allows_dynamic_style_binding() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :style="{ color: theme.color }">text</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_v_bind_style_binding() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-bind:style="styles">text</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_class_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="text-red">text</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
