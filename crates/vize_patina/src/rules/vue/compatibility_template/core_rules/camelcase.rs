//! `vue/camelcase` compatibility rule.

use super::super::core_checks::{CoreKind, check_expression};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l1::markup::entity::EntityContext;
use vize_relief::{DirectiveNode, ElementNode, InterpolationNode};

static META: RuleMeta = RuleMeta {
    name: "vue/camelcase",
    description: "Require camelCase identifiers in template expressions",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require camelCase identifiers in template expressions.
pub struct Camelcase;

impl Rule for Camelcase {
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
            CoreKind::Camelcase,
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
                CoreKind::Camelcase,
                EntityContext::Attribute,
            );
        }
    }
}
