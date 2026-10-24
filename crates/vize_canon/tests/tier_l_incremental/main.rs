#![expect(clippy::expect_used, reason = "tests assert by panicking")]
#![expect(clippy::panic, reason = "tests assert by panicking")]
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Deserialize;
use vize_canon::{
    BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait, CompilerConfigurationSnapshot,
};
use vize_carton::corsa_resolver::{CorsaResolveRequest, resolve_corsa_executable};
use vize_l0::String;

mod artifact;
mod budget;
mod failure;
mod fixture;
use artifact::{Artifact, BatchIncrementalBudget, FixtureEvidence, lane, write_artifact};
use budget::{
    assert_cold_metrics, assert_no_injected_diagnostics, assert_warm_metrics, assert_within_budget,
    budget_scale,
};
use fixture::{env_path, git_revision};

const FIXTURE_ID: &str = "vue-vben-admin";
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

// This is a separate pre-cold disk observation, not an export of private consumed stamps.
fn assert_configuration_readiness(repo_root: &Path, fixture_root: &Path) {
    let config = fixture_root.join("playground/tsconfig.json");
    let attempt = CompilerConfigurationSnapshot::capture_attempt(fixture_root, Some(&config));
    let observations = attempt.observations.paths();
    let disk = observations.iter().map(|path| {
        let metadata = fs::symlink_metadata(path);
        let state = match metadata {
            Ok(meta) if meta.is_symlink() => serde_json::json!({
                "kind": "symlink", "target": fs::read_link(path).map_err(|error| error.to_string()),
                "freshReadBytes": fs::read(path).map_err(|error| error.to_string()),
            }),
            Ok(meta) if meta.is_file() => serde_json::json!({
                "kind": "file", "freshReadBytes": fs::read(path).map_err(|error| error.to_string()),
            }),
            Ok(meta) => serde_json::json!({"kind": if meta.is_dir() { "directory" } else { "other" }}),
            Err(error) => serde_json::json!({"kind": "metadata-error", "errorKind": vize_l0::cstr!("{:?}", error.kind()), "error": error.to_string()}),
        };
        serde_json::json!({"path": path, "state": state})
    }).collect::<Vec<_>>();
    let changed = attempt.observations.changed_paths();
    let output = artifact::output_dir(repo_root);
    fs::create_dir_all(&output).expect("readiness evidence directory should exist");
    fs::write(output.join("compiler-readiness.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "schemaVersion": 1, "config": config, "fixtureRoot": fixture_root,
        "qualification": "public candidate paths and separate fresh disk reads before cold timer; not consumed byte stamps",
        "privateOriginalStampsExported": false,
        "dependencies": attempt.dependencies, "observationPaths": observations,
        "freshDiskObservations": disk, "changedPaths": changed,
        "outcome": attempt.outcome.as_ref().map(|_| "ready").map_err(|error| error.to_string()),
    })).expect("readiness evidence should serialize")).expect("readiness evidence should write");
    attempt
        .outcome
        .expect("authored Tier-L compiler configuration must resolve before cold scan");
    assert!(
        changed.is_empty(),
        "Tier-L compiler configuration changed during readiness capture"
    );
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
    let corsa_path = env_path("VIZE_TIER_L_CORSA_BIN", &repo_root).unwrap_or_else(|| {
        resolve_corsa_executable(CorsaResolveRequest {
            explicit_path: None,
            project_root: Some(&repo_root),
        })
        .expect("Corsa binary should resolve for Tier-L")
    });
    assert!(
        corsa_path.exists(),
        "Corsa binary is missing: {}",
        corsa_path.display()
    );

    assert_configuration_readiness(&repo_root, &fixture_root);

    let injected_path = fixture_root.join(INJECTED_FILE);
    let injected = InjectedFixtureFile::create(injected_path.clone());
    let options = BatchTypeCheckerOptions {
        tsconfig_path: Some(fixture_root.join("playground/tsconfig.json")),
        ..Default::default()
    };
    let mut checker =
        BatchTypeChecker::with_options_and_corsa_path(&fixture_root, options, Some(&corsa_path))
            .expect("Tier-L checker should start");

    let mut failure = failure::FailureReceipt::new(
        artifact::output_dir(&repo_root),
        FIXTURE_ID,
        project.revision.clone(),
        INJECTED_FILE,
        budget.clone(),
        budget_scale,
    );
    failure.begin(0);
    let source_revision = git_revision(&repo_root);
    let cold_started = Instant::now();
    let vue_paths = collect_vue_paths(&fixture_root);
    checker
        .scan_paths(&vue_paths)
        .expect("Tier-L scan should succeed");
    failure.scanned_inputs(
        &fixture_root,
        &source_revision,
        &vue_paths,
        checker.virtual_files().into_iter().map(|file| {
            (
                file.original_path.as_path(),
                file.virtual_path.as_path(),
                file.content.as_str(),
            )
        }),
    );
    assert_eq!(
        checker
            .virtual_files()
            .iter()
            .filter(|file| file
                .original_path
                .extension()
                .is_some_and(|extension| extension == "vue"))
            .count(),
        TIER_L_VUE_FILES,
        "fixture must remain Tier-L scale"
    );
    // The 500 Vue roots import 181 TypeScript sources. They now participate in
    // the canonical graph instead of being bypassed through filesystem reads.
    assert_eq!(checker.file_count(), budget.max_requested_files);
    for path in &vue_paths {
        assert!(
            checker
                .virtual_files()
                .iter()
                .any(|file| &file.original_path == path)
        );
    }
    let cold = checker
        .check_incremental(std::slice::from_ref(&injected_path))
        .expect("cold incremental session should complete");
    let cold_ms = cold_started.elapsed().as_millis();
    let cold_metrics = checker.incremental_metrics();
    failure.complete(0, cold_ms, cold_metrics, checker.file_count());
    assert_no_injected_diagnostics(&cold);
    assert_cold_metrics(cold_metrics, &budget);
    assert_within_budget("cold", cold_ms, budget.cold_ms, budget_scale);

    failure.begin(1);
    injected.write(BROKEN_SOURCE);
    let broken_started = Instant::now();
    let broken = checker
        .check_incremental(std::slice::from_ref(&injected_path))
        .expect("broken warm check should complete");
    let broken_ms = broken_started.elapsed().as_millis();
    let broken_metrics = checker.incremental_metrics();
    failure.complete(1, broken_ms, broken_metrics, checker.file_count());
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
    assert_warm_metrics(broken_metrics, 2, 1, &budget);
    assert_within_budget("broken warm", broken_ms, budget.warm_ms, budget_scale);

    failure.begin(2);
    injected.write(CLEAN_SOURCE);
    let repaired_started = Instant::now();
    let repaired = checker
        .check_incremental(std::slice::from_ref(&injected_path))
        .expect("repaired warm check should complete");
    let repaired_ms = repaired_started.elapsed().as_millis();
    let repaired_metrics = checker.incremental_metrics();
    failure.complete(2, repaired_ms, repaired_metrics, checker.file_count());
    assert_no_injected_diagnostics(&repaired);
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
    failure.disarm();
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
