//! `vue/prefer-const`
//!
//! Report `let` and `var` bindings that neither the script nor the template
//! reassigns. A template update such as `counter++` is a write, so the
//! binding stays `let`. Parameters and `const` stay quiet.

use vize_relief::RootNode;

use super::decide::Which;
use super::options::{PreferConstOptions, UnusedOptions};
use super::run::report;
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "vue/prefer-const",
    description: "Require const for bindings the script and template never reassign",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `let` and `var` that the script and the template never reassign.
#[derive(Default)]
pub struct PreferConst {
    options: PreferConstOptions,
}

impl Rule for PreferConst {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        if ctx.sfc_descriptor().is_none() {
            return;
        }
        report(
            ctx,
            Some(root),
            Which::PreferConst,
            &UnusedOptions::default(),
            &self.options,
            META.name,
        );
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let Some(descriptor) = ctx.sfc_descriptor() else {
            return;
        };
        if descriptor.template.is_some() {
            return;
        }
        report(
            ctx,
            None,
            Which::PreferConst,
            &UnusedOptions::default(),
            &self.options,
            META.name,
        );
    }
}
