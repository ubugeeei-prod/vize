//! P3-15 bridge for the S3 scheduling contract.
//!
//! Every Rust-lowered reference template is lowered, mutated by a named
//! ordering scenario and checked by the TS-27 validator. The committed
//! `schedule-contract.txt` records the exact violation codes. The Lean
//! reference re-applies the same scenarios to the committed graph Folios and
//! recomputes the codes with its independent model of the validator's edge
//! contract, whose acceptance is proved to order every state edge.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::path::Path;

use vize_s0::Allocator;
use vize_s3::op::{EdgeKind, EffectId, OpId, Phase, Program, StateEdge};
use vize_s3::verify::verify;

const STEMS: [&str; 5] = [
    "rust-lowered-static-dynamic",
    "rust-lowered-control-slots",
    "rust-lowered-loop-keyed",
    "rust-lowered-loop-unkeyed",
    "rust-lowered-loop-nested",
];

const SCENARIOS: [&str; 9] = [
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

fn source(stem: &str) -> String {
    if stem == "rust-lowered-static-dynamic" {
        return crate::STATIC_DYNAMIC_SOURCE.to_string();
    }
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../formal/impeto/fixtures");
    std::fs::read_to_string(fixtures.join(format!("{stem}.template.txt"))).unwrap()
}

fn next_op_id(program: &Program<'_>) -> OpId {
    OpId::new(program.ops.iter().map(|op| op.id.index()).max().unwrap() + 1)
}

fn next_effect_id(program: &Program<'_>) -> EffectId {
    EffectId::new(
        program
            .effects
            .iter()
            .map(|effect| effect.id.index() + 1)
            .max()
            .unwrap_or(0),
    )
}

/// Returns false when the scenario does not apply to this program.
fn mutate(program: &mut Program<'_>, scenario: &str) -> bool {
    let first = program.ops[0].id;
    let last = program.ops[program.ops.len() - 1].id;
    match scenario {
        "built" => {}
        "scheduled" => program.phase = Phase::Scheduled,
        "scheduled-reversed-ops" => {
            program.phase = Phase::Scheduled;
            program.ops.reverse();
        }
        "scheduled-reversed-first-edge" => {
            let Some(edge) = program.edges.first_mut() else {
                return false;
            };
            core::mem::swap(&mut edge.from, &mut edge.to);
            program.phase = Phase::Scheduled;
        }
        "scheduled-self-edge" => {
            program.phase = Phase::Scheduled;
            program.push_edge(StateEdge::new(first, first, EdgeKind::DomOrder));
        }
        "dangling-target" => {
            let missing = next_op_id(program);
            program.push_edge(StateEdge::new(first, missing, EdgeKind::DataDependency));
        }
        "duplicate-last-op" => {
            let op = program.ops[program.ops.len() - 1];
            program.push_op(op);
        }
        "foreign-scope-edge" => {
            let Some(effect) = program.effects.first().map(|effect| effect.id) else {
                return false;
            };
            program.push_edge(StateEdge::scoped(
                first,
                last,
                EdgeKind::EffectOrder,
                effect,
            ));
        }
        "missing-scope-edge" => {
            let missing = next_effect_id(program);
            program.push_edge(StateEdge::scoped(
                first,
                last,
                EdgeKind::EffectOrder,
                missing,
            ));
        }
        _ => unreachable!("unknown scheduling scenario {scenario}"),
    }
    true
}

#[test]
fn schedule_contract_codes_match_lean_reference() {
    let mut actual = String::new();
    for stem in STEMS {
        let source = source(stem);
        for scenario in SCENARIOS {
            let allocator = Allocator::default();
            let (tree, errors) = vize_s1::parse(&allocator, source.trim_end());
            assert!(errors.is_empty(), "{errors:?}");
            let s2 = vize_s1_to_s2::lower(&allocator, &tree, &errors);
            let mut program = vize_s2_to_s3::lower(&allocator, &s2.root).program;
            assert_eq!(verify(&program), [], "{stem} must lower to valid S3");
            if !mutate(&mut program, scenario) {
                continue;
            }
            let codes: Vec<&str> = verify(&program)
                .iter()
                .map(|violation| violation.code.as_str())
                .collect();
            let codes = if codes.is_empty() {
                "-".to_string()
            } else {
                codes.join(",")
            };
            actual.push_str(&format!("{stem} {scenario} {codes}\n"));
        }
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../formal/impeto/fixtures/schedule-contract.txt");
    if std::env::var("VIZE_UPDATE_SCHEDULE_CONTRACT_FIXTURE").as_deref() == Ok("1") {
        std::fs::write(&path, &actual).unwrap();
    }
    assert_eq!(actual, std::fs::read_to_string(&path).unwrap());
}
