//! vue/no-deprecated-html-element-is
//!
//! Disallow the `is` attribute on native HTML elements (removed in Vue 3).
//!
//! Vue 2 let you swap a native element for a component by writing
//! `<div is="MyComponent">` (component substitution). Vue 3 removed that
//! behaviour: `is` on a native element is now interpreted as the standard DOM
//! attribute, so the component is never mounted. To opt back into the legacy
//! "customized built-in element" lookup you must prefix the value with `vue:`
//! (`<div is="vue:MyComponent">`); for true dynamic components use
//! `<component :is="...">`.
//!
//! This mirrors eslint-plugin-vue's `vue/no-deprecated-html-element-is`. It is
//! an opt-in migration rule and only fires for the default Vue 3 dialect.
//!
//! ## Examples
//!
//! ### Invalid (Vue 3)
//! ```vue
//! <template>
//!   <div is="MyComponent" />
//! </template>
//! ```
//!
//! ### Valid (Vue 3)
//! ```vue
//! <template>
//!   <component :is="MyComponent" />
//!   <div is="vue:MyComponent" />
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::dialect::VueDialect;
use vize_relief::{ElementNode, ElementType, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-html-element-is",
    description: "Disallow the `is` attribute on native HTML elements",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow the `is` attribute on native HTML elements.
pub struct NoDeprecatedHtmlElementIs;

impl Rule for NoDeprecatedHtmlElementIs {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Only the default Vue 3 dialect removed native-element `is` substitution.
        if ctx.dialect() != VueDialect::Vue {
            return;
        }

        // Skip components (PascalCase / built-ins): `is` on a real component is
        // not the removed native-element substitution.
        if element.tag_type != ElementType::Element {
            return;
        }

        // `<component is="...">` is the valid dynamic-component element.
        if element.tag.as_str() == "component" {
            return;
        }

        for prop in element.props.iter() {
            if let PropNode::Attribute(attr) = prop
                && attr.name == "is"
            {
                // `is="vue:Foo"` opts into the customized built-in lookup and is
                // still supported in Vue 3, so it must not be flagged.
                let is_vue_prefixed = attr
                    .value
                    .as_ref()
                    .is_some_and(|v| v.content.as_str().starts_with("vue:"));
                if is_vue_prefixed {
                    continue;
                }

                ctx.error_with_help(
                    ctx.t("vue/no-deprecated-html-element-is.message"),
                    &attr.loc,
                    ctx.t("vue/no-deprecated-html-element-is.help"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedHtmlElementIs;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDeprecatedHtmlElementIs));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_is_on_native_element() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div is="MyComponent" />"#, "App.vue");
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_is_on_native_element_with_children() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<table><tr is="my-row"></tr></table>"#, "App.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn allows_component_element_with_static_is() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<component is="MyComponent" />"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn allows_component_element_with_bind_is() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<component :is="currentComponent" />"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn allows_vue_prefixed_is() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div is="vue:MyComponent" />"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn allows_dynamic_is_binding_on_native_element() {
        // A bound `:is` is a directive, not the removed static substitution.
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :is="something" />"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_components() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent is="Other" />"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_native_element_without_is() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="x" />"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }
}
