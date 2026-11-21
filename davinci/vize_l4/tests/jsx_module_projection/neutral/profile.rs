//! In-process paired timings for one-shot and repeated projections of one file.
//! Run alone with the optimized CI profile. The parse is outside both arms.

use super::*;
use std::{hint::black_box, time::Instant};
use vize_l0::{String, append};
use vize_l1_to_l2::framework::vue::jsx::lower_module;

#[test]
#[ignore = "run the optimized paired module projection profile separately"]
fn paired_module_profile() {
    for roots in [1, 32, 192] {
        let mut source = String::default();
        for i in 0..roots {
            append!(
                &mut source,
                "export const view{i} = (value: number) => <section id={{String(value)}}><span>{{value}}</span></section>;\n"
            );
        }
        let syntax_arena = Allocator::default();
        let observation =
            Parser::new(syntax_arena.as_oxc(), &source, SourceType::tsx()).parse_observed();
        let input = observation.admitted().unwrap();
        let program = input.program();
        let scoping = SemanticBuilder::new()
            .build(program)
            .semantic
            .into_scoping();
        let options = ModuleOptions {
            dialect: &VIZE,
            default_exec: Exec::ReRender,
        };
        let bodies = Allocator::default();
        let retained = lower_module(&bodies, &source, program, Some(&scoping), options)
            .into_components()
            .unwrap();
        let prepared = lower_module_def(&bodies, &input, Some(&scoping), options).unwrap();
        paired(
            "one-shot",
            roots,
            || {
                let body = Allocator::default();
                let old = lower_module(&body, &source, program, Some(&scoping), options)
                    .into_components()
                    .unwrap();
                black_box(project_jsx_module(program, &old).unwrap());
            },
            || {
                let body = Allocator::default();
                let new = lower_module_def(&body, &input, Some(&scoping), options).unwrap();
                black_box(project_module(&new.module).unwrap());
            },
        );
        paired(
            "projection",
            roots,
            || {
                black_box(project_jsx_module(program, &retained).unwrap());
            },
            || {
                black_box(project_module(&prepared.module).unwrap());
            },
        );
    }
}

fn paired(label: &str, roots: usize, mut old: impl FnMut(), mut new: impl FnMut()) {
    const SAMPLES: usize = 31;
    let runs = if roots == 1 { 128 } else { 8 };
    let measure = |f: &mut dyn FnMut()| {
        let start = Instant::now();
        for _ in 0..runs {
            f();
        }
        start.elapsed().as_nanos() / runs
    };
    old();
    new();
    let mut baseline = [0; SAMPLES];
    let mut changed = [0; SAMPLES];
    let mut ratio = [0.0; SAMPLES];
    for (i, ((a, b), r)) in baseline
        .iter_mut()
        .zip(&mut changed)
        .zip(&mut ratio)
        .enumerate()
    {
        if i % 2 == 0 {
            *a = measure(&mut old);
            *b = measure(&mut new);
        } else {
            *b = measure(&mut new);
            *a = measure(&mut old);
        }
        *r = *b as f64 / *a as f64;
    }
    baseline.sort_unstable();
    changed.sort_unstable();
    ratio.sort_unstable_by(f64::total_cmp);
    println!(
        "{label} roots={roots} ns median old={} new={} paired ratio={:.3}",
        baseline[SAMPLES / 2],
        changed[SAMPLES / 2],
        ratio[SAMPLES / 2]
    );
}
