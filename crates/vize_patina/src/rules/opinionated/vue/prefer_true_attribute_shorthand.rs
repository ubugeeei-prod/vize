//! vue/prefer-true-attribute-shorthand
//!
//! Prefer the shorthand for a boolean attribute bound to `true`.
//!
//! On a component, `:visible="true"` is more concisely written as `visible`.
//! Native HTML attributes are outside this Vue component-prop style rule.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <MyComponent :visible="true" />
//! ```
//!
//! ### Valid
//! ```vue
//! <MyComponent visible />
//! <MyComponent :visible="false" />
//! <MyComponent :visible="isVisible" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::MarkupElement;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/prefer-true-attribute-shorthand",
    description: "Prefer the shorthand for a boolean attribute bound to `true`",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Prefer the shorthand for a boolean attribute bound to `true`.
pub struct PreferTrueAttributeShorthand;

impl Rule for PreferTrueAttributeShorthand {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if !matches!(directive.name, "bind" | "model") {
            return;
        }
        // Only a static argument (`:foo`), not `v-bind="obj"`.
        let Some(ExpressionNode::Simple(arg)) = &directive.arg else {
            return;
        };
        if !arg.is_static {
            return;
        }
        let name = arg.content;
        if !MarkupElement::new(element).is_component() {
            return;
        }
        let is_true =
            matches!(&directive.exp, Some(ExpressionNode::Simple(s)) if s.content.trim() == "true");
        if is_true {
            ctx.warn_with_help(
                ctx.t_fmt(
                    "vue/prefer-true-attribute-shorthand.message",
                    &[("name", name)],
                ),
                &directive.loc,
                ctx.t("vue/prefer-true-attribute-shorthand.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PreferTrueAttributeShorthand;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(PreferTrueAttributeShorthand));
        Linter::with_registry(registry)
    }

    #[test]
    fn allows_native_boolean_true_binding() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input :disabled="true" />"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_dynamic_arguments() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent :[prop]="true" v-bind:[other]="true" />"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_native_non_boolean_true_bindings() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :aria-hidden="true" :data-active="true" />"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_component_true_binding() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<Draggable :list="[]" item-key="id" :force-fallback="true" />"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_shorthand() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent visible />"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_false_and_dynamic() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent :visible="false" :open="isOpen" />"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }
}
