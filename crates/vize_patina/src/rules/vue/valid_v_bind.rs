//! vue/valid-v-bind
//!
//! Enforce valid `v-bind` directives.
//!
//! `v-bind` must:
//! - Have an attribute name (argument) or be used for object binding
//! - Have an expression (or use Vue 3.4+ same-name shorthand)
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-bind></div>
//! <div :></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div :class="foo"></div>
//! <div v-bind:class="foo"></div>
//! <div v-bind="{ class: foo }"></div>
//! <div :loading></div>  <!-- Vue 3.4+ same-name shorthand for :loading="loading" -->
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-bind",
    description: "Enforce valid `v-bind` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-bind directives
pub struct ValidVBind;

const VALID_MODIFIERS: &[&str] = &["attr", "camel", "prop", "sync"];

impl Rule for ValidVBind {
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

        for modifier in directive.modifiers.iter() {
            if !VALID_MODIFIERS.contains(&modifier.content.as_str()) {
                ctx.error_with_help(
                    ctx.t("vue/valid-v-bind.unsupported_modifier"),
                    &modifier.loc,
                    ctx.t("vue/valid-v-bind.help"),
                );
            }
        }

        let has_arg = directive.arg.is_some();
        let has_exp = directive
            .exp
            .as_ref()
            .map(|e| !is_empty_expression(e))
            .unwrap_or(false);

        // Object syntax: v-bind="{ class: foo }"
        if !has_arg && has_exp {
            // This is valid object syntax
            return;
        }

        // Attribute syntax: :class="foo" or Vue 3.4+ same-name shorthand: :loading
        if let Some(arg) = &directive.arg {
            // Vue 3.4+ same-name shorthand allows static :attr without expression.
            // Dynamic arguments still need an explicit expression.
            if !has_exp && !is_static_argument(arg) {
                ctx.error_with_help(
                    ctx.t("vue/valid-v-bind.missing_expression"),
                    &directive.loc,
                    ctx.t("vue/valid-v-bind.help"),
                );
            }
            return;
        }

        // No argument and no expression
        ctx.error_with_help(
            ctx.t("vue/valid-v-bind.missing_expression"),
            &directive.loc,
            ctx.t("vue/valid-v-bind.help"),
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

fn is_static_argument(arg: &ExpressionNode) -> bool {
    matches!(arg, ExpressionNode::Simple(simple) if simple.is_static)
}

#[cfg(test)]
mod tests {
    use super::ValidVBind;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVBind));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_bind() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_bind_long_form() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-bind:class="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_bind_same_name_shorthand() {
        // Vue 3.4+ same-name shorthand: :loading is equivalent to :loading="loading"
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :loading></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_bind_same_name_shorthand_multiple() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :loading :disabled :checked></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_bind_dynamic_argument_with_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :[name]="value"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_bind_supported_modifiers() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :id.camel="id" :value.prop="value" :aria-label.attr="label" :title.sync="title"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_bind_no_arg_no_exp() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-bind></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_bind_dynamic_shorthand() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :[name]></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_bind_unsupported_modifier() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :foo.bar="value"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
