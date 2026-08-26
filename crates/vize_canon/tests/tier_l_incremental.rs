use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use serde::Deserialize;
use vize_canon::{
    BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait, IncrementalCheckMetrics,
};
use vize_s0::{String, ToCompactString};

#[path = "support/tier_l_incremental_artifact.rs"]
mod artifact;
use artifact::{Artifact, BatchIncrementalBudget, FixtureEvidence, lane, write_artifact};

const FIXTURE_ID: &str = "vue-vben-admin";
const BUDGET_SCALE_ENV: &str = "VIZE_TIER_L_BUDGET_SCALE";
const TIER_L_VUE_FILES: usize = 500;
const INJECTED_FILE: &str = "apps/web-antd/src/__vize_batch_incremental_oracle__.vue";
const CLEAN_SOURCE: &str = r#"<script setup lang="ts">
const __vizeBatchIncrementalOracle: number = 1;
</script>

<template><span>{{ __vizeBatchIncrementalOracle }}</span></template>
"#;
const BROKEN_SOURCE: &str = r#"<script setup lang="ts">
const __vizeBatchIncrementalOracle: number = 'broken';
</script>

<template><span>{{ __vizeBatchIncrementalOracle }}</span></template>
"#;

#[derive(Deserialize)]
struct Registry {
    projects: Vec<RegistryProject>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegistryProject {
    id: String,
    revision: String,
    batch_incremental_budget: Option<BatchIncrementalBudget>,
}

struct InjectedFixtureFile(PathBuf);

impl InjectedFixtureFile {
    fn create(path: PathBuf) -> Self {
        assert!(!path.exists(), "injected fixture path must start absent");
        fs::write(&path, CLEAN_SOURCE).expect("clean fixture source should write");
        Self(path)
    }

    fn write(&self, source: &str) {
        fs::write(&self.0, source).expect("fixture patch should write");
    }
}

impl Drop for InjectedFixtureFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[test]
#[ignore = "runs the pinned Tier-L fixture in the per-PR Vue parity lane"]
fn vben_batch_incremental_session_reuses_exact_materialized_delta() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let registry_path = repo_root.join("tests/_fixtures/vue-ecosystem-fixtures.json");
    let registry: Registry = serde_json::from_slice(
        &fs::read(registry_path).expect("fixture registry should be readable"),
    )
    .expect("fixture registry should parse");
    let project = registry
        .projects
        .into_iter()
        .find(|project| project.id == FIXTURE_ID)
        .expect("Vben fixture must be registered");
    let budget = project
        .batch_incremental_budget
        .expect("Vben must own a batchIncrementalBudget");
    assert_budget(&budget);
    let budget_scale = budget_scale();

    let fixture_root = env_path("VIZE_TIER_L_FIXTURE", &repo_root)
        .unwrap_or_else(|| repo_root.join("tests/_fixtures/_git/vue-vben-admin"))
        .canonicalize()
        .expect("Tier-L fixture path should canonicalize");
    assert_eq!(
        git_revision(&fixture_root),
        project.revision,
        "fixture revision drift"
    );
    let corsa_path = env_path("VIZE_TIER_L_CORSA_BIN", &repo_root)
        .unwrap_or_else(|| repo_root.join("node_modules/.bin/tsgo"));
    assert!(
        corsa_path.exists(),
        "Corsa binary is missing: {}",
        corsa_path.display()
    );

    let injected_path = fixture_root.join(INJECTED_FILE);
    let injected = InjectedFixtureFile::create(injected_path.clone());
    let options = BatchTypeCheckerOptions {
        tsconfig_path: Some(fixture_root.join("playground/tsconfig.json")),
        ..Default::default()
    };
    let mut checker =
        BatchTypeChecker::with_options_and_corsa_path(&fixture_root, options, Some(&corsa_path))
            .expect("Tier-L checker should start");

