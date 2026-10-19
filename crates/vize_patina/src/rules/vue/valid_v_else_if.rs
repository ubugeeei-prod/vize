//! vue/valid-v-else-if
//!
//! Enforce valid `v-else-if` directives.
//!
//! `v-else-if` must:
//! - Be on an element directly following a `v-if` or `v-else-if` element
//! - Not share its element with `v-if` or `v-else`
//! - Have a condition, and no argument or modifier
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-else-if="foo"></div> <!-- without preceding v-if -->
//! <div v-if="foo"></div><div v-else-if></div>
//! <div v-if="foo"></div><div v-else-if:arg="bar"></div>
//! <div v-if="foo"></div><div v-else-if.mod="bar"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-if="foo"></div>
//! <div v-else-if="bar"></div>
//! ```

use super::if_chain::{OrphanBranch, for_each_orphan_branch};
use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, RootNode, TemplateChildNode,
};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-else-if",
    description: "Enforce valid `v-else-if` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-else-if directives
pub struct ValidVElseIf;

impl Rule for ValidVElseIf {
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
        if directive.name != "else-if" {
            return;
        }

        if has_directive(element, "if") {
            ctx.warn_with_help(
                ctx.t("vue/valid-v-else-if.with_v_if"),
                &directive.loc,
                ctx.t("vue/valid-v-else-if.help"),
            );
        }

        if has_directive(element, "else") {
            ctx.warn_with_help(
                ctx.t("vue/valid-v-else-if.with_v_else"),
                &directive.loc,
                ctx.t("vue/valid-v-else-if.help"),
            );
        }

        if let Some(arg) = &directive.arg {
            let loc = match arg {
                ExpressionNode::Simple(simple) => &simple.loc,
                ExpressionNode::Compound(compound) => &compound.loc,
            };
            ctx.warn_with_help(
                ctx.t("vue/valid-v-else-if.unexpected_argument"),
                loc,
                ctx.t("vue/valid-v-else-if.help"),
            );
        }

        if let (Some(first), Some(last)) = (directive.modifiers.first(), directive.modifiers.last())
        {
            ctx.report(
                LintDiagnostic::warn(
                    META.name,
                    ctx.t("vue/valid-v-else-if.unexpected_modifier").as_ref(),
                    first.loc.span.start,
                    last.loc.span.end,
                )
                .with_help(ctx.t("vue/valid-v-else-if.help").as_ref()),
            );
        }

        if !has_condition(&directive.exp) {
            ctx.warn_with_help(
                ctx.t("vue/valid-v-else-if.expected_value"),
                &directive.loc,
                ctx.t("vue/valid-v-else-if.help"),
            );
        }
    }
}

fn check_adjacent_if_chain(ctx: &mut LintContext, children: &[TemplateChildNode]) {
    for_each_orphan_branch(children, |branch, loc| {
        if matches!(branch, OrphanBranch::ElseIf) {
            ctx.warn_with_help(
                ctx.t("vue/valid-v-else-if.missing_v_if"),
                loc,
                ctx.t("vue/valid-v-else-if.help"),
            );
        }
    });
}

fn has_directive(element: &ElementNode, name: &str) -> bool {
    element
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == name))
}

