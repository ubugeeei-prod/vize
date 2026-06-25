use super::{ProfileFileRow, ProfilePhase, ProfilePhaseKind, ProfileReport, render_profile_report};
use std::path::PathBuf;
use std::time::Duration;
use vize_carton::String;
use vize_carton::profiler::{
    AllocationSnapshot, CounterEntry, CounterSummary, ProfileEntry, ProfileSummary,
};

#[test]
#[allow(clippy::disallowed_macros)]
fn profile_report_snapshot() {
    let phases = [
        ProfilePhase {
            name: "collect files",
            duration: Duration::from_millis(10),
            kind: ProfilePhaseKind::Wall,
            note: "walked inputs",
        },
        ProfilePhase {
            name: "compile total",
            duration: Duration::from_millis(42),
            kind: ProfilePhaseKind::Cumulative,
            note: "rayon sum",
        },
    ];
    let files = [ProfileFileRow {
        path: PathBuf::from("src/App.vue"),
        bytes: 2048,
        total: Duration::from_millis(31),
        primary_label: "parse",
        primary: Duration::from_millis(8),
        secondary_label: "compile",
        secondary: Duration::from_millis(20),
        note: Some(String::from("1 style block")),
    }];
    let recommendations = [String::from(
        "src/App.vue spent most of its time in compile; inspect template complexity.",
    )];
    let operations = ProfileSummary {
        entries: vec![
            ProfileEntry {
                name: "atelier.sfc.parse",
                count: 1,
                total: Duration::from_millis(8),
                self_total: Duration::from_millis(7),
                child_total: Duration::from_millis(1),
                average: Duration::from_millis(8),
                self_average: Duration::from_millis(7),
                min: Duration::from_millis(8),
                max: Duration::from_millis(8),
                self_min: Duration::from_millis(7),
                self_max: Duration::from_millis(7),
                p50: Duration::from_millis(8),
                p95: Duration::from_millis(8),
                p99: Duration::from_millis(8),
                samples_over_1ms: 1,
                samples_over_10ms: 0,
                samples_over_100ms: 0,
            },
            ProfileEntry {
                name: "atelier.transform.element",
                count: 24,
                total: Duration::from_millis(6),
                self_total: Duration::from_millis(6),
                child_total: Duration::ZERO,
                average: Duration::from_micros(250),
                self_average: Duration::from_micros(250),
                min: Duration::from_micros(100),
                max: Duration::from_micros(600),
                self_min: Duration::from_micros(100),
                self_max: Duration::from_micros(600),
                p50: Duration::from_micros(256),
                p95: Duration::from_micros(512),
                p99: Duration::from_micros(1024),
                samples_over_1ms: 0,
                samples_over_10ms: 0,
                samples_over_100ms: 0,
            },
        ],
    };
    let counters = CounterSummary {
        entries: vec![
            CounterEntry {
                name: "cache.stats_compile.hits",
                samples: 1,
                total: 7,
                average: 7.0,
                min: 7,
                max: 7,
            },
            CounterEntry {
                name: "cache.stats_compile.misses",
                samples: 1,
                total: 3,
                average: 3.0,
                min: 3,
                max: 3,
            },
            CounterEntry {
                name: "cache.stats_compile.stores",
                samples: 1,
                total: 3,
                average: 3.0,
                min: 3,
                max: 3,
            },
            CounterEntry {
                name: "source.plate.sfc.requests",
                samples: 1,
                total: 1,
                average: 1.0,
                min: 1,
                max: 1,
            },
            CounterEntry {
                name: "source.block.template.bytes",
                samples: 1,
                total: 512,
                average: 512.0,
                min: 512,
                max: 512,
            },
            CounterEntry {
                name: "source.cache.miss.files",
                samples: 1,
                total: 1,
                average: 1.0,
                min: 1,
                max: 1,
            },
            CounterEntry {
                name: "lane.atelier.dom.requests",
                samples: 1,
                total: 1,
                average: 1.0,
                min: 1,
                max: 1,
            },
            CounterEntry {
                name: "dialect.vue3.files",
                samples: 1,
                total: 1,
                average: 1.0,
                min: 1,
                max: 1,
            },
            CounterEntry {
                name: "template_syntax.standard.files",
                samples: 1,
                total: 1,
                average: 1.0,
                min: 1,
                max: 1,
            },
            CounterEntry {
                name: "io.read.bytes",
                samples: 1,
                total: 2048,
                average: 2048.0,
                min: 2048,
                max: 2048,
            },
            CounterEntry {
                name: "io.read.calls",
                samples: 1,
                total: 1,
                average: 1.0,
                min: 1,
                max: 1,
            },
            CounterEntry {
                name: "syscall.fs.read_to_string.calls",
                samples: 1,
                total: 1,
                average: 1.0,
                min: 1,
                max: 1,
            },
        ],
    };
    let report = ProfileReport {
        title: "build",
        summary: "1 file on 4 threads",
        total: Duration::from_millis(50),
        phases: &phases,
        files: &files,
        slow_threshold: Duration::from_millis(30),
        throughput_bytes: Some(2048),
        operations: Some(&operations),
        counters: Some(&counters),
        allocations: Some(AllocationSnapshot {
            alloc_calls: 40,
            alloc_zeroed_calls: 2,
            alloc_failures: 0,
            alloc_zeroed_failures: 0,
            alloc_bytes: 65_536,
            alloc_zeroed_bytes: 4096,
            dealloc_calls: 35,
            dealloc_bytes: 49_152,
            realloc_calls: 6,
            realloc_failures: 0,
            realloc_old_bytes: 12_288,
            realloc_new_bytes: 24_576,
        }),
        recommendations: &recommendations,
    };

    insta::assert_snapshot!(render_profile_report(&report));
}
