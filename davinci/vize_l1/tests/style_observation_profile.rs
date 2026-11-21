//! Qualify the combined structural and class-name workload with both views live.

#![cfg(test)]

use davinci_harness::alloc::{self, CountingAllocator};
use std::{hint::black_box, time::Instant};
use vize_l0::{SourceBlock, SourceRoot, String};
use vize_l1::css::lang::{StyleLanguage, StyleObservation};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator::mimalloc();

fn independent(language: StyleLanguage, block: SourceBlock<'_>) {
    let sheet = language.structure(block.source(), block.start());
    let classes = language.authored_classes(block.source());
    black_box((sheet.nodes(), sheet.comments(), classes.as_ref()));
}

fn together(language: StyleLanguage, block: SourceBlock<'_>) {
    let observed = StyleObservation::read(language, block);
    black_box((
        observed.structure().nodes(),
        observed.structure().comments(),
        observed.authored_classes(),
    ));
}

fn measure(operation: impl Fn(), iterations: u128) -> u128 {
    let start = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    start.elapsed().as_nanos() / iterations
}

fn paired(label: &str, language: StyleLanguage, source: &str) {
    let block = SourceRoot::new(source).unwrap().whole_block();
    let old = || independent(language, black_box(block));
    let new = || together(language, black_box(block));
    old();
    new();
    let iterations = if source.len() < 200 { 128 } else { 8 };
    let mut baseline = Vec::new();
    let mut shared = Vec::new();
    let mut ratios = Vec::new();
    for pair in 0..31 {
        let (a, b) = if pair % 2 == 0 {
            (measure(old, iterations), measure(new, iterations))
        } else {
            let b = measure(new, iterations);
            (measure(old, iterations), b)
        };
        baseline.push(a);
        shared.push(b);
        ratios.push(b as f64 / a as f64);
    }
    for (pair, (a, b)) in baseline.iter().zip(&shared).enumerate() {
        let first = if pair % 2 == 0 {
            "independent"
        } else {
            "shared"
        };
        eprintln!(
            "sample {label} pair={pair} first={first} iterations={iterations} independent_ns={a} shared_ns={b} shared_over_independent={:.6}",
            *b as f64 / *a as f64
        );
    }
    baseline.sort_unstable();
    shared.sort_unstable();
    ratios.sort_by(f64::total_cmp);
    let a = alloc::measure(old).unwrap();
    let b = alloc::measure(new).unwrap();
    eprintln!(
        "{label} p50_ns={}/{} p95_ns={}/{} paired={:.3} calls={}/{} peak_bytes={}/{}",
        baseline[15],
        shared[15],
        baseline[29],
        shared[29],
        ratios[15],
        a.calls,
        b.calls,
        a.peak_bytes_over_start,
        b.peak_bytes_over_start
    );
    assert!(b.calls <= a.calls);
    let observed = StyleObservation::read(language, block);
    let repeat = alloc::measure(|| {
        black_box((observed.structure().nodes(), observed.authored_classes()));
    })
    .unwrap();
    assert_eq!(repeat.calls, 0);
}

#[test]
#[ignore = "paired optimized measurements with a process-global allocator"]
fn combined_style_reads() {
    use StyleLanguage::{Css, Less, Sass, Scss, Stylus};
    use std::fmt::Write;
    alloc::mark_installed();
    for (label, language, source) in [
        ("CSS", Css, ".card { color:red; }"),
        ("SCSS", Scss, ".card { &__item { color:red; } }"),
        ("Less", Less, ".card { &-item { color:red; } }"),
        ("Sass", Sass, ".card\n  &__item\n    color: red\n"),
        ("Stylus", Stylus, ".card\n  &-item\n    color red\n"),
    ] {
        paired(label, language, source);
    }
    let mut source = String::default();
    for i in 0..64 {
        writeln!(source, ".row{i} {{ &__title {{ color: red; }} }}").unwrap();
    }
    paired("SCSS/64", Scss, &source);
}
