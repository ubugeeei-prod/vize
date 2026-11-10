//! Restricted `v-bind` arguments.

use super::shared::static_argument;
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-restricted-v-bind",
    description: "Disallow restricted v-bind arguments",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-restricted-v-bind`; the default rejects `:v-*` attributes.
pub struct NoRestrictedVBind;

impl Rule for NoRestrictedVBind {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name == "bind"
            && static_argument(directive).is_some_and(|argument| argument.starts_with("v-"))
        {
            ctx.warn_with_help(
                "Using `:v-*` is not allowed",
                &directive.loc,
                "Remove `:` and use the attribute as a directive",
            );
        }
    }
}
