use super::*;
use davinci_harness::alloc;
use std::{hint::black_box, time::Instant};

#[test]
#[ignore = "paired complete graph registration, plugin packets and scored Doctor reports"]
fn measure_complete_projects() {
    let demands = ["templateScopes".into()];
    let request = spec(&demands, None);
    alloc::mark_installed();
    let mut projects = examples();
    projects.extend([("small", board(2)), ("hotspot", board(12))]);
    let mut regressions = Vec::new();
    for (name, files) in projects {
        let _ = equal(&files, &request);
        let run = |joined| project(black_box(&files), black_box(&request), joined);
        let a = alloc::measure(|| run(false)).unwrap();
        let b = alloc::measure(|| run(true)).unwrap();
        let timing = |joined| {
            let start = Instant::now();
            for _ in 0..3 {
                drop(black_box(run(joined)));
            }
            start.elapsed().as_nanos() / 3
        };
        let mut pairs = Vec::new();
        for index in 0..35 {
            let first = index % 2 == 0;
            let x = timing(first);
            let y = timing(!first);
            if index >= 4 {
                pairs.push(if first { (y, x) } else { (x, y) });
            }
        }
        let median = |mut samples: Vec<u128>| {
            samples.sort_unstable();
            samples[samples.len() / 2]
        };
        eprintln!(
            "doctor_project {name} files={} calls={}/{} peak={}/{} ns={pairs:?}",
            files.len(),
            a.calls,
            b.calls,
            a.peak_bytes_over_start,
            b.peak_bytes_over_start
        );
        let (old, new) = pairs.into_iter().unzip();
        if b.calls > a.calls
            || b.peak_bytes_over_start > a.peak_bytes_over_start
            || median(new) > median(old)
        {
            regressions.push(name);
        }
    }
    assert!(
        regressions.is_empty(),
        "project regressions {regressions:?}"
    );
}
