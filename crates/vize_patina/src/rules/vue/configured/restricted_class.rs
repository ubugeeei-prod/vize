//! `vue/no-restricted-class`.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::RestrictedNames;
use vize_l0::{String, cstr};
use vize_relief::{PropNode, RootNode};

use super::template_walk::{quoted_span, walk};
use super::{hits, listed, report_template};

static META: RuleMeta = RuleMeta {
    name: "vue/no-restricted-class",
    description: "disallow specific classes in Vue components",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-restricted-class`.
#[derive(Default)]
pub struct NoRestrictedClass {
    names: Vec<String>,
}

impl NoRestrictedClass {
    /// Build the rule from `linter.ruleOptions["vue/no-restricted-class"]`.
    pub(crate) fn from_options(options: &RestrictedNames) -> Self {
        Self {
            names: listed(options),
        }
    }
}

impl Rule for NoRestrictedClass {
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
                if attribute.name != "class" {
                    continue;
                }
                let Some(value) = &attribute.value else {
                    continue;
                };
                for token in value.content.split_whitespace() {
                    if !hits(&self.names, token) {
                        continue;
                    }
                    let (start, end) =
                        quoted_span(ctx.source, value.loc.span.start, value.loc.span.end);
                    findings.push((cstr!("'{token}' class is not allowed."), start, end));
                }
            }
        });
        report_template(ctx, META.name, findings);
    }
}
