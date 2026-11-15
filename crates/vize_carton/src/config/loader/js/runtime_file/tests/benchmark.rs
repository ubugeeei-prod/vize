use std::{env, fs, hint::black_box, path::Path, path::PathBuf, time::Instant};

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
    let round = env::var("VIZE_RUNTIME_BENCH_ROUND").unwrap();
    let lane_order = env::var("VIZE_RUNTIME_BENCH_ORDER").unwrap();
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
    let mut regular_files_created = 0;
    let mut source_writes = 0;
    let mut written_bytes = 0;

    for sample in 0..samples {
        let cache = owner.path().join(crate::cstr!("sample-{sample}").as_str());
        for _ in 0..iterations {
            let expected = super::super::cache_root(&cache)
                .join(crate::cstr!("{}.mjs", super::super::sha256_hex(&source)).as_str());
            let source_existed = is_regular_file(&expected);
            let started = Instant::now();
            let runtime = if lane == "base" {
                super::super::temporary_runtime(&source).unwrap()
            } else {
                super::super::process_runtime(&cache, &source).unwrap()
            };
            let actual = fs::read(runtime.path()).unwrap();
            assert_eq!(actual, source);
            let elapsed = started.elapsed().as_secs_f64() * 1_000.0;
            if lane == "base" {
                assert!(is_regular_file(runtime.path()));
                regular_files_created += 1;
            } else if !source_existed {
                let root = super::super::cache_root(&cache);
                assert!(is_regular_file(&root.join(super::super::CATALOG_LEASE)));
                assert!(is_regular_file(&expected));
                assert!(is_regular_file(&expected.with_extension("lease")));
                regular_files_created += 3;
            }
            if lane == "base" || !source_existed {
                source_writes += 1;
                written_bytes += actual.len();
            }
            drop(black_box(actual));
            drop(runtime);
            timings.push(elapsed);
        }
    }

    let raw_timings = timings.clone();
    timings.sort_by(f64::total_cmp);
    let invocations = samples * iterations;
    let percentile = |fraction: f64| timings[(timings.len() as f64 * fraction).ceil() as usize - 1];
    let cached = lane == "head";
    let (retained_files, retained_bytes) = regular_file_stats(owner.path());
    let expected_source_writes = if cached { samples } else { invocations };
    let expected_regular_files = if cached { samples * 3 } else { invocations };
    assert_eq!(source_writes, expected_source_writes);
    assert_eq!(regular_files_created, expected_regular_files);
    assert_eq!(written_bytes, expected_source_writes * source.len());
    assert_eq!(retained_files, if cached { samples * 3 } else { 0 });
    assert_eq!(
        retained_bytes,
        if cached { samples * source.len() } else { 0 }
    );
    let report = serde_json::json!({
        "schema": 1,
        "implementation": "production-rust",
        "lane": lane,
        "round": round,
        "laneOrder": lane_order,
        "samples": samples,
        "iterations": iterations,
        "invocations": invocations,
        "payloadBytes": source.len(),
        "sourceSha256": super::super::sha256_hex(&source),
        "wallMs": {
            "p50": percentile(0.5),
            "p95": percentile(0.95),
            "total": timings.iter().sum::<f64>(),
            "raw": raw_timings,
        },
        "storage": {
            "regularFilesCreated": regular_files_created,
            "sourceWrites": source_writes,
            "writtenBytes": written_bytes,
            "retainedFiles": retained_files,
            "retainedBytes": retained_bytes,
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

fn regular_file_stats(root: &Path) -> (usize, usize) {
    let Ok(entries) = fs::read_dir(root) else {
        return (0, 0);
    };
    entries.flatten().fold((0, 0), |total, entry| {
        let path = entry.path();
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            return total;
        };
        let observed = if metadata.is_dir() && !metadata.file_type().is_symlink() {
            regular_file_stats(&path)
        } else if metadata.is_file() && !metadata.file_type().is_symlink() {
            (1, metadata.len() as usize)
        } else {
            (0, 0)
        };
        (total.0 + observed.0, total.1 + observed.1)
    })
}

fn is_regular_file(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
}
