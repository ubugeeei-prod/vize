//! `vue/no-empty-pattern` compatibility rule.

use super::super::core_checks::{
    CoreKind, check_directive as check_core_directive, check_expression,
};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l1::markup::entity::EntityContext;
use vize_relief::{DirectiveNode, ElementNode, InterpolationNode, PropNode};

mod slot_scope;

static META: RuleMeta = RuleMeta {
    name: "vue/no-empty-pattern",
    description: "Disallow empty destructuring patterns in template expressions",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow empty destructuring patterns in template expressions.
pub struct NoEmptyPattern;

impl Rule for NoEmptyPattern {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn template_hooks(&self) -> crate::rule::RuleHooks {
        crate::rule::RuleHooks::DIRECTIVE.union(crate::rule::RuleHooks::INTERPOLATION)
    }

    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
        check_expression(
            ctx,
            &interpolation.content,
            CoreKind::NoEmptyPattern,
            EntityContext::Text,
        );
    }

    /// A Vue 2 `slot-scope` or `scope` attribute is a parameter list, which
    /// vue-eslint-parser reads as a directive; the template AST keeps it as a
    /// plain attribute.
    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        for prop in element.props.iter() {
            let PropNode::Attribute(attribute) = prop else {
                continue;
            };
            if !matches!(attribute.name, "slot-scope" | "scope") {
                continue;
            }
            let start = attribute.loc.span.start as usize;
            let Some(text) = ctx.source.get(start..attribute.loc.span.end as usize) else {
                continue;
            };
            for (from, to) in slot_scope::empty_patterns(text) {
                ctx.warn_at_with_help(
                    "Unexpected empty destructuring pattern",
                    ByteRange::new((start + from) as u32, (start + to) as u32),
                    "Remove the empty pattern or name the values it should bind",
                );
            }
        }
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        check_core_directive(ctx, directive, CoreKind::NoEmptyPattern);
    }
}
