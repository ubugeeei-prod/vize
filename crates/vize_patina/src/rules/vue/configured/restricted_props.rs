//! `vue/no-restricted-props`.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::RestrictedNames;
use vize_l0::{String, cstr};

use super::props_scan::groups;
use super::{each_program, hits, listed, report};

static META: RuleMeta = RuleMeta {
    name: "vue/no-restricted-props",
    description: "disallow specific props",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-restricted-props`. An empty list checks nothing.
#[derive(Default)]
pub struct NoRestrictedProps {
    names: Vec<String>,
}

impl NoRestrictedProps {
    /// Build the rule from `linter.ruleOptions["vue/no-restricted-props"]`.
    pub(crate) fn from_options(options: &RestrictedNames) -> Self {
        Self {
            names: listed(options),
        }
    }
}

impl Rule for NoRestrictedProps {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        if self.names.is_empty() {
            return;
        }
        let mut findings = Vec::new();
        each_program(ctx, |program, offset| {
            for group in groups(program) {
                for (name, span) in group.props {
                    if hits(&self.names, &name) {
                        findings.push((
                            cstr!("Using `{name}` props is not allowed."),
                            offset + span.start,
                            offset + span.end,
                        ));
                    }
                }
            }
        });
        report(ctx, META.name, findings);
    }
}
