//! vue/require-component-is
//!
//! Require `v-bind:is` attribute on `<component>` elements.
//!
//! The `<component>` element is a dynamic component that requires an `is` or
//! `:is` attribute to determine which component to render.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <component />
//! <component></component>
//! ```
//!
//! ### Valid
//! ```vue
//! <component :is="currentComponent" />
//! <component :is="'MyComponent'" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupElement, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/require-component-is",
    description: "Require `v-bind:is` on `<component>` elements",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

#[derive(Default)]
/// Implements the `vue/require-component-is` lint rule.
pub struct RequireComponentIs;

impl RequireComponentIs {
    fn has_is_attribute(element: &MarkupElement<'_>) -> bool {
        let mut found = false;
        element.walk_directives(&mut |directive| {
            if directive.name_eq("bind") && directive.is_static_arg_exact("is") {
                found = true;
            }
        });
        found
    }

    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        if element.is_unqualified_tag_exact("component") && !Self::has_is_attribute(element) {
            ctx.error_at_with_help(
                ctx.t("vue/require-component-is.message"),
                element.range(),
                ctx.t("vue/require-component-is.help"),
            );
        }
    }
}

impl MarkupRule for RequireComponentIs {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check_element(ctx.lint(), element);
    }
}

impl Rule for RequireComponentIs {
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
    use super::RequireComponentIs;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(RequireComponentIs));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_with_bind_is() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<component :is="currentComponent" />"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_with_static_is() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<component is="MyComponent" />"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_no_is() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<component />"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_dynamic_is_argument() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<component :[is]="currentComponent" />"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_no_is_with_children() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<component>content</component>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_not_component_tag() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }
}
