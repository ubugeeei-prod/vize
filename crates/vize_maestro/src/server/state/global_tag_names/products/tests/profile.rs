use super::*;
use davinci_harness::alloc;
use std::{hint::black_box, time::Instant};

#[test]
#[ignore = "interleave complete returned products with all three original tools"]
fn paired_declaration_products() {
    alloc::mark_installed();
    let state = ServerState::new();
    let cache = SourceSnapshotCache::default();
    let uri = Url::parse("file:///types/project.d.ts").unwrap();
    let mut failures = Vec::new();
    let mut compare = |name: &str, source: &str| {
        state
            .documents
            .open(uri.clone(), source.into(), 1, "typescript".into());
        let snapshot = cache.capture(&state.documents, &uri).unwrap();
        let run = |joined| {
            if joined {
                black_box(&snapshot)
                    .declaration_tool_products(black_box(&state))
                    .unwrap()
            } else {
                separate(black_box(source))
            }
        };
        assert_eq!(run(true), run(false), "{name}");
        let old = alloc::measure(|| run(false)).unwrap();
        let new = alloc::measure(|| run(true)).unwrap();
        let timing = |joined| {
            let start = Instant::now();
            for _ in 0..8 {
                drop(black_box(run(joined)));
            }
            start.elapsed().as_nanos() / 8
        };
        let mut pairs = Vec::new();
        for round in 0..35 {
            let first = round % 2 == 0;
            let a = timing(first);
            let b = timing(!first);
            if round >= 4 {
                pairs.push(if first { (b, a) } else { (a, b) });
            }
        }
        eprintln!(
            "declaration_time {name} calls={}/{} peak={}/{} ns={pairs:?}",
            old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
        );
        let median = |mut values: Vec<u128>| {
            values.sort_unstable();
            values[values.len() / 2]
        };
        let (a, b) = pairs.into_iter().unzip();
        if new.calls > old.calls
            || new.peak_bytes_over_start > old.peak_bytes_over_start
            || median(b) > median(a)
        {
            failures.push(name.to_owned());
        }
    };
    for (index, source) in directed().into_iter().enumerate() {
        compare(&format!("directed-{index}"), source);
    }
    production_inputs(compare);
    assert!(failures.is_empty(), "regressions on {failures:?}");
}
