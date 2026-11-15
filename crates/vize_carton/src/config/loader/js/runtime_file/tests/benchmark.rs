use std::{env, fs, hint::black_box, path::PathBuf, time::Instant};

const BENCHMARK_SCRIPT: &str = concat!(
    include_str!("../../../vite-runtime.mjs"),
    include_str!("../../../module-runtime.mjs"),
    include_str!("../../../config-inputs.mjs")
);

#[test]
#[ignore = "production storage benchmark invoked explicitly by Actions"]
fn runtime_storage_benchmark() {
    let lane = env::var("VIZE_RUNTIME_BENCH_LANE").unwrap();
    assert!(matches!(lane.as_str(), "base" | "head"));
    let samples = env::var("VIZE_RUNTIME_BENCH_SAMPLES")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let iterations = env::var("VIZE_RUNTIME_BENCH_ITERATIONS")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    assert!(samples > 0 && iterations > 0);
    let output = PathBuf::from(env::var_os("VIZE_RUNTIME_BENCH_OUTPUT").unwrap());
    let owner = tempfile::tempdir().unwrap();
    let source = super::super::runtime_source(BENCHMARK_SCRIPT);
    let mut timings = Vec::with_capacity(samples * iterations);

    for sample in 0..samples {
        let cache = owner.path().join(crate::cstr!("sample-{sample}").as_str());
        for _ in 0..iterations {
            let started = Instant::now();
            let runtime = if lane == "base" {
                super::super::temporary_runtime(&source).unwrap()
            } else {
                super::super::process_runtime(&cache, &source).unwrap()
            };
            let actual = fs::read(runtime.path()).unwrap();
            assert_eq!(actual, source);
            drop(black_box(actual));
            drop(runtime);
            timings.push(started.elapsed().as_secs_f64() * 1_000.0);
        }
    }

    timings.sort_by(f64::total_cmp);
    let invocations = samples * iterations;
    let percentile = |fraction: f64| timings[(timings.len() as f64 * fraction).ceil() as usize - 1];
    let cached = lane == "head";
    let source_writes = if cached { samples } else { invocations };
    let retained_files = if cached { samples * 3 } else { 0 };
    let report = serde_json::json!({
        "schema": 1,
        "implementation": "production-rust",
        "lane": lane,
        "samples": samples,
        "iterations": iterations,
        "invocations": invocations,
        "payloadBytes": source.len(),
        "sourceSha256": super::super::sha256_hex(&source),
        "wallMs": {
            "p50": percentile(0.5),
            "p95": percentile(0.95),
            "total": timings.iter().sum::<f64>(),
        },
        "storage": {
            "regularFilesCreated": if cached { samples * 3 } else { invocations },
            "sourceWrites": source_writes,
            "writtenBytes": source_writes * source.len(),
            "retainedFiles": retained_files,
            "retainedBytes": if cached { samples * source.len() } else { 0 },
            "perIsolatedSample": {
                "regularFilesCreated": if cached { 3 } else { iterations },
                "sourceWrites": if cached { 1 } else { iterations },
                "writtenBytes": if cached { source.len() } else { iterations * source.len() },
                "retainedFiles": if cached { 3 } else { 0 },
                "retainedBytes": if cached { source.len() } else { 0 },
            },
        },
    });
    fs::write(output, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    println!("{report}");
}
