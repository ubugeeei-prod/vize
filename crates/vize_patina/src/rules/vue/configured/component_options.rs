//! `vue/no-restricted-component-options`.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::RestrictedNames;
use vize_l0::{String, cstr};

use super::component_scan::component_options;
use super::{each_program, hits, listed, report};

static META: RuleMeta = RuleMeta {
    name: "vue/no-restricted-component-options",
    description: "disallow specific component option",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-restricted-component-options`.
#[derive(Default)]
pub struct NoRestrictedComponentOptions {
    names: Vec<String>,
}

impl NoRestrictedComponentOptions {
    /// Build the rule from `linter.ruleOptions["vue/no-restricted-component-options"]`.
    pub(crate) fn from_options(options: &RestrictedNames) -> Self {
        Self {
            names: listed(options),
        }
    }
}

impl Rule for NoRestrictedComponentOptions {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        if self.names.is_empty() {
            return;
        }
        let mut findings = Vec::new();
        each_program(ctx, |program, offset| {
            for (name, span) in component_options(program) {
                if hits(&self.names, &name) {
                    findings.push((
                        cstr!("Using `{name}` is not allowed."),
                        offset + span.start,
                        offset + span.end,
                    ));
                }
            }
        });
        report(ctx, META.name, findings);
    }
}
