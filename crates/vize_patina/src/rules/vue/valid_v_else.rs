//! vue/valid-v-else
//!
//! Enforce valid `v-else` directives.
//!
//! `v-else` must:
//! - Be on an element immediately following a `v-if` or `v-else-if` element
//! - Not have an expression
//! - Not be used with `v-if` or `v-else-if` on the same element
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-else="foo"></div>
//! <div v-else v-if="bar"></div>
//! <div v-else></div> <!-- without preceding v-if -->
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-if="foo"></div>
//! <div v-else></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{Fix, Severity, TextEdit};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{DirectiveNode, ElementNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-else",
    description: "Enforce valid `v-else` directives",
    category: RuleCategory::Essential,
    fixable: true,
    default_severity: Severity::Error,
};

/// Enforce valid v-else directives
pub struct ValidVElse;

impl Rule for ValidVElse {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "else" {
            return;
        }

        // Check 1: v-else should not have an expression
        if directive.exp.is_some() {
            let fix = Fix::new(
                "Remove the expression from v-else",
                TextEdit::delete(directive.loc.start.offset, directive.loc.end.offset),
            );
            ctx.report(
                crate::diagnostic::LintDiagnostic::error(
                    META.name,
                    ctx.t("vue/valid-v-else.unexpected_expression").as_ref(),
                    directive.loc.start.offset,
                    directive.loc.end.offset,
                )
                .with_help(ctx.t("vue/valid-v-else.help").as_ref())
                .with_fix(fix),
            );
        }

        // Check 2: v-else should not be used with v-if or v-else-if
        let has_v_if = element.props.iter().any(|p| {
            matches!(p, PropNode::Directive(d) if d.name.as_str() == "if" || d.name.as_str() == "else-if")
        });
        if has_v_if {
            ctx.error_with_help(
                ctx.t("vue/valid-v-else.missing_v_if"),
                &directive.loc,
                ctx.t("vue/valid-v-else.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ValidVElse;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVElse));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_else() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div v-if="foo"></div><div v-else></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_else_with_expression() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="foo"></div><div v-else="bar"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_v_else_with_v_if() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-if="foo" v-else></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
