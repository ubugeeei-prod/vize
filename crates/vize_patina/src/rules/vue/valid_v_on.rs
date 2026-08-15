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
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-on",
    description: "Enforce valid `v-on` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-on directives
pub struct ValidVOn;

const JAVASCRIPT_KEYWORDS: &[&str] = &[
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "finally",
    "for",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "new",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "static",
    "super",
    "switch",
    "throw",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

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
        if directive.name != "on" {
            return;
        }

        let has_arg = directive.arg.is_some();
        let has_exp = directive
            .exp
            .as_ref()
            .map(|e| !is_empty_expression(e))
            .unwrap_or(false);

        if directive.exp.as_ref().is_some_and(is_keyword_expression) {
            ctx.error_with_help(
                ctx.t("vue/valid-v-on.invalid_handler"),
                &directive.loc,
                ctx.t("vue/valid-v-on.help"),
            );
            return;
        }

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

fn is_keyword_expression(exp: &ExpressionNode) -> bool {
    let ExpressionNode::Simple(simple) = exp else {
        return false;
    };

    let candidate = simple.content.trim();
    JAVASCRIPT_KEYWORDS.contains(&candidate)
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

    #[test]
    fn test_invalid_v_on_keyword_handler() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div @click="for"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_on_keyword_object_syntax() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-on="class"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_v_on_literal_handler_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div @click="true"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }
}