fn has_condition(exp: &Option<ExpressionNode>) -> bool {
    match exp {
        Some(ExpressionNode::Simple(simple)) => !simple.content.trim().is_empty(),
        Some(ExpressionNode::Compound(compound)) => !compound.children.is_empty(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    //! Cases mirror eslint-plugin-vue's `valid-v-else-if` suite. Templates are
    //! written at the `lint_template` call so the markup differential battery
    //! picks them up as fixtures.

    use super::ValidVElseIf;
    use crate::linter::{LintResult, Linter};
    use crate::rule::RuleRegistry;

    fn linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVElseIf));
        Linter::with_registry(registry)
    }

    fn messages(result: &LintResult) -> Vec<&str> {
        result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                assert_eq!(diagnostic.rule_name, "vue/valid-v-else-if");
                diagnostic.message.as_str()
            })
            .collect()
    }

    fn reported<'a>(source: &'a str, result: &LintResult) -> Vec<&'a str> {
        result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                source
                    .get(diagnostic.start as usize..diagnostic.end as usize)
                    .unwrap_or_default()
            })
            .collect()
    }

    const MISSING_V_IF: &str = "'v-else-if' directives require being preceded by the element which has a 'v-if' or 'v-else-if' directive.";
    const NO_ARGUMENT: &str = "'v-else-if' directives require no argument.";
    const NO_MODIFIER: &str = "'v-else-if' directives require no modifier.";
    const EXPECTED_VALUE: &str = "'v-else-if' directives require that attribute value.";

    #[test]
    fn accepts_chain_after_v_if() {
        let result = linter().lint_template(
            r#"<div><div v-if="foo"></div><div v-else-if="foo"></div></div>"#,
            "test.vue",
        );
        assert!(messages(&result).is_empty());
    }

    #[test]
    fn accepts_repeated_branches() {
        let result = linter().lint_template(
        r#"<div><div v-if="foo"></div><div v-else-if="foo"></div><div v-else-if="bar"></div></div>"#,
        "test.vue",
    );
        assert!(messages(&result).is_empty());
    }

    #[test]
    fn accepts_comments_and_whitespace_between_branches() {
        let result = linter().lint_template(
            "<div v-if=\"a\"></div>\n  <!-- next -->\n  <div v-else-if=\"b\"></div>",
            "test.vue",
        );
        assert!(messages(&result).is_empty());
    }

    #[test]
    fn accepts_template_branches() {
        let result = linter().lint_template(
            r#"<template v-if="a">a</template><template v-else-if="b">b</template>"#,
            "test.vue",
        );
        assert!(messages(&result).is_empty());
    }

    #[test]
    fn reports_orphan_at_root() {
        let result = linter().lint_template(r#"<div v-else-if="foo"></div>"#, "test.vue");
        assert_eq!(messages(&result), [MISSING_V_IF]);
    }

    #[test]
    fn reports_orphan_in_nested_children() {
        let result =
            linter().lint_template(r#"<div><div v-else-if="foo"></div></div>"#, "test.vue");
        assert_eq!(messages(&result), [MISSING_V_IF]);
    }

    #[test]
    fn reports_branch_after_plain_element() {
        let result = linter().lint_template(
            r#"<div><div></div><div v-else-if="foo"></div></div>"#,
            "test.vue",
        );
        assert_eq!(messages(&result), [MISSING_V_IF]);
    }

    #[test]
    fn reports_branch_after_v_else() {
        let result = linter().lint_template(
            r#"<div><div v-if="foo"></div><div v-else></div><div v-else-if="foo"></div></div>"#,
            "test.vue",
        );
        assert_eq!(messages(&result), [MISSING_V_IF]);
    }

    /// The Vue compiler rejects text between branches; eslint-plugin-vue only
    /// looks at the previous element and stays silent here.
    #[test]
    fn reports_branch_separated_by_text() {
        let result = linter().lint_template(
            r#"<div v-if="foo"></div> text <div v-else-if="bar"></div>"#,
            "test.vue",
        );
        assert_eq!(messages(&result), [MISSING_V_IF]);
    }

    #[test]
    fn reports_only_the_first_orphan_of_a_run() {
        let result = linter().lint_template(
            r#"<div v-else-if="foo"></div><div v-else-if="bar"></div>"#,
            "test.vue",
        );
        assert_eq!(messages(&result), [MISSING_V_IF]);
    }

    #[test]
    fn reports_v_if_on_the_same_element() {
        let result = linter().lint_template(
            r#"<div><div v-if="foo"></div><div v-if="foo" v-else-if="bar"></div></div>"#,
            "test.vue",
        );
        assert_eq!(
            messages(&result),
            ["'v-else-if' and 'v-if' directives can't exist on the same element."]
        );
    }

    #[test]
    fn reports_v_else_on_the_same_element() {
        let result = linter().lint_template(
            r#"<div><div v-if="foo"></div><div v-else v-else-if="foo"></div></div>"#,
            "test.vue",
        );
        assert_eq!(
            messages(&result),
            ["'v-else-if' and 'v-else' directives can't exist on the same element."]
        );
    }

    #[test]
    fn reports_argument_at_the_argument() {
        let source = r#"<div v-if="foo"></div><div v-else-if:aaa="foo"></div>"#;
        let result = linter().lint_template(
            r#"<div v-if="foo"></div><div v-else-if:aaa="foo"></div>"#,
            "test.vue",
        );
        assert_eq!(messages(&result), [NO_ARGUMENT]);
        assert_eq!(reported(source, &result), ["aaa"]);
    }

    #[test]
    fn reports_all_modifiers_once() {
        let source = r#"<div v-if="foo"></div><div v-else-if.aaa.bbb="foo"></div>"#;
        let result = linter().lint_template(
            r#"<div v-if="foo"></div><div v-else-if.aaa.bbb="foo"></div>"#,
            "test.vue",
        );
        assert_eq!(messages(&result), [NO_MODIFIER]);
        assert_eq!(reported(source, &result), ["aaa.bbb"]);
    }

    #[test]
    fn reports_missing_value() {
        let result =
            linter().lint_template(r#"<div v-if="foo"></div><div v-else-if></div>"#, "test.vue");
        assert_eq!(messages(&result), [EXPECTED_VALUE]);
    }

    #[test]
    fn reports_empty_value() {
        let result = linter().lint_template(
            r#"<div v-if="foo"></div><div v-else-if=""></div>"#,
            "test.vue",
        );
        assert_eq!(messages(&result), [EXPECTED_VALUE]);
    }

    #[test]
    fn leaves_orphan_v_else_to_valid_v_else() {
        let result = linter().lint_template(r#"<div v-else></div>"#, "test.vue");
        assert!(messages(&result).is_empty());
    }
}
