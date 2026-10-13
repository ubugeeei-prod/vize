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
mod tests;
