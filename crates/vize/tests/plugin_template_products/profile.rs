use super::*;
use davinci_harness::alloc;
use std::{hint::black_box, time::Instant};

#[test]
#[ignore = "paired complete plugin batches and Doctor component products"]
fn measure_complete_plugin_and_component_responses() {
    alloc::mark_installed();
    let demands = ["templateScopes".into()];
    let request = spec(&demands, None);
    let mut failures = Vec::new();
    let mut measure = |name: &str, source: &str| {
        let run = |joined| {
            if joined {
                analyze(black_box(source), name, black_box(&request))
            } else {
                separate(black_box(source), name, black_box(&request))
            }
        };
        compare(&run(true), &run(false), name);
        let old = alloc::measure(|| run(false)).unwrap();
        let new = alloc::measure(|| run(true)).unwrap();
        let timing = |joined| {
            let start = Instant::now();
            for _ in 0..6 {
                drop(black_box(run(joined)));
            }
            start.elapsed().as_nanos() / 6
        };
        let mut pairs = Vec::new();
        for i in 0..35 {
            let first = i % 2 != 0;
            let a = timing(first);
            let b = timing(!first);
            if i >= 4 {
                pairs.push(if first { (b, a) } else { (a, b) });
            }
        }
        eprintln!(
            "plugin_template_time {name} calls={}/{} peak={}/{} ns={pairs:?}",
            old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
        );
        let median = |mut a: Vec<u128>| {
            a.sort_unstable();
            a[a.len() / 2]
        };
        let (a, b) = pairs.into_iter().unzip();
        if new.calls > old.calls
            || new.peak_bytes_over_start > old.peak_bytes_over_start
            || median(b) > median(a)
        {
            failures.push(name.to_owned());
        }
    };
    for (i, s) in directed().into_iter().enumerate() {
        measure(&format!("directed-{i}"), s);
    }
    originals(|i, n, s| {
        if i % 127 == 0 {
            measure(n, s)
        }
    });
    assert!(failures.is_empty(), "regressions on {failures:?}");
}
