//! vue/no-template-shadow
//!
//! Disallow variable names that shadow Vue component properties.
//!
//! When a v-for variable shadows a component property or another v-for variable,
//! it can lead to confusing behavior.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <!-- If 'item' is a component property -->
//! <div v-for="item in items">{{ item }}</div>
//!
//! <!-- Nested v-for with same variable name -->
//! <div v-for="item in items">
//!   <div v-for="item in item.children">{{ item }}</div>
//! </div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-for="item in items">
//!   <div v-for="child in item.children">{{ child }}</div>
//! </div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::visitor::parse_v_for_variables;
use vize_relief::ast::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-template-shadow",
    description: "Disallow variable names that shadow variables in outer scope",
    category: RuleCategory::StronglyRecommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow template variable shadowing
pub struct NoTemplateShadow;

impl Rule for NoTemplateShadow {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "for" {
            return;
        }

        // Get the v-for variables
        let Some(ref exp) = directive.exp else {
            return;
        };

        let vars = parse_v_for_variables(exp);

        // Check each variable against PARENT scope (not current scope, which includes our own vars)
        // We use is_parent_v_for_var instead of is_v_for_var to avoid checking against ourselves
        for var in &vars {
            let var_name = var.as_str();
            if ctx.is_parent_v_for_var(var_name) {
                ctx.warn_with_help(
                    ctx.t_fmt("vue/no-template-shadow.message", &[("name", var_name)]),
                    &directive.loc,
                    ctx.t("vue/no-template-shadow.help"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoTemplateShadow;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoTemplateShadow));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_no_shadow() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child }}</span></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_nested_shadow() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item }}</span></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
        assert!(result.diagnostics[0].message.contains("shadows"));
    }

    #[test]
    fn test_valid_different_names() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="(item, index) in items" :key="index"><span v-for="(child, childIndex) in item.children" :key="childIndex">{{ child }}</span></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }
}
