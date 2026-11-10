//! `vue/no-loss-of-precision` compatibility rule.

use super::super::core_checks::{CoreKind, check_expression};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l1::markup::entity::EntityContext;
use vize_relief::{DirectiveNode, ElementNode, InterpolationNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-loss-of-precision",
    description: "Disallow numeric literals that lose precision",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow numeric literals that lose precision.
pub struct NoLossOfPrecision;

impl Rule for NoLossOfPrecision {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
        check_expression(
            ctx,
            &interpolation.content,
            CoreKind::NoLossOfPrecision,
            EntityContext::Text,
        );
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if let Some(expression) = &directive.exp {
            check_expression(
                ctx,
                expression,
                CoreKind::NoLossOfPrecision,
                EntityContext::Attribute,
            );
        }
    }
}
