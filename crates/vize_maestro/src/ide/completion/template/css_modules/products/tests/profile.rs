use super::*;
use davinci_harness::alloc;
use std::{hint::black_box, time::Instant};

#[test]
#[ignore = "compare complete member requests with ordinary checker and completion products"]
fn paired_style_module_requests() {
    alloc::mark_installed();
    let base = options();
    let mut failures = Vec::new();
    let mut measure = |name: &str, source: &str, at: usize| {
        in_document(source, at, |ctx| {
            let run = |joined| {
                if joined {
                    black_box(ctx)
                        .style_module_products(black_box(&base), true, false)
                        .unwrap()
                        .unwrap()
                } else {
                    original(black_box(ctx), black_box(&base), true, false)
                }
            };
            let candidate = run(true);
            let baseline = run(false);
            assert_equivalent(&candidate, &baseline, name);
            if candidate.completions.is_none() {
                return;
            }
            let old = alloc::measure(|| run(false)).unwrap();
            let new = alloc::measure(|| run(true)).unwrap();
            let time = |joined| {
                let start = Instant::now();
                for _ in 0..8 {
                    drop(black_box(run(joined)));
                }
                start.elapsed().as_nanos() / 8
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
                "style_module_time {name} calls={}/{} peak={}/{} ns={pairs:?}",
                old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
            );
            let (mut a, mut b): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
            a.sort_unstable();
            b.sort_unstable();
            if b[15] > a[15]
                || new.calls > old.calls
                || new.peak_bytes_over_start > old.peak_bytes_over_start
            {
                failures.push(name.to_owned());
            }
        });
    };
    for (language, body) in STYLES {
        let source = format!("{TEMPLATE}<style module lang='{language}'>{body}</style>");
        measure(
            language,
            &source,
            source.find(CURSOR).unwrap() + CURSOR.len(),
        );
    }
    original_modules(|name, source, offsets| {
        if let Some(&at) = offsets.get(1) {
            measure(name, source, at);
        }
    });
    assert!(failures.is_empty(), "regressions: {failures:?}");
}
