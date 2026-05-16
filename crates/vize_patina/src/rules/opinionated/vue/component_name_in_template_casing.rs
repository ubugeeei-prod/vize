//! vue/component-name-in-template-casing
//!
//! Enforce specific casing for component names in templates.
//!
//! ## Examples
//!
//! ### Invalid (default: PascalCase)
//! ```vue
//! <my-component />
//! <myComponent />
//! ```
//!
//! ### Valid
//! ```vue
//! <MyComponent />
//! <RouterView />
//! <slot />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::{is_html_tag, is_svg_tag};
use vize_croquis::builtins::is_builtin_component;
use vize_croquis::naming::{is_kebab_case_loose, is_pascal_case};
use vize_relief::ast::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/component-name-in-template-casing",
    description: "Enforce specific casing for component names in templates",
    category: RuleCategory::Recommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Casing style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComponentCasing {
    /// PascalCase: MyComponent
    #[default]
    PascalCase,
    /// kebab-case: my-component
    KebabCase,
}

/// Component name in template casing rule
pub struct ComponentNameInTemplateCasing {
    pub casing: ComponentCasing,
}

impl Default for ComponentNameInTemplateCasing {
    fn default() -> Self {
        Self {
            casing: ComponentCasing::PascalCase,
        }
    }
}

impl Rule for ComponentNameInTemplateCasing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        let tag = element.tag.as_str();

        // Skip HTML elements, SVG elements, and Vue built-ins
        let tag_lower = tag.to_lowercase();
        if is_html_tag(&tag_lower)
            || is_svg_tag(tag)
            || is_builtin_component(tag)
            || is_builtin_component(&tag_lower)
        {
            return;
        }

        match self.casing {
            ComponentCasing::PascalCase => {
                if !is_pascal_case(tag) {
                    ctx.warn_with_help(
                        ctx.t("vue/component-name-in-template-casing.pascal"),
                        &element.loc,
                        ctx.t("vue/component-name-in-template-casing.help_pascal"),
                    );
                }
            }
            ComponentCasing::KebabCase => {
                if !is_kebab_case_loose(tag) {
                    ctx.warn_with_help(
                        ctx.t("vue/component-name-in-template-casing.kebab"),
                        &element.loc,
                        ctx.t("vue/component-name-in-template-casing.help_kebab"),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentNameInTemplateCasing;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ComponentNameInTemplateCasing::default()));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_pascal_case() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_kebab_case() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<my-component />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_html_element() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_vue_built_in() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<slot />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
