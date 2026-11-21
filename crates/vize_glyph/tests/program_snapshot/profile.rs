use super::*;
use davinci_harness::alloc::{self, CountingAllocator};
use std::{hint::black_box, time::Instant};
use vize_l1::embed::syntax::parse_program_for_format;

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator::mimalloc();

fn independent<const SEMANTIC: usize, const PRINTING: usize>(text: &str) {
    let arena = Allocator::default();
    let source = EmbedSource::authored(text, vize_l0::Span::new(0, text.len() as u32)).unwrap();
    let options = ProgramOptions::module(Lang::Ts);
    for _ in 0..SEMANTIC {
        let syntax = parse_program_once(&arena, source, options);
        black_box(syntax.admitted_program().unwrap().program().body.len());
    }
    for _ in 0..PRINTING {
        let parsed = parse_program_for_format(arena.as_oxc(), source.text(), options.source_type());
        assert!(parsed.diagnostics.is_empty());
        black_box(parsed.program.body.len());
    }
}

fn shared<const SEMANTIC: usize, const PRINTING: usize>(text: &str) {
    let arena = Allocator::default();
    let source = EmbedSource::authored(text, vize_l0::Span::new(0, text.len() as u32)).unwrap();
    let semantic = parse_program_once(&arena, source, ProgramOptions::module(Lang::Ts));
    let snapshot = ProgramSnapshot::observe_printing(&arena, &semantic).unwrap();
    for _ in 0..SEMANTIC {
        black_box(
            snapshot
                .semantic()
                .admitted_program()
                .unwrap()
                .program()
                .body
                .len(),
        );
    }
    for _ in 0..PRINTING {
        black_box(snapshot.for_print().unwrap().body.len());
    }
}

#[test]
#[ignore = "paired optimized profile over a small script and a generated TypeScript module"]
fn repeated_profile_requests() {
    use std::fmt::Write;
    alloc::mark_installed();
    let small = "export const x = 1;";
    let mut medium = String::default();
    for i in 0..64 {
        writeln!(medium, "export const item{i}: string = 'original {i}';").unwrap();
    }
    for (input, text) in [
        ("single export", small),
        ("64 typed exports", medium.as_str()),
    ] {
        measure_demand::<1, 1>(input, text);
        measure_demand::<4, 2>(input, text);
    }
    let arena = Allocator::default();
    let source = EmbedSource::authored(small, vize_l0::Span::new(0, small.len() as u32)).unwrap();
    let semantic = parse_program_once(&arena, source, ProgramOptions::module(Lang::Js));
    let snapshot = ProgramSnapshot::observe_printing(&arena, &semantic).unwrap();
    let metrics = alloc::measure(|| {
        black_box(snapshot.semantic());
        black_box(snapshot.for_print().unwrap());
    })
    .unwrap();
    assert_eq!(metrics.calls, 0);
}

fn measure_demand<const SEMANTIC: usize, const PRINTING: usize>(input: &str, text: &str) {
    let first = || independent::<SEMANTIC, PRINTING>(text);
    let second = || shared::<SEMANTIC, PRINTING>(text);
    let rounds = if text.len() < 100 { 256 } else { 48 };
    let measure = |f: &dyn Fn()| {
        let start = Instant::now();
        for _ in 0..rounds {
            f();
        }
        start.elapsed().as_nanos() / rounds
    };
    first();
    second();
    let mut originals = Vec::new();
    let mut snapshots = Vec::new();
    let mut ratios = Vec::new();
    for i in 0..31 {
        let (old, new) = if i % 2 == 0 {
            (measure(&first), measure(&second))
        } else {
            let new = measure(&second);
            (measure(&first), new)
        };
        originals.push(old);
        snapshots.push(new);
        ratios.push(new as f64 / old as f64);
    }
    for (pair, (old, new)) in originals.iter().zip(&snapshots).enumerate() {
        let first = if pair % 2 == 0 {
            "independent"
        } else {
            "shared"
        };
        eprintln!(
            "sample {input} semantic={SEMANTIC} print={PRINTING} pair={pair} first={first} iterations={rounds} independent_ns={old} shared_ns={new} shared_over_independent={:.6}",
            *new as f64 / *old as f64
        );
    }
    originals.sort_unstable();
    snapshots.sort_unstable();
    ratios.sort_by(f64::total_cmp);
    let old = alloc::measure(first).unwrap();
    let new = alloc::measure(second).unwrap();
    eprintln!(
        "{input} semantic={SEMANTIC} print={PRINTING} n=31 base_p50_ns={} shared_p50_ns={} paired_p50={:.3} base_allocations={} shared_allocations={} base_peak_bytes={} shared_peak_bytes={}",
        originals[15],
        snapshots[15],
        ratios[15],
        old.calls,
        new.calls,
        old.peak_bytes_over_start,
        new.peak_bytes_over_start
    );
}
