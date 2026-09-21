//! vue/max-template-complexity
//!
//! Warn when a component's **own** template complexity exceeds the
//! corpus-pinned thresholds (Davinci P4-9b): cyclomatic complexity above 11
//! or cognitive complexity above 16, the full-corpus p95 recorded in
//! `davinci-road/plan/complexity-metrics.md`.
//!
//! The facts come from the S2 `template-complexity` fact group
//! (`vize_s1_to_s2::pass::cfg`, read under this rule's declared demand)
//! over the lowered template — `v-if` /
//! `v-else-if` / `v-else` branches, `v-for` loops, scoped-slot nesting and
//! the logical and conditional operators of every evaluated expression —
//! never from text scanning. **Tier `exact`**: the rule reports a property
//! of the facts, and the facts are proven against an independent evaluator
//! (TS-34), so there is no heuristic step between the template and the
//! verdict.
//!
//! Only **own** complexity is judged: a child component never taxes its
//! parent, so extracting a branch into a component always lowers the
//! parent's score. The cross-file (rendered) number is Doctor's.
//!
//! The diagnostic points at the `<template>` tag and labels the largest
//! contributors, so the warning says *where* the complexity comes from.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <div v-if="a && b">
//!     <li v-for="row in rows">
//!       <b v-if="row.x ? row.y : row.z">…</b>
//!       <!-- … and more nested branches … -->
//!     </li>
//!   </div>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <RowList v-if="a && b" :rows="rows" />
//! </template>
//! ```

use vize_davinci::fact::{Demand, FactConsumer, FactGroup};
use vize_s1_to_s2::pass::cfg::{
    COGNITIVE_WARN_ABOVE, CYCLOMATIC_WARN_ABOVE, Contribution, TemplateComplexityGroup,
    template_facts,
};

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "vue/max-template-complexity",
    description: "Limit a component's own template complexity (cyclomatic and cognitive)",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// How many contributors a diagnostic labels.
const MAX_LABELS: usize = 5;

/// Warn on a component whose own template complexity exceeds the thresholds.
#[derive(Default)]
pub struct MaxTemplateComplexity;

/// The rule reads exactly one fact group (TS-35: declared, never inferred).
impl FactConsumer for MaxTemplateComplexity {
    const NAME: &'static str = "vue/max-template-complexity";
    const DEMAND: Demand = Demand::NONE.with(TemplateComplexityGroup::ID);
}

impl Rule for MaxTemplateComplexity {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let Some(template) = ctx.sfc_descriptor().and_then(|sfc| sfc.template.as_ref()) else {
            return;
        };
        // A foreign template dialect or an external `src` has no S1 tree
        // here; the rule stays silent rather than guess.
        if template.src.is_some() || template.lang.as_deref().is_some_and(|lang| lang != "html") {
            return;
        }
        let (Ok(tag_start), Ok(start), Ok(end)) = (
            u32::try_from(template.loc.tag_start),
            u32::try_from(template.loc.start),
            u32::try_from(template.loc.end),
        ) else {
            return;
        };
        let Some(facts) = template_facts::<Self>(ctx.source, start, end) else {
            return;
        };
        if !facts.exceeds_default_thresholds() {
            return;
        }

        let mut diagnostic = LintDiagnostic::warn(
            META.name,
            vize_s0::cstr!(
                "Template complexity is too high: cyclomatic {} (limit {CYCLOMATIC_WARN_ABOVE}), cognitive {} (limit {COGNITIVE_WARN_ABOVE})",
                facts.cyclomatic,
                facts.cognitive,
            ),
            tag_start,
            start,
        )
        .with_help(
            "Move the deepest branches or loops into child components: a child's complexity \
             never counts toward this component's own score",
        );
        for row in top_contributors(&facts.contributions) {
            diagnostic = diagnostic.with_label(label(row), row.span.start, row.span.end);
        }
        ctx.report(diagnostic);
    }
}

/// The largest contributors: cognitive increment first, then cyclomatic,
/// then source order; rows that add nothing are never labelled.
fn top_contributors(rows: &[Contribution]) -> impl Iterator<Item = &Contribution> {
    let mut ranked: Vec<&Contribution> = rows
        .iter()
        .filter(|row| row.cognitive > 0 || row.cyclomatic > 0)
        .collect();
    ranked.sort_by(|left, right| {
        right
            .cognitive
            .cmp(&left.cognitive)
            .then(right.cyclomatic.cmp(&left.cyclomatic))
            .then(left.span.start.cmp(&right.span.start))
    });
    ranked.truncate(MAX_LABELS);
    ranked.sort_by_key(|row| row.sort_key());
    ranked.into_iter()
}

fn label(row: &Contribution) -> vize_s0::String {
    let nesting = if row.nesting == 0 {
        vize_s0::String::default()
    } else {
        vize_s0::cstr!(" at nesting {}", row.nesting)
    };
    vize_s0::cstr!(
        "{}{nesting}: +{} cognitive, +{} cyclomatic",
        row.kind.as_str(),
        row.cognitive,
        row.cyclomatic,
    )
}

#[cfg(test)]
#[path = "max_template_complexity_tests.rs"]
mod tests;
