//! Matching binds and update listeners that can use `v-model`.

use super::shared::{mirrors_assignment, simple_expression, static_argument};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::{cstr, is_native_tag};
use vize_relief::{ElementNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/prefer-v-model",
    description: "Prefer v-model over matching bind and update listeners",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/prefer-v-model`.
pub struct PreferVModel;

impl Rule for PreferVModel {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if is_native_tag(element.tag) {
            return;
        }
        for bind in element.props.iter().filter_map(|prop| match prop {
            PropNode::Directive(directive)
                if directive.name == "bind" && directive.modifiers.is_empty() =>
            {
                Some(directive)
            }
            _ => None,
        }) {
            let Some(prop_name) = static_argument(bind) else {
                continue;
            };
            let Some(bind_expression) = simple_expression(bind.exp.as_ref()) else {
                continue;
            };
            let update_name = cstr!("update:{}", prop_name);
            let matching = element.props.iter().find_map(|prop| match prop {
                PropNode::Directive(directive)
                    if directive.name == "on"
                        && directive.modifiers.is_empty()
                        && static_argument(directive) == Some(update_name.as_str()) =>
                {
                    Some(directive)
                }
                _ => None,
            });
            let Some(handler) =
                matching.and_then(|directive| simple_expression(directive.exp.as_ref()))
            else {
                continue;
            };
            if mirrors_assignment(bind_expression, handler) {
                ctx.warn_with_help(
                    cstr!(
                        "Prefer `v-model` over `:{}` and `@update:{}`",
                        prop_name,
                        prop_name
                    ),
                    &bind.loc,
                    "Replace the matching bind and update listener with v-model",
                );
            }
        }
    }
}
