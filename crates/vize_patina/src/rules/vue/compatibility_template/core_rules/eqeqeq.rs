//! `vue/eqeqeq` compatibility rule.

use super::super::core_checks::{
    CoreKind, check_directive as check_core_directive, check_expression, check_sfc_styles,
};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l1::markup::entity::EntityContext;
use vize_relief::{DirectiveNode, ElementNode, InterpolationNode};

static META: RuleMeta = RuleMeta {
    name: "vue/eqeqeq",
    description: "Require strict equality in template expressions",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require strict equality in template expressions.
pub struct Eqeqeq;

impl Rule for Eqeqeq {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        check_sfc_styles(ctx, CoreKind::Eqeqeq);
    }

    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
        check_expression(
            ctx,
            &interpolation.content,
            CoreKind::Eqeqeq,
            EntityContext::Text,
        );
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        check_core_directive(ctx, directive, CoreKind::Eqeqeq);
    }
}
