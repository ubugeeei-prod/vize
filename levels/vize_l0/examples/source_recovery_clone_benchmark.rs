//! Source-copy admission microbenchmark; no backend, daemon or persisted cache.
#![expect(
    clippy::expect_used,
    reason = "a source benchmark control fails by panicking"
)]

#[cfg(feature = "micro-profiling")]
#[global_allocator]
static ALLOCATOR: vize_l0::profiler::ProfilingAllocator<std::alloc::System> =
    vize_l0::profiler::ProfilingAllocator::from_allocator(std::alloc::System);

fn main() {
    let args: vize_l0::SmallVec<[_; 4]> = std::env::args()
        .skip(1)
        .map(vize_l0::String::from)
        .collect();
    let iterations = args
        .first()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(10);
    let minimum_bytes = args
        .get(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(4096);
    let instrument = args.get(2).is_some_and(|value| value == "profile");
    let scenario = args.get(3).map_or("irreparable", |value| value.as_str());
    assert!(matches!(scenario, "irreparable" | "valid" | "repairable"));
    assert!(!instrument || cfg!(feature = "micro-profiling"));

    let mut source = vize_l0::String::default();
    if scenario == "irreparable" {
        source.push_str("const broken = ;\r\n");
    }
    let mut rows = 0;
    while source.len() < minimum_bytes {
        use std::fmt::Write;
        write!(source, "const 雪_{rows}: string = \"日本🙂\";\r\n").expect("actual code builds");
        rows += 1;
    }
    if scenario != "irreparable" {
        source.push_str(if scenario == "valid" {
            "const damaged = 0;\r\n"
        } else {
            "const damaged = missing.;\r\n"
        });
    }
    assert!(source.contains("日本🙂") && source.contains("\r\n"));
    let source_hash = vize_l0::hash::hash_bytes(source.as_bytes());
    let expected_bindings = if scenario == "irreparable" {
        0
    } else {
        rows + 1
    };
    let profiler = vize_l0::profiler::global_profiler();
    if instrument {
        profiler.clear();
        profiler.enable();
    }
    let started = std::time::Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..iterations {
        let result = vize_croquis::script_parser::parse_script_setup(std::hint::black_box(&source));
        checksum += std::hint::black_box(result.bindings.bindings.len()) as u64;
    }
    let elapsed_ns = started.elapsed().as_nanos();
    #[cfg(feature = "micro-profiling")]
    let allocations = vize_l0::profiler::allocation_snapshot();
    profiler.disable();
    assert_eq!(checksum, (expected_bindings * iterations) as u64);
    let report = serde_json::json!({
        "scope": "repeated source parse+analysis only; generation and report outside timed/allocation windows",
        "iterations": iterations, "minimumSourceBytes": minimum_bytes, "sourceBytes": source.len(),
        "sourceHash": source_hash, "rows": rows, "scenario": scenario,
        "elapsedNs": elapsed_ns, "checksum": checksum, "instrumented": instrument,
    });
    #[cfg(feature = "micro-profiling")]
    let report = {
        let mut report = report;
        if instrument {
            report["micro"] = serde_json::to_value(vize_l0::profiler::micro::snapshot())
                .expect("counts serialize");
            report["allocation"] = serde_json::json!({
                "calls": allocations.allocation_calls(),
                "requestedBytes": allocations.requested_bytes(),
            });
        }
        report
    };
    println!("{report}");
}
