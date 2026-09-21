//! P3-10 try-measure-commit extraction.
//!
//! The pass reads the placement alternatives [`crate::placement::annotate`]
//! recorded and chooses among them Flambda2-style: for each candidate, in
//! record order, it *performs* the placement on a trial plan, *simplifies*
//! locally (a hoist absorbs its subtree's effect units, a group merges its
//! unit with the leader's and unions their keys, a cache drops its unit),
//! *measures* the three [`Metrics`] with the key-set fact approximation in
//! scope, and *commits* only under the pinned [`rule::judge`]. Every measured
//! trial spends one unit of the tier's per-component candidate budget.
//!
//! The pass writes nothing but `PlacementRecord::chosen`: ops, regions, edges,
//! effect scopes, and operands stay canonical, so partition facts exported
//! for the canonical program remain exact. [`EXTRACT`] states that as
//! `Preserved::ALL`.

mod folio;
mod measure;
mod report;
mod rule;
mod tier;

pub use folio::{FolioDecision, S3ExtractionFolio};
pub use measure::{CACHE_BYTES, EFFECT_UNIT_BYTES, HOIST_BYTES, Metric, Metrics};
pub use report::{Decision, DecisionKind, Delta, EXTRACT_PASS, Extraction, Reason};
pub use rule::{Rejection, epsilon_pct, judge};
pub use tier::{OptTier, OptimizationBudget, TiePolicy};

use alloc::vec::Vec;

use vize_davinci::pass::{Fusability, PassDesc, PassKind, Preserved};

use crate::op::Program;
use crate::placement::facts::Index;
use crate::placement::{Placement, PlacementRecord};
use measure::{Choice, Model};

/// The pass description: optional, whole-program, and graph-preserving.
pub const EXTRACT: PassDesc = PassDesc::new(
    EXTRACT_PASS,
    PassKind::Optional,
    Fusability::Barrier,
    Preserved::ALL,
);

/// Choose placements for `program` at `tier`, starting from the all-inline
/// plan, and commit the winners into `program.placements`.
///
/// Expects placements that `annotate` recorded or that `S3V010` accepts;
/// records whose op does not resolve are left alone.
pub fn extract(program: &mut Program<'_>, tier: OptTier) -> Extraction {
    let (chosen, extraction) = plan(&Index::new(program), tier);
    for (record, chosen) in program.placements.iter_mut().zip(chosen) {
        record.chosen = chosen;
    }
    extraction
}

fn plan(index: &Index<'_, '_>, tier: OptTier) -> (Vec<Placement>, Extraction) {
    let program = index.program;
    let budget = tier.budget();
    let model = Model::new(index);
    let mut plan = alloc::vec![Choice::INLINE; program.ops.len()];
    let before = model.measure(&plan);
    let mut current = before;
    let mut left = budget.candidate_budget;
    let mut decisions = Vec::new();
    let mut chosen = Vec::with_capacity(program.placements.len());
    for record in program.placements.iter() {
        let Some(position) = index.position(record.op) else {
            chosen.push(Placement::Inline);
            continue;
        };
        let span = program.ops[position].span;
        let mut committed = Placement::Inline;
        for placement in record.alternatives.iter() {
            // One committed shape per op; later alternatives are not tried.
            if placement == Placement::Inline || committed != Placement::Inline {
                continue;
            }
            let choice = Choice {
                placement,
                leader: record.leader.and_then(|leader| index.position(leader)),
            };
            let (reason, after) =
                if let Some(reason) = blocked(index, &plan, record, position, choice) {
                    (reason, current)
                } else if left == 0 {
                    (Reason::BudgetExhausted, current)
                } else {
                    left -= 1;
                    plan[position] = choice;
                    let after = model.measure(&plan);
                    match judge(current, after, &budget) {
                        Ok(()) => (Reason::Committed, after),
                        Err(Rejection::Regressed(metric)) => (Reason::Regressed(metric), after),
                        Err(Rejection::NoImprovement) => (Reason::NoImprovement, after),
                    }
                };
            let kind = if reason == Reason::Committed {
                committed = placement;
                DecisionKind::Applied
            } else {
                plan[position] = Choice::INLINE;
                DecisionKind::Missed
            };
            decisions.push(Decision {
                op: record.op,
                span,
                placement,
                kind,
                reason,
                delta: Delta::between(current, after),
                budget_left: left,
            });
            if kind == DecisionKind::Applied {
                current = after;
            }
        }
        chosen.push(committed);
    }
    let extraction = Extraction {
        tier,
        candidate_budget: budget.candidate_budget,
        budget_left: left,
        before,
        after: current,
        decisions,
    };
    (chosen, extraction)
}

/// Whether the committed plan already rules the candidate out, before any
/// budget is spent on measuring it.
fn blocked(
    index: &Index<'_, '_>,
    plan: &[Choice],
    record: &PlacementRecord,
    position: usize,
    choice: Choice,
) -> Option<Reason> {
    match choice.placement {
        Placement::Hoist => {
            let program = index.program;
            let mut region = index.region(program.ops[position].region);
            for _ in 0..=program.regions.len() {
                let owner = index.position(region?.owner?)?;
                if plan[owner].placement == Placement::Hoist {
                    return Some(Reason::Subsumed);
                }
                region = index.region(program.ops[owner].region);
            }
            None
        }
        Placement::Group => {
            let leader = record.leader?;
            let mut current = index.keyed_pred(record.op);
            for _ in 0..index.program.ops.len() {
                let Some(pred) = current else {
                    return Some(Reason::NotContiguous);
                };
                if pred == leader {
                    return None;
                }
                let member = index.position(pred).map(|pred| plan[pred]);
                if member.is_none_or(|member| {
                    member.placement != Placement::Group || member.leader != choice.leader
                }) {
                    return Some(Reason::NotContiguous);
                }
                current = index.keyed_pred(pred);
            }
            Some(Reason::NotContiguous)
        }
        Placement::Inline | Placement::Cache => None,
    }
}
