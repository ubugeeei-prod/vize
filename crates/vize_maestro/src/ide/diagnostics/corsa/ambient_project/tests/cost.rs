//! Full native responses and memory at the cold, repeated and post-drop boundaries.
use super::{NUMBER, PROVIDER, Project, fs, json, pack};
use davinci_harness::alloc;
use std::{sync::atomic::Ordering, time::Instant};

#[test]
#[ignore = "requires the repository's pinned TypeScript and Vue packages"]
fn repeated_native_reads_preserve_every_mapped_diagnostic() {
    #[cfg(feature = "micro-profiling")]
    alloc::mark_installed();
    for (count, open_declaration) in [(0, false), (1, false), (32, false), (128, false), (0, true)]
    {
        let p = Project::new();
        p.write(
            "tsconfig.json",
            &fs::read_to_string(pack().join("tsconfig.json.txt")).unwrap(),
        );
        p.write("src/components/GlobalBadge.vue", NUMBER);
        p.write("src/components.d.ts", PROVIDER);
        if open_declaration {
            p.open("src/components.d.ts", PROVIDER, "typescript");
        }
        let host = "src/views/App.vue";
        let elements = if count == 0 {
            "<GlobalBadge :level=\"1\" />\n".to_owned()
        } else {
            "<GlobalBadge :level=\"'x'\" />\n".repeat(count)
        };
        let text = format!("<template>\n{elements}</template>\n");
        p.write(host, &text);
        let snapshot = p.open(host, &text, "vue");
        let session = p.start();
        let read = || crate::runtime::block_on(session.diagnostics(&snapshot)).unwrap();

        let live = alloc::stats().live_bytes;
        let start = Instant::now();
        let (first, cold) = alloc::measure_returning(read);
        let cold_ns = start.elapsed().as_nanos();
        let retained = alloc::stats().live_bytes.saturating_sub(live);
        let expected: Vec<_> = (1..=count).map(|line| json!({
            "code":2322, "message":"Type 'string' is not assignable to type 'number'.",
            "range":{"start":{"line":line,"character":14},"end":{"line":line,"character":19}},
            "severity":1, "source":"vize/types"
        })).collect();
        assert_eq!(
            serde_json::to_value(&first).unwrap(),
            serde_json::Value::Array(expected),
            "{count} diagnostics"
        );
        let hits = &session.bridge.cache_stats().hits;
        let before = hits.load(Ordering::Relaxed);
        let (mut times, mut calls, mut peaks) = (Vec::new(), Vec::new(), Vec::new());
        for _ in 0..9 {
            let start = Instant::now();
            let (rows, heap) = alloc::measure_returning(read);
            times.push(start.elapsed().as_nanos());
            if let Some(heap) = heap {
                calls.push(heap.calls);
                peaks.push(heap.peak_bytes_over_start);
            }
            assert_eq!(rows, first);
        }
        assert_eq!(hits.load(Ordering::Relaxed) - before, 9);
        times.sort_unstable();
        calls.sort_unstable();
        peaks.sort_unstable();
        let warm_ns = times[4];
        let warm_allocations = calls.get(4).copied();
        let warm_peak = peaks.get(4).copied();
        drop((first, times, calls, peaks));
        let resident = alloc::stats().live_bytes.saturating_sub(live);
        println!(
            "ambient_sample {}",
            json!({
                "rows": count,
                "declaration": if open_declaration { "open" } else { "disk" },
                "warm_ns": warm_ns,
                "warm_allocations": warm_allocations,
                "warm_peak": warm_peak,
                "cold_ns": cold_ns,
                "cold_allocations": cold.map(|sample| sample.calls),
                "cold_peak": cold.map(|sample| sample.peak_bytes_over_start),
                "retained_with_response": retained,
                "retained_after_drop": resident
            })
        );
        crate::runtime::block_on(session.shutdown()).unwrap();
    }
}