    let cold_started = Instant::now();
    let vue_paths = collect_vue_paths(&fixture_root);
    checker
        .scan_paths(&vue_paths)
        .expect("Tier-L scan should succeed");
    assert!(
        checker.file_count() == TIER_L_VUE_FILES,
        "fixture must remain Tier-L scale"
    );
    let cold = checker
        .check_incremental(std::slice::from_ref(&injected_path))
        .expect("cold incremental session should complete");
    let cold_ms = cold_started.elapsed().as_millis();
    assert_no_injected_diagnostics(&cold);
    let cold_metrics = checker.incremental_metrics();
    assert_cold_metrics(cold_metrics, &budget);
    assert_within_budget("cold", cold_ms, budget.cold_ms, budget_scale);

    injected.write(BROKEN_SOURCE);
    let broken_started = Instant::now();
    let broken = checker
        .check_incremental(std::slice::from_ref(&injected_path))
        .expect("broken warm check should complete");
    let broken_ms = broken_started.elapsed().as_millis();
    let injected_errors = broken
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.file.file_name().and_then(|name| name.to_str())
                == Some("__vize_batch_incremental_oracle__.vue")
                && diagnostic.code == Some(2322)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        injected_errors.len(),
        1,
        "broken patch must report exactly one injected TS2322"
    );
    assert_eq!((injected_errors[0].line, injected_errors[0].column), (1, 6));
    assert!(
        injected_errors[0]
            .message
            .contains("not assignable to type 'number'"),
        "unexpected injected TS2322: {}",
        injected_errors[0].message
    );
    let broken_metrics = checker.incremental_metrics();
    assert_warm_metrics(broken_metrics, 2, 1, &budget);
    assert_within_budget("broken warm", broken_ms, budget.warm_ms, budget_scale);

    injected.write(CLEAN_SOURCE);
    let repaired_started = Instant::now();
    let repaired = checker
        .check_incremental(std::slice::from_ref(&injected_path))
        .expect("repaired warm check should complete");
    let repaired_ms = repaired_started.elapsed().as_millis();
    assert_no_injected_diagnostics(&repaired);
    let repaired_metrics = checker.incremental_metrics();
    assert_warm_metrics(repaired_metrics, 3, 2, &budget);
    assert_within_budget("repaired warm", repaired_ms, budget.warm_ms, budget_scale);

    let artifact = Artifact {
        schema_version: 1,
        fixture: FixtureEvidence {
            id: FIXTURE_ID,
            revision: project.revision,
            injected_file: INJECTED_FILE,
        },
        budget,
        budget_scale,
        file_count: checker.file_count(),
        lanes: vec![
            lane("cold", cold_ms, cold_metrics),
            lane("brokenWarm", broken_ms, broken_metrics),
            lane("repairedWarm", repaired_ms, repaired_metrics),
        ],
    };
    write_artifact(&repo_root, &artifact);
}

fn env_path(name: &str, repo_root: &Path) -> Option<PathBuf> {
    std::env::var_os(name).map(|value| {
        let path = PathBuf::from(value);
        if path.is_absolute() {
            path
        } else {
            repo_root.join(path)
        }
    })
}

fn git_revision(root: &Path) -> String {
    let output = Command::new("git")
        .args([
            "-C",
            root.to_str().expect("UTF-8 fixture path"),
            "rev-parse",
            "HEAD",
        ])
        .output()
        .expect("git should inspect the fixture");
    assert!(output.status.success(), "fixture revision lookup failed");
    std::str::from_utf8(&output.stdout)
        .expect("revision should be UTF-8")
        .trim()
        .to_compact_string()
}

fn collect_vue_paths(fixture_root: &Path) -> Vec<PathBuf> {
    let mut paths = ["apps", "packages", "playground"]
        .into_iter()
        .flat_map(|relative| walkdir::WalkDir::new(fixture_root.join(relative)))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("vue"))
        .collect::<Vec<_>>();
    paths.sort();
    assert!(
        paths.len() > TIER_L_VUE_FILES,
        "pinned fixture fell below Tier-L scale"
    );
    paths.truncate(TIER_L_VUE_FILES);
    assert!(paths.iter().any(|path| path.ends_with(INJECTED_FILE)));
    paths
}

