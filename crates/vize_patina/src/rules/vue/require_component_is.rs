//! vue/require-component-is
//!
//! Require an `is` target on `<component>` elements.
//!
//! The `<component>` element is a dynamic component that requires an `is` or
//! `:is` attribute to determine which component to render.
//!
//! A static `is` is a valid target: Vue compiles `<component is="MyComponent">`
//! to `resolveDynamicComponent("MyComponent")`, which resolves the registered
//! component by name. eslint-plugin-vue reports it and asks for `v-bind:is`;
//! this rule does not, because the static form renders the named component.
//! An empty static `is` names no component and is still reported.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <component />
//! <component></component>
//! <component is="" />
//! ```
//!
//! ### Valid
//! ```vue
//! <component :is="currentComponent" />
//! <component :is="'MyComponent'" />
//! <component is="MyComponent" />
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
        if !found {
            element.walk_attributes(&mut |attribute| {
                // A blank value resolves to no component, so it is not a target.
                found |= attribute.name() == "is"
                    && attribute
                        .value()
                        .is_some_and(|value| !value.trim().is_empty());
            });
        }
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

    // Vue resolves a static `is` as a registered component name.
    #[test]
    fn test_valid_with_static_is() {
        let linter = create_linter();
        for source in [
            r#"<component is="MyComponent" />"#,
            r#"<component is="GlobalBadge" text="by-string" />"#,
            r#"<component is="my-widget"></component>"#,
            r#"<component is="vue:div" />"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(result.error_count, 0, "{source} must be valid");
        }
    }

    // A blank or missing value names no component, and the name is
    // case-sensitive in templates.
    #[test]
    fn test_invalid_with_blank_or_misnamed_static_is() {
        let linter = create_linter();
        for source in [
            r#"<component is="" />"#,
            r#"<component is="  " />"#,
            r#"<component is />"#,
            r#"<component IS="MyComponent" />"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(result.error_count, 1, "{source} must stay reported");
        }
    }

    #[test]
    fn pinned_static_is_sfc() {
        let linter = create_linter();
        let source = include_str!(
            "../../../../../tests/_fixtures/differential/linter/require-component-static-is/StaticIs.vue.txt"
        );
        let result = linter.lint_sfc(source, "StaticIs.vue");
        let reported: Vec<&str> = result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                source
                    .get(diagnostic.start as usize..diagnostic.end as usize)
                    .unwrap()
            })
            .collect();
        assert_eq!(reported, [r#"<component is="" />"#, "<component />"]);
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
