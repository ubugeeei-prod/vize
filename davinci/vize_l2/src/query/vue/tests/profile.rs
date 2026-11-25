//! Compare two existing independent passes with one fused ordinary-script pass.
//! The original parse is shared. Each half starts from its own fresh arena.

use super::*;
use crate::framework::vue::{OptionsObserver, OrdinaryObserver};
use alloc::vec::Vec;
use core::hint::black_box;
use std::time::Instant;
use vize_l0::String;

fn sample(source: &str, repetitions: u32, fused: bool) -> u128 {
    let arena = Allocator::default();
    let parsed = Parsed::new(&arena, source).unwrap();
    let view = parsed.l1ir.admitted().unwrap().ordinary().unwrap();
    let syntax = parsed.ordinary.as_ref().unwrap();
    let input = || {
        ProgramInput::checked(
            syntax.admitted_program().unwrap(),
            view.block(),
            view.container_index(),
        )
        .unwrap()
    };
    let start = Instant::now();
    for _ in 0..repetitions {
        if fused {
            let owner = parsed.produce(&arena, ALL).unwrap();
            black_box(owner.model().options().unwrap());
            black_box(owner);
        } else {
            let mut first = FileProducer::new(&arena, source).unwrap();
            let mut binding = OrdinaryObserver::new();
            first
                .program_observed(input(), ProgramScope::Module, &mut binding)
                .unwrap();
            black_box(binding.finish().unwrap());
            black_box(first.finish().unwrap());
            let mut second = FileProducer::new(&arena, source).unwrap();
            let mut options = OptionsObserver::new();
            second
                .program_observed(input(), ProgramScope::Module, &mut options)
                .unwrap();
            black_box(options.finish().unwrap());
            black_box(second.finish().unwrap());
        }
    }
    start.elapsed().as_nanos() / u128::from(repetitions)
}

#[test]
#[ignore = "paired opt-in observations, run in the optimized ci-opt profile"]
fn paired_ordinary_passes() {
    for (bindings, repetitions) in [(1, 128), (32, 32), (192, 8)] {
        let mut source = String::from("<script>");
        for i in 0..bindings {
            use core::fmt::Write;
            write!(source, "export const x{i}={i};").unwrap();
        }
        source.push_str("export default {props:['value'], inject:['service']}</script>");
        sample(&source, repetitions, false);
        sample(&source, repetitions, true);
        let (mut base, mut head, mut ratios) = (Vec::new(), Vec::new(), Vec::new());
        for run in 0..31 {
            let (b, h) = if run % 2 == 0 {
                (
                    sample(&source, repetitions, false),
                    sample(&source, repetitions, true),
                )
            } else {
                let h = sample(&source, repetitions, true);
                (sample(&source, repetitions, false), h)
            };
            base.push(b);
            head.push(h);
            ratios.push(h as f64 / b as f64);
        }
        base.sort_unstable();
        head.sort_unstable();
        ratios.sort_by(f64::total_cmp);
        std::eprintln!(
            "bindings={bindings} n=31 base_p50_ns={} head_p50_ns={} base_p95_ns={} head_p95_ns={} paired_p50={:.3}",
            base[15],
            head[15],
            base[29],
            head[29],
            ratios[15]
        );
    }
}
