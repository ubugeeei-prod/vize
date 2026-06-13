//! vue/prefer-true-attribute-shorthand
//!
//! Prefer the shorthand for a boolean attribute bound to `true`.
//!
//! `:visible="true"` is equivalent to the shorthand `visible`. The explicit
//! `="true"` binding adds noise without changing behaviour.
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
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "bind" {
            return;
        }
        // Only a static argument (`:foo`), not `v-bind="obj"`.
        let Some(ExpressionNode::Simple(arg)) = &directive.arg else {
            return;
        };
        // Modifiers such as `.prop` change semantics; leave them alone.
        if !directive.modifiers.is_empty() {
            return;
        }
        let is_true =
            matches!(&directive.exp, Some(ExpressionNode::Simple(s)) if s.content.trim() == "true");
        if is_true {
            ctx.warn_with_help(
                ctx.t_fmt(
                    "vue/prefer-true-attribute-shorthand.message",
                    &[("name", arg.content.as_str())],
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
    fn reports_true_binding() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent :visible="true" />"#, "App.vue");
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
