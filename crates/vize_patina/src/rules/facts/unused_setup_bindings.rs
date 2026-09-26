//! `vue/no-unused-setup-bindings` (P4-3c), opt-in, tier Sound.
//!
//! Report valid script-setup declarations with no resolved script, template
//! or style `v-bind()` read. Underscore prefixes express intentional non-use.
//! Croquis stores the authoritative relation, and this consumer only reads
//! its declared group. Unknown/invalid or external blocks are outside the domain.

use vize_croquis::facts::{Demand, FactConsumer, FactGroup, UnusedBindings};
use vize_relief::RootNode;

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "vue/no-unused-setup-bindings",
    description: "Disallow unread script setup bindings",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Opt-in rule over the UnusedBindings group.
#[derive(Default)]
pub struct NoUnusedSetupBindings;

impl FactConsumer for NoUnusedSetupBindings {
    const NAME: &'static str = "vue/no-unused-setup-bindings";
    const DEMAND: Demand = Demand::NONE.with(UnusedBindings::ID);
}

impl NoUnusedSetupBindings {
    fn report(ctx: &mut LintContext<'_>) {
        let Some(view) = ctx.facts::<Self>() else {
            return;
        };
        let Ok(table) = view.get::<UnusedBindings>() else {
            return;
        };
        let diagnostics = diagnostics(table);
        for diagnostic in diagnostics {
            ctx.report_in_script(diagnostic);
        }
    }
}

impl Rule for NoUnusedSetupBindings {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _: &RootNode<'a>) {
        if ctx.sfc_descriptor().is_some() {
            Self::report(ctx);
        }
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        if !ctx.is_rule_enabled(META.name) {
            return;
        }
        let Some(descriptor) = ctx.sfc_descriptor() else {
            return;
        };
        if descriptor.template.is_some() {
            return;
        }
        // A script-only SFC never reaches the template analysis path. Draw
        // exactly once for that artifact using the shared descriptor.
        let analysis = vize_atelier_sfc::croquis::analyze_sfc_descriptor(
            descriptor,
            None,
            vize_atelier_sfc::croquis::SfcCroquisOptions::lint_demand().with_unused_bindings(),
        );
        let mut facts = vize_croquis::facts::CroquisFacts::new(&analysis);
        let Ok(table) = facts.prepare::<Self>().get::<UnusedBindings>() else {
            return;
        };
        let diagnostics = diagnostics(table);
        for diagnostic in diagnostics {
            ctx.report_in_script(diagnostic);
        }
    }
}

fn diagnostics(table: &vize_croquis::facts::FactTable<UnusedBindings>) -> Vec<LintDiagnostic> {
    table
        .iter()
        .filter(|(name, _)| !name.starts_with('_'))
        .map(|(name, fact)| {
            LintDiagnostic::warn(
                META.name,
                vize_l0::cstr!("Setup binding '{name}' is never read"),
                fact.span.0,
                fact.span.1,
            )
            .with_help(
                "Remove the binding or prefix its name with underscore if intentionally unused",
            )
        })
        .collect()
}

#[cfg(test)]
mod tests;
