//! `vue/no-restricted-static-attribute`.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::RestrictedNames;
use vize_l0::{String, cstr};
use vize_relief::{PropNode, RootNode};

use super::template_walk::walk;
use super::{hits, listed, report_template};

static META: RuleMeta = RuleMeta {
    name: "vue/no-restricted-static-attribute",
    description: "disallow specific attribute",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-restricted-static-attribute`.
#[derive(Default)]
pub struct NoRestrictedStaticAttribute {
    names: Vec<String>,
}

impl NoRestrictedStaticAttribute {
    /// Build the rule from `linter.ruleOptions["vue/no-restricted-static-attribute"]`.
    pub(crate) fn from_options(options: &RestrictedNames) -> Self {
        Self {
            names: listed(options),
        }
    }
}

impl Rule for NoRestrictedStaticAttribute {
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
                let PropNode::Attribute(attribute) = prop else {
                    continue;
                };
                if !hits(&self.names, attribute.name) {
                    continue;
                }
                let span = if attribute.name_loc.span.end > attribute.name_loc.span.start {
                    attribute.name_loc.span
                } else {
                    attribute.loc.span
                };
                let name = attribute.name;
                findings.push((
                    cstr!("Using `{name}` is not allowed."),
                    span.start,
                    span.end,
                ));
            }
        });
        report_template(ctx, META.name, findings);
    }
}
