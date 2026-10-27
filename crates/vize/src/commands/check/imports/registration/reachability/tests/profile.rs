use std::io::Write;
use std::path::{Path, PathBuf};

use super::super::resolve_local;
use super::{CanonicalPathCache, ImportFileOptions, PathAliasResolver};
use vize_l0::String;
use vize_l0::profiler::{global_profiler, micro};

struct Case {
    specifier: String,
    expected: Option<PathBuf>,
}

fn execute<const RECORD_INPUTS: bool>(
    importer: &Path,
    cases: &[Case],
    aliases: &PathAliasResolver,
    iterations: usize,
    canonical: &mut CanonicalPathCache,
) -> u64 {
    let mut successes = 0;
    for _ in 0..iterations {
        for case in cases {
            let answer = resolve_local::<RECORD_INPUTS>(
                importer,
                &case.specifier,
                canonical,
                ImportFileOptions::from(false),
                Some(aliases),
            );
            assert_eq!(answer.0.as_deref(), case.expected.as_deref());
            successes += u64::from(answer.0.is_some());
            drop(std::hint::black_box(answer));
        }
    }
    successes
}

fn measured<const RECORD_INPUTS: bool>(
    group: &str,
    importer: &Path,
    cases: &[Case],
    aliases: &PathAliasResolver,
    iterations: usize,
    canonical: &mut CanonicalPathCache,
) -> u64 {
    match group {
        "relative" => vize_l0::micro_span!(
            "cli.probe.carrier.relative",
            execute::<RECORD_INPUTS>(importer, cases, aliases, iterations, canonical)
        ),
        "absolute" => vize_l0::micro_span!(
            "cli.probe.carrier.absolute",
            execute::<RECORD_INPUTS>(importer, cases, aliases, iterations, canonical)
        ),
        "alias" => vize_l0::micro_span!(
            "cli.probe.carrier.alias",
            execute::<RECORD_INPUTS>(importer, cases, aliases, iterations, canonical)
        ),
        _ => panic!("unknown prepared group"),
    }
}

#[test]
#[ignore = "isolated manual source profiling carrier; requires the same prepared physical fixture"]
fn profile_local_probe_histories() {
    let role = std::env::var("VIZE_PROBE_PROFILE_ROLE").unwrap();
    assert!(role == "legacy" || role == "candidate");
    let root = PathBuf::from(std::env::var_os("VIZE_PROBE_FIXTURE").unwrap())
        .canonicalize()
        .unwrap();
    let output = PathBuf::from(std::env::var_os("VIZE_PROBE_PROFILE_OUTPUT").unwrap());
    assert!(!output.exists());
    let iterations: usize = std::env::var("VIZE_PROBE_ITERATIONS")
        .unwrap()
        .parse()
        .unwrap();
    assert!(iterations > 0 && iterations <= 10_000);
    let importer = root.join("entry.ts");
    let aliases = PathAliasResolver::from_tsconfig(Some(&root.join("tsconfig.json")));
    let make = |specifier: &str, expected: Option<&str>| Case {
        specifier: specifier.into(),
        expected: expected.map(|path| root.join(path)),
    };
    let relative = [
        make("./model.js", Some("model.ts")),
        make("./esm.mjs", Some("esm.mts")),
        make("./common.cjs", Some("common.cts")),
        make("./folder", Some("folder/index.ts")),
        make("./Widget.vue", Some("Widget.vue")),
        make("./日本😀.vue", Some("日本😀.vue")),
        make("./absent", None),
    ];
    let absolute = [
        Case {
            specifier: root.join("model.ts").to_str().unwrap().into(),
            expected: Some(root.join("model.ts")),
        },
        Case {
            specifier: root.join("absolute-missing").to_str().unwrap().into(),
            expected: None,
        },
    ];
    let alias = [
        make("@/component", Some("real/component.vue")),
        make("@/special", Some("specific.ts")),
        make("base", Some("base.ts")),
        make("@/absent", None),
    ];
    let profiler = global_profiler();
    let mut windows = Vec::new();
    for (group, cases) in [
        ("relative", relative.as_slice()),
        ("absolute", absolute.as_slice()),
        ("alias", alias.as_slice()),
    ] {
        // Both roles pre-register every possible site and profiler map before
        // the cut. Each measured execute starts with a fresh canonical cache.
        profiler.clear();
        let mut legacy_warmup = CanonicalPathCache::default();
        let mut candidate_warmup = CanonicalPathCache::default();
        let mut canonical = CanonicalPathCache::default();
        profiler.enable();
        measured::<true>(group, &importer, cases, &aliases, 1, &mut legacy_warmup);
        measured::<false>(group, &importer, cases, &aliases, 1, &mut candidate_warmup);
        profiler.disable();
        let registered = micro::snapshot();
        profiler.clear();
        profiler.enable();
        let successes = if role == "legacy" {
            measured::<true>(
                group,
                &importer,
                cases,
                &aliases,
                iterations,
                &mut canonical,
            )
        } else {
            measured::<false>(
                group,
                &importer,
                cases,
                &aliases,
                iterations,
                &mut canonical,
            )
        };
        profiler.disable();
        let entries = micro::snapshot();
        let stats = entries
            .iter()
            .filter_map(|entry| {
                profiler.get(entry.key).map(|metrics| {
                    serde_json::json!({
                        "key": entry.key, "count": metrics.count,
                        "elapsedNs": metrics.total_duration.as_nanos(),
                        "selfElapsedNs": metrics.self_duration.as_nanos(),
                        "requestedAllocationCalls": metrics.alloc_calls,
                        "requestedAllocationBytes": metrics.alloc_bytes,
                        "selfRequestedAllocationCalls": metrics.self_alloc_calls,
                        "selfRequestedAllocationBytes": metrics.self_alloc_bytes,
                    })
                })
            })
            .collect::<Vec<_>>();
        windows.push(serde_json::json!({
            "group": group, "iterations": iterations, "cases": cases.len(),
            "successes": successes, "registeredDuringWarmup": registered,
            "sourceCounters": entries, "sourceSpans": stats,
            "scope": "selected local resolver callbacks; synchronous Rust thread; no backend",
            "historyOwnership": if group == "alias" { "retained AliasProbe proof" } else { "discarded CLI source-interner history" },
            "peakLiveBytes": null, "processTreeRss": null, "hardwareInstructions": null,
        }));
    }
    let receipt = serde_json::json!({
        "schema": "vize.cli-probe-history.micro", "version": 1,
        "role": role, "fixtureRootPhysical": root,
        "compiledSourceRevision": option_env!("VIZE_PROBE_PROFILE_SOURCE_SHA"),
        "windows": windows,
        "normalMachineBinaryComparison": false,
        "normalPerformanceQualified": false, "coldQualified": false,
        "methodology": "same profiling image; legacy consttrue is source-body equivalent, not the old machine binary; counter instrumentation can prevent DCE or sinking",
    });
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .unwrap();
    file.write_all(serde_json::to_vec_pretty(&receipt).unwrap().as_slice())
        .unwrap();
}
