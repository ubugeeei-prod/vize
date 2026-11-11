//! `vue/no-restricted-html-elements`.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::RestrictedNames;
use vize_l0::{String, cstr};
use vize_relief::{ElementType, RootNode};

use super::template_walk::walk;
use super::{hits, listed, report_template};

static META: RuleMeta = RuleMeta {
    name: "vue/no-restricted-html-elements",
    description: "disallow specific elements",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-restricted-html-elements`.
#[derive(Default)]
pub struct NoRestrictedHtmlElements {
    names: Vec<String>,
}

impl NoRestrictedHtmlElements {
    /// Build the rule from `linter.ruleOptions["vue/no-restricted-html-elements"]`.
    pub(crate) fn from_options(options: &RestrictedNames) -> Self {
        Self {
            names: listed(options),
        }
    }
}

impl Rule for NoRestrictedHtmlElements {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        if self.names.is_empty() {
            return;
        }
        let mut findings = Vec::new();
        walk(&root.children, 1, &mut |element, _depth| {
            if element.tag_type == ElementType::Element && hits(&self.names, element.tag) {
                findings.push((
                    cstr!("Unexpected use of forbidden element {}.", element.tag),
                    element.loc.span.start,
                    element.loc.span.end,
                ));
            }
        });
        report_template(ctx, META.name, findings);
    }
}
