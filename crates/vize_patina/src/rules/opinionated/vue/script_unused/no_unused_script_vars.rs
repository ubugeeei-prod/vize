//! `vue/no-unused-script-vars`
//!
//! Report unused script variables, imports, type imports, and parameters.
//! A template read counts, including a name nested code cannot see: a
//! component tag, a custom directive (`v-focus` reads `vFocus`), and
//! `ref="panel"`. A `<style>` `v-bind()` and the `<script setup generic>`
//! attribute count too. Nested locals stay local.
//!
//! Reads follow typescript-eslint `no-unused-vars`: `a += 1`, `x = f(x)`, and
//! a read inside the binding's own declaration are not uses. Options take
//! the typescript-eslint keys. Variables and parameters matching `/^_/u` stay
//! quiet by default; caught errors have no default pattern.

use vize_relief::RootNode;

use super::decide::Which;
use super::options::UnusedOptions;
use super::run::report;
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "vue/no-unused-script-vars",
    description: "Disallow unused script variables, imports, and parameters",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Unused script bindings, with template reads counted.
#[derive(Default)]
pub struct NoUnusedScriptVars {
    options: UnusedOptions,
}

impl NoUnusedScriptVars {
    /// Rule with `linter.ruleOptions["vue/no-unused-script-vars"]` applied.
    pub fn from_options(options: &vize_l0::config::NoUnusedScriptVarsOptions) -> Self {
        Self {
            options: UnusedOptions::from_config(options),
        }
    }
}

impl Rule for NoUnusedScriptVars {
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
            Which::Unused,
            &self.options,
            &super::options::PreferConstOptions::default(),
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
            Which::Unused,
            &self.options,
            &super::options::PreferConstOptions::default(),
            META.name,
        );
    }
}
