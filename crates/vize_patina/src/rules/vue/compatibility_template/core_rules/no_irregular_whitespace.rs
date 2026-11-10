//! `vue/no-irregular-whitespace` compatibility rule.

use super::super::core_checks::{
    CoreKind, check_directive as check_core_directive, check_expression,
};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l1::markup::entity::EntityContext;
use vize_relief::{DirectiveNode, ElementNode, InterpolationNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-irregular-whitespace",
    description: "Disallow irregular whitespace in template expressions",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow irregular whitespace in template expressions.
pub struct NoIrregularWhitespace;

impl Rule for NoIrregularWhitespace {
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
            CoreKind::NoIrregularWhitespace,
            EntityContext::Text,
        );
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        check_core_directive(ctx, directive, CoreKind::NoIrregularWhitespace);
    }
}
