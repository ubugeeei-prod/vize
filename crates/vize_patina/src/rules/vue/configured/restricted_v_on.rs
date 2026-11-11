//! `vue/no-restricted-v-on`.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::RestrictedNames;
use vize_l0::{String, cstr};
use vize_relief::{ExpressionNode, PropNode, RootNode};

use super::template_walk::walk;
use super::{hits, listed, report_template};

static META: RuleMeta = RuleMeta {
    name: "vue/no-restricted-v-on",
    description: "disallow specific argument in `v-on`",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-restricted-v-on`.
#[derive(Default)]
pub struct NoRestrictedVOn {
    names: Vec<String>,
}

impl NoRestrictedVOn {
    /// Build the rule from `linter.ruleOptions["vue/no-restricted-v-on"]`.
    pub(crate) fn from_options(options: &RestrictedNames) -> Self {
        Self {
            names: listed(options),
        }
    }
}

impl Rule for NoRestrictedVOn {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        if self.names.is_empty() {
            return;
        }
        let mut findings = Vec::new();
        walk(&root.children, 1, &mut |element, _depth| {
            for prop in &element.props {
                let PropNode::Directive(directive) = prop else {
                    continue;
                };
                if directive.name != "on" {
                    continue;
                }
                let Some(ExpressionNode::Simple(argument)) = &directive.arg else {
                    continue;
                };
                if argument.is_static && hits(&self.names, argument.content) {
                    let name = argument.content;
                    findings.push((
                        cstr!("Using `@{name}` is not allowed on this element."),
                        directive.loc.span.start,
                        directive.loc.span.end,
                    ));
                }
            }
        });
        report_template(ctx, META.name, findings);
    }
}
