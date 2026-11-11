//! `vue/max-template-depth`.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::MaxTemplateDepthOptions;
use vize_l0::cstr;
use vize_relief::RootNode;

use super::report_template;
use super::template_walk::walk;

static META: RuleMeta = RuleMeta {
    name: "vue/max-template-depth",
    description: "enforce maximum depth of template",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/max-template-depth`.
#[derive(Default)]
pub struct MaxTemplateDepth {
    limit: Option<u32>,
}

impl MaxTemplateDepth {
    /// Build the rule from `linter.ruleOptions["vue/max-template-depth"]`.
    pub(crate) fn from_options(options: &MaxTemplateDepthOptions) -> Self {
        Self {
            limit: options.max_depth.filter(|value| *value >= 1),
        }
    }
}

impl Rule for MaxTemplateDepth {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        let Some(limit) = self.limit else {
            return;
        };
        let mut findings = Vec::new();
        walk(&root.children, 1, &mut |element, depth| {
            if depth > limit {
                findings.push((
                    cstr!(
                        "Element is nested too deeply (depth of {depth}, maximum allowed is {limit})."
                    ),
                    element.loc.span.start,
                    element.loc.span.end,
                ));
            }
        });
        report_template(ctx, META.name, findings);
    }
}
