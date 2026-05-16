//! vue/valid-v-on
//!
//! Enforce valid `v-on` directives.
//!
//! `v-on` must:
//! - Have an event name (argument)
//! - Have a handler expression (unless using object syntax)
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-on></div>
//! <div @></div>
//! <div @click></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div @click="handleClick"></div>
//! <div v-on:click="handleClick"></div>
//! <div v-on="{ click: handleClick }"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-on",
    description: "Enforce valid `v-on` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-on directives
pub struct ValidVOn;

impl Rule for ValidVOn {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "on" {
            return;
        }

        let has_arg = directive.arg.is_some();
        let has_exp = directive
            .exp
            .as_ref()
            .map(|e| !is_empty_expression(e))
            .unwrap_or(false);

        // Object syntax: v-on="{ click: handler }"
        if !has_arg && has_exp {
            // This is valid object syntax
            return;
        }

        // Event syntax: @click="handler"
        if has_arg {
            if !has_exp {
                // @click without handler - check if it's an inline listener like @click.prevent
                let has_modifiers = !directive.modifiers.is_empty();
                if !has_modifiers {
                    ctx.error_with_help(
                        ctx.t("vue/valid-v-on.missing_event"),
                        &directive.loc,
                        ctx.t("vue/valid-v-on.help"),
                    );
                }
            }
            return;
        }

        // No argument and no expression
        ctx.error_with_help(
            ctx.t("vue/valid-v-on.missing_event"),
            &directive.loc,
            ctx.t("vue/valid-v-on.help"),
        );
    }
}

/// Check if expression is empty
fn is_empty_expression(exp: &ExpressionNode) -> bool {
    match exp {
        ExpressionNode::Simple(s) => s.content.trim().is_empty(),
        ExpressionNode::Compound(c) => c.children.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::ValidVOn;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVOn));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_on_click() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div @click="handleClick"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_on_long_form() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-on:click="handleClick"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_on_modifier_only() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<form @submit.prevent></form>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_on_no_handler() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div @click></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
