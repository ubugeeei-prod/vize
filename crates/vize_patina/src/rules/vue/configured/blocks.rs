//! `vue/no-restricted-block`.
//!
//! Reports a custom block whose type is one of the configured names. The
//! opening tag is the report span.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::config::RestrictedNames;
use vize_l0::{String, cstr};

use super::{hits, listed};

static META: RuleMeta = RuleMeta {
    name: "vue/no-restricted-block",
    description: "disallow specific block",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-restricted-block`. An empty list checks nothing.
#[derive(Default)]
pub struct NoRestrictedBlock {
    names: Vec<String>,
}

impl NoRestrictedBlock {
    /// Build the rule from `linter.ruleOptions["vue/no-restricted-block"]`.
    pub(crate) fn from_options(options: &RestrictedNames) -> Self {
        Self {
            names: listed(options),
        }
    }
}

impl Rule for NoRestrictedBlock {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        if self.names.is_empty() {
            return;
        }
        let Some(descriptor) = ctx.sfc_descriptor() else {
            return;
        };
        let mut findings = Vec::new();
        for block in &descriptor.custom_blocks {
            if !hits(&self.names, block.block_type.as_ref()) {
                continue;
            }
            let start = u32::try_from(block.loc.tag_start).unwrap_or(0);
            let end = u32::try_from(block.loc.start).unwrap_or(start);
            let name = block.block_type.as_ref();
            findings.push((
                cstr!("Using `<{name}>` is not allowed."),
                start,
                end.max(start.saturating_add(1)),
            ));
        }
        super::report(ctx, META.name, findings);
    }
}
