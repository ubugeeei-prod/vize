use super::*;
use davinci_harness::alloc::{self, CountingAllocator};
use std::{hint::black_box, time::Instant};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator::mimalloc();

fn measure(source: &str, filename: &str, failures: &mut Vec<String>) {
    let run = |joined| {
        let arena = Allocator::default();
        if joined {
            black_box(parse_art_products(
                &arena,
                black_box(source),
                ArtParseOptions {
                    filename: black_box(filename).into(),
                },
            ));
        } else {
            black_box(original(&arena, black_box(source), black_box(filename)));
        }
    };
    let old = alloc::measure(|| run(false)).unwrap();
    let new = alloc::measure(|| run(true)).unwrap();
    let time = |joined| {
        let start = Instant::now();
        for _ in 0..12 {
            run(joined);
        }
        start.elapsed().as_nanos() / 12
    };
    let mut pairs = Vec::new();
    for round in 0..35 {
        let first = round % 2 == 0;
        let a = time(first);
        let b = time(!first);
        if round >= 4 {
            pairs.push(if first { (b, a) } else { (a, b) });
        }
    }
    eprintln!(
        "art_product_time {filename} calls={}/{} peak={}/{} ns={pairs:?}",
        old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
    );
    let median = |mut series: Vec<u128>| {
        series.sort_unstable();
        series[series.len() / 2]
    };
    let (old_ns, new_ns) = pairs.into_iter().unzip();
    if new.calls > old.calls
        || new.peak_bytes_over_start > old.peak_bytes_over_start
        || median(new_ns) > median(old_ns)
    {
        failures.push(filename.into());
    }
}

#[test]
#[ignore = "complete gallery descriptor and warnings timing pairs"]
fn complete_responses_share_the_original_art_surface() {
    alloc::mark_installed();
    let mut failures = Vec::new();
    for (i, source) in DIRECTED.iter().enumerate() {
        measure(source, &format!("directed-{i}.art.vue"), &mut failures);
    }
    originals(|index, name, source| {
        if name.ends_with(".art.vue") || name.ends_with(".art.vue.txt") || index % 113 == 0 {
            measure(source, name, &mut failures);
        }
    });
    assert!(failures.is_empty(), "regressions on {failures:?}");
}
