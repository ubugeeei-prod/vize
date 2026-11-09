//! Differential for the scheduling contract. Rust lowers each reference
//! template, applies a named ordering scenario and records the TS-27
//! validator's exact codes in `schedule-contract.txt`. This applies the same
//! scenarios to the committed graph pages and recomputes the codes with
//! `order_codes`. Accepted scheduled programs are also executed to observe the
//! ordering that `accepted_schedule_orders_edges` proves.

use std::path::Path;

use super::dump::parse_program;
use super::program::{EdgeKind, Phase, Program, StateEdge};
use super::schedule::{order_codes, precedes};
use super::trace::{Backend, run};

pub const STEMS: [&str; 5] = [
    "rust-lowered-static-dynamic",
    "rust-lowered-control-slots",
    "rust-lowered-loop-keyed",
    "rust-lowered-loop-unkeyed",
    "rust-lowered-loop-nested",
];

pub const SCENARIOS: [&str; 9] = [
    "built",
    "scheduled",
    "scheduled-reversed-ops",
    "scheduled-reversed-first-edge",
    "scheduled-self-edge",
    "dangling-target",
    "duplicate-last-op",
    "foreign-scope-edge",
    "missing-scope-edge",
];

fn edge(source: u32, target: u32, kind: EdgeKind, effect: Option<u32>) -> StateEdge {
    StateEdge {
        source,
        target,
        kind,
        effect,
    }
}

fn next_id(ids: impl Iterator<Item = u32>) -> Option<u32> {
    ids.map(|id| id.checked_add(1))
        .try_fold(0, |max, id| Some(max.max(id?)))
}

/// The scenario applied to `program`, or `None` when it does not apply.
fn mutate(program: &Program, scenario: &str) -> Result<Option<Program>, String> {
    let (Some(first), Some(last)) = (program.ops.first(), program.ops.last()) else {
        return Ok(None);
    };
    let overflow = || String::from("id overflow");
    let next_op = next_id(program.ops.iter().map(|op| op.id)).ok_or_else(overflow)?;
    let next_effect = next_id(program.effects.iter().map(|e| e.id)).ok_or_else(overflow)?;
    let mut out = program.clone();
    let scheduled = |mut out: Program| {
        out.phase = Phase::Scheduled;
        out
    };
    Ok(Some(match scenario {
        "built" => out,
        "scheduled" => scheduled(out),
        "scheduled-reversed-ops" => {
            out.ops.reverse();
            scheduled(out)
        }
        "scheduled-reversed-first-edge" => {
            let Some(head) = out.edges.first_mut() else {
                return Ok(None);
            };
            *head = edge(head.target, head.source, head.kind, head.effect);
            scheduled(out)
        }
        "scheduled-self-edge" => {
            out.edges
                .push(edge(first.id, first.id, EdgeKind::DomOrder, None));
            scheduled(out)
        }
        "dangling-target" => {
            out.edges
                .push(edge(first.id, next_op, EdgeKind::DataDependency, None));
            out
        }
        "duplicate-last-op" => {
            out.ops.push(*last);
            out
        }
        "foreign-scope-edge" => {
            let Some(scope) = program.effects.first() else {
                return Ok(None);
            };
            out.edges.push(edge(
                first.id,
                last.id,
                EdgeKind::EffectOrder,
                Some(scope.id),
            ));
            out
        }
        "missing-scope-edge" => {
            out.edges.push(edge(
                first.id,
                last.id,
                EdgeKind::EffectOrder,
                Some(next_effect),
            ));
            out
        }
        _ => return Err(format!("unknown scheduling scenario {scenario}")),
    }))
}

fn observe(stem: &str, scenario: &str, program: &Program) -> Result<Option<String>, String> {
    let Some(mutated) = mutate(program, scenario)? else {
        return Ok(None);
    };
    let codes = order_codes(&mutated);
    if !matches!(scenario, "built" | "scheduled") && codes.is_empty() {
        return Err(format!("{stem} {scenario}: the scenario must be rejected"));
    }
    if codes.is_empty() && mutated.phase == Phase::Scheduled {
        for backend in Backend::ALL {
            let trace = run(backend, &mutated);
            for state in &mutated.edges {
                if !precedes(&trace, state.source, state.target) {
                    return Err(format!(
                        "{stem}: accepted schedule executes op#{} too early",
                        state.target
                    ));
                }
            }
        }
    }
    let codes = match codes.is_empty() {
        true => String::from("-"),
        false => codes.join(","),
    };
    Ok(Some(format!("{stem} {scenario} {codes}\n")))
}

/// Recompute `schedule-contract.txt` from the committed graph pages.
pub fn schedule_contract(fixtures: &Path) -> Result<String, String> {
    let mut actual = String::new();
    for stem in STEMS {
        let path = fixtures.join(format!("{stem}.s3.dump"));
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{stem}: {e}"))?;
        let program = parse_program(&text).map_err(|e| format!("{stem}: {e}"))?;
        for scenario in SCENARIOS {
            if let Some(line) = observe(stem, scenario, &program)? {
                actual.push_str(&line);
            }
        }
    }
    Ok(actual)
}