fn assert_budget(budget: &BatchIncrementalBudget) {
    assert!((1..=300_000).contains(&budget.cold_ms));
    assert!((1..=budget.cold_ms).contains(&budget.warm_ms));
    assert!((1..=10_000).contains(&budget.max_requested_files));
    assert_eq!(
        budget.max_changed_files, 1,
        "one edited SFC must remain the delta budget"
    );
}

fn budget_scale() -> f64 {
    match std::env::var(BUDGET_SCALE_ENV) {
        Ok(raw) if raw.is_empty() => 1.0,
        Ok(raw) => {
            let scale = raw.parse::<f64>().unwrap_or_else(|_| {
                panic!(
                    "{BUDGET_SCALE_ENV} must be a finite number greater than 0 and at most 100, got {raw:?}"
                )
            });
            assert!(
                scale.is_finite() && scale > 0.0 && scale <= 100.0,
                "{BUDGET_SCALE_ENV} must be a finite number greater than 0 and at most 100, got {raw:?}"
            );
            scale
        }
        Err(std::env::VarError::NotPresent) => 1.0,
        Err(error) => panic!("{BUDGET_SCALE_ENV} must be valid UTF-8: {error}"),
    }
}

fn assert_no_injected_diagnostics(result: &vize_canon::BatchTypeCheckResult) {
    assert!(
        result.diagnostics.iter().all(|diagnostic| {
            diagnostic.file.file_name().and_then(|name| name.to_str())
                != Some("__vize_batch_incremental_oracle__.vue")
        }),
        "clean source retained injected-file diagnostics"
    );
}

fn assert_cold_metrics(metrics: IncrementalCheckMetrics, budget: &BatchIncrementalBudget) {
    assert_eq!((metrics.checks, metrics.session_starts), (1, 1));
    assert_eq!((metrics.session_reuses, metrics.session_refreshes), (0, 0));
    assert_eq!(metrics.session_to_cli_fallbacks, 0);
    assert!(metrics.last_session_started);
    assert!(!metrics.last_session_to_cli_fallback);
    assert_eq!(
        (
            metrics.last_changed_files,
            metrics.last_created_files,
            metrics.last_deleted_files
        ),
        (0, 0, 0)
    );
    assert_requested_budget(metrics, budget);
}

fn assert_warm_metrics(
    metrics: IncrementalCheckMetrics,
    checks: usize,
    reuses: usize,
    budget: &BatchIncrementalBudget,
) {
    assert_eq!((metrics.checks, metrics.session_starts), (checks, 1));
    assert_eq!(
        (metrics.session_reuses, metrics.session_refreshes),
        (reuses, reuses)
    );
    assert_eq!(metrics.session_to_cli_fallbacks, 0);
    assert!(metrics.last_session_reused && metrics.last_session_refreshed);
    assert!(!metrics.last_session_to_cli_fallback);
    assert_eq!(metrics.last_changed_files, budget.max_changed_files);
    assert_eq!(
        (metrics.last_created_files, metrics.last_deleted_files),
        (0, 0)
    );
    assert_requested_budget(metrics, budget);
}

fn assert_requested_budget(metrics: IncrementalCheckMetrics, budget: &BatchIncrementalBudget) {
    assert!(metrics.last_requested_files > 0);
    assert_eq!(
        metrics.last_requested_files, budget.max_requested_files,
        "the pinned Tier-L corpus must request every registered Vue file"
    );
    assert!(
        metrics.last_requested_files <= budget.max_requested_files,
        "requested {} files, budget is {}",
        metrics.last_requested_files,
        budget.max_requested_files
    );
}

fn assert_within_budget(lane: &str, elapsed_ms: u128, budget_ms: u64, budget_scale: f64) {
    let scaled_budget_ms = (budget_ms as f64 * budget_scale).ceil() as u128;
    assert!(
        elapsed_ms < scaled_budget_ms,
        "{lane} took {elapsed_ms}ms, budget is {budget_ms}ms at scale {budget_scale} ({scaled_budget_ms}ms)"
    );
}
