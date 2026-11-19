//! Repeated source microbenchmark; this is not a cold whole-project measurement.
#![expect(clippy::expect_used, reason = "a benchmark control fails by panicking")]

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
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(100);
    let rows = args
        .get(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(256);
    let instrument = args.get(2).is_some_and(|s| s == "profile");
    let valid = args.get(3).is_some_and(|s| s == "valid");
    assert!(
        !instrument || cfg!(feature = "micro-profiling"),
        "profile mode requires instrumentation"
    );
    let mut source = vize_l0::String::default();
    for row in 0..rows {
        use std::fmt::Write;
        writeln!(source, "const item_{row}: number = {row};").expect("source builds");
    }
    source.push_str(if valid {
        "const damaged = 0;\n"
    } else {
        "const damaged = missing.;\n"
    });
    let source_hash = vize_l0::hash::hash_bytes(source.as_bytes());
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
    assert_eq!(checksum, ((rows + 1) * iterations) as u64);
    let report = serde_json::json!({
        "scope":"repeated source parse+semantic-analysis microbenchmark; no backend or daemon",
        "iterations":iterations,"rows":rows,"sourceBytes":source.len(),"sourceHash":source_hash,
        "elapsedNs":elapsed_ns,"checksum":checksum,"instrumented":instrument,
        "scenario":if valid {"valid"} else {"incomplete-member"},
    });
    #[cfg(feature = "micro-profiling")]
    let report = {
        let mut report = report;
        if instrument {
            report["micro"] = serde_json::to_value(vize_l0::profiler::micro::snapshot())
                .expect("counts serialize");
            report["allocation"] = serde_json::json!({
                "calls":allocations.allocation_calls(),"requestedBytes":allocations.requested_bytes(),
            });
        }
        report
    };
    println!("{report}");
}
