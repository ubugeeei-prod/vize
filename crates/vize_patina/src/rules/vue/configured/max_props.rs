//! `vue/max-props`.
//!
//! Prop names come from a `defineProps` type literal when one is present, and
//! otherwise from the runtime object or array. The same component is not
//! counted twice.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::MaxPropsOptions;
use vize_l0::cstr;

use super::props_scan::{groups, unique_count};
use super::{each_program, report};

static META: RuleMeta = RuleMeta {
    name: "vue/max-props",
    description: "enforce maximum number of props in Vue component",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/max-props`. An absent limit checks nothing.
#[derive(Default)]
pub struct MaxProps {
    limit: Option<u32>,
}

impl MaxProps {
    /// Build the rule from `linter.ruleOptions["vue/max-props"]`.
    pub(crate) fn from_options(options: &MaxPropsOptions) -> Self {
        Self {
            limit: options.max_props.filter(|value| *value >= 1),
        }
    }
}

impl Rule for MaxProps {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let Some(limit) = self.limit else {
            return;
        };
        let mut findings = Vec::new();
        each_program(ctx, |program, offset| {
            for group in groups(program) {
                let count = unique_count(&group.props);
                if count > limit as usize {
                    findings.push((
                        cstr!(
                            "Component has too many props ({count}). Maximum allowed is {limit}."
                        ),
                        offset + group.anchor.start,
                        offset + group.anchor.end,
                    ));
                }
            }
        });
        report(ctx, META.name, findings);
    }
}
