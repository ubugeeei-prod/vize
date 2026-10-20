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

use super::if_chain::{OrphanBranch, for_each_orphan_branch};
use crate::context::LintContext;
use crate::diagnostic::{Fix, Severity, TextEdit};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, PropNode, RootNode, TemplateChildNode};

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

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        check_adjacent_if_chain(ctx, &root.children);
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        check_adjacent_if_chain(ctx, &element.children);
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name != "else" {
            return;
        }

        // Check 1: v-else should not have an expression
        if directive.exp.is_some() {
            let mut diagnostic = crate::diagnostic::LintDiagnostic::error(
                META.name,
                ctx.t("vue/valid-v-else.unexpected_expression").as_ref(),
                directive.loc.span.start,
                directive.loc.span.end,
            )
            .with_help(ctx.t("vue/valid-v-else.help").as_ref());
            if let Some(fix) = unexpected_expression_fix(ctx.source, directive) {
                diagnostic = diagnostic.with_fix(fix);
            }
            ctx.report(diagnostic);
        }

        // Check 2: v-else should not be used with v-if or v-else-if
        let has_v_if = element
            .props
            .iter()
            .any(|p| matches!(p, PropNode::Directive(d) if d.name == "if" || d.name == "else-if"));
        if has_v_if {
            ctx.error_with_help(
                ctx.t("vue/valid-v-else.missing_v_if"),
                &directive.loc,
                ctx.t("vue/valid-v-else.help"),
            );
        }
    }
}

fn unexpected_expression_fix(source: &str, directive: &DirectiveNode) -> Option<Fix> {
    let raw_name = directive.raw_name?;
    let authored =
        source.get(directive.loc.span.start as usize..directive.loc.span.end as usize)?;
    authored.starts_with(raw_name).then(|| {
        Fix::new(
            "Remove the expression from v-else",
            TextEdit::delete(
                directive.loc.span.start + raw_name.len() as u32,
                directive.loc.span.end,
            ),
        )
    })
}

/// Report every `v-else` that does not directly follow a conditional branch.
///
/// An orphaned `v-else-if` belongs to `vue/valid-v-else-if`, as in
/// eslint-plugin-vue.
fn check_adjacent_if_chain(ctx: &mut LintContext, children: &[TemplateChildNode]) {
    for_each_orphan_branch(children, |branch, loc| {
        if matches!(branch, OrphanBranch::Else) {
            ctx.error_with_help(
                ctx.t("vue/valid-v-else.missing_v_if"),
                loc,
                ctx.t("vue/valid-v-else.help"),
            );
        }
    });
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
    fn test_valid_v_else_if_chain() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="foo"></div><div v-else-if="bar"></div><div v-else></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_else_after_comment() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="foo"></div><!-- fallback branch --><div v-else></div>"#,
            "test.vue",
        );
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
        assert_eq!(
            result.diagnostics[0]
                .fix
                .as_ref()
                .unwrap()
                .apply(r#"<div v-if="foo"></div><div v-else="bar"></div>"#),
            r#"<div v-if="foo"></div><div v-else></div>"#
        );
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_v_else_with_v_if() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-if="foo" v-else></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_else_without_adjacent_v_if() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-else></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
        assert_eq!(result.diagnostics[0].rule_name, "vue/valid-v-else");
    }

    #[test]
    fn test_orphan_v_else_if_belongs_to_valid_v_else_if() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-else-if="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_v_else_after_orphan_v_else_if_is_not_reported_twice() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-else-if="foo"></div><div v-else></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_else_after_text_gap() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="foo"></div> text <div v-else></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }
}
