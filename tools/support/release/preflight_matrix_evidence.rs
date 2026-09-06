use regex::Regex;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::Path,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

pub const REQUIRED_REAL_PROJECT_MATRIX_SHARD_COUNT: usize = 22;
pub const REAL_PROJECT_MATRIX_WORKFLOW_NAME: &str = "Real Project Matrix";

const ARTIFACT_DOWNLOAD_TIMEOUT_SECONDS: u64 = 120;
const ARTIFACT_MAX_BYTES: u64 = 512 * 1024 * 1024;
const ARTIFACT_MAX_ENTRIES: usize = 4_096;
const ARTIFACT_MAX_UNCOMPRESSED_BYTES: u64 = 512 * 1024 * 1024;

static SCRATCH_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn assert_real_project_matrix_release_artifacts<F>(
    repo_root: &Path,
    run: &Value,
    artifacts: &[Value],
    mut read_artifact_entries: F,
) -> Result<(), String>
where
    F: FnMut(&Value) -> Result<BTreeMap<String, String>, String>,
{
    let expected_typecheck_projects = typecheck_performance_project_ids(repo_root)?;
    let mut observed_typecheck_projects = BTreeMap::new();
    for shard in 0..REQUIRED_REAL_PROJECT_MATRIX_SHARD_COUNT {
        let artifact_name = format!("real-project-matrix-{shard}");
        let matches = artifacts
            .iter()
            .filter(|artifact| artifact.get("name").and_then(Value::as_str) == Some(&artifact_name))
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(format!(
                "Real Project Matrix release evidence must contain exactly one {artifact_name} artifact; found {}",
                matches.len()
            ));
        }
        let artifact = matches[0];
        assert_artifact_bound_to_run(run, artifact)?;
        let entries = read_artifact_entries(artifact)?;
        assert_real_project_shard_artifact(
            run,
            &artifact_name,
            shard,
            &entries,
            &expected_typecheck_projects,
            &mut observed_typecheck_projects,
        )?;
    }
    assert_release_typecheck_coverage(&expected_typecheck_projects, &observed_typecheck_projects)
}

pub fn download_artifact_entries(
    token: &str,
    artifact: &Value,
) -> Result<BTreeMap<String, String>, String> {
    let artifact_name = string_field(artifact, "name").unwrap_or("unknown");
    let url = string_field(artifact, "archive_download_url").ok_or_else(|| {
        format!("Real Project Matrix artifact {artifact_name} has no download URL")
    })?;
    let scratch = env::temp_dir().join(format!(
        "vize-release-matrix-artifact-{}-{}",
        std::process::id(),
        SCRATCH_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let result = download_artifact_entries_in(token, artifact_name, url, &scratch);
    let _ = fs::remove_dir_all(&scratch);
    result
}

fn download_artifact_entries_in(
    token: &str,
    artifact_name: &str,
    url: &str,
    scratch: &Path,
) -> Result<BTreeMap<String, String>, String> {
    fs::create_dir_all(scratch)
        .map_err(|error| format!("cannot create {}: {error}", scratch.display()))?;
    let archive = scratch.join("artifact.zip");
    let output = scratch.join("out");
    let timeout_seconds = ARTIFACT_DOWNLOAD_TIMEOUT_SECONDS.to_string();
    let download = Command::new("curl")
        .args([
            "--fail-with-body",
            "--silent",
            "--show-error",
            "--location",
            "--max-time",
            &timeout_seconds,
            "--header",
            "Accept: application/vnd.github+json",
            "--header",
            "X-GitHub-Api-Version: 2022-11-28",
            "--header",
            &format!("Authorization: Bearer {token}"),
            "--output",
        ])
        .arg(&archive)
        .arg(url)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run curl: {error}"))?;
    if !download.status.success() {
        let detail = String::from_utf8_lossy(&download.stderr).trim().to_string();
        return Err(format!(
            "Failed to download Real Project Matrix artifact {artifact_name}{}",
            if detail.is_empty() {
                String::new()
            } else {
                format!(":\n{detail}")
            }
        ));
    }
    let size = fs::metadata(&archive)
        .map_err(|error| {
            format!("cannot stat Real Project Matrix artifact {artifact_name}: {error}")
        })?
        .len();
    if size == 0 {
        return Err(format!(
            "Real Project Matrix artifact {artifact_name} downloaded no bytes"
        ));
    }
    if size > ARTIFACT_MAX_BYTES {
        return Err(format!(
            "Real Project Matrix artifact {artifact_name} exceeds {ARTIFACT_MAX_BYTES} bytes"
        ));
    }

    assert_archive_within_limits(&archive, artifact_name)?;
    let unzip = Command::new("unzip")
        .arg("-q")
        .arg(&archive)
        .arg("-d")
        .arg(&output)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run unzip: {error}"))?;
    if !unzip.status.success() {
        let detail = [unzip.stdout.as_slice(), unzip.stderr.as_slice()]
            .into_iter()
            .filter(|value| !value.is_empty())
            .map(|value| String::from_utf8_lossy(value).trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!(
            "Failed to unpack Real Project Matrix artifact {artifact_name}{}",
            if detail.is_empty() {
                String::new()
            } else {
                format!(":\n{detail}")
            }
        ));
    }

    let mut entries = BTreeMap::new();
    let mut total_bytes = 0_u64;
    collect_text_entries(&output, &output, &mut entries, &mut total_bytes)?;
    if entries.is_empty() {
        return Err("Real Project Matrix artifact is empty".to_string());
    }
    Ok(entries)
}

fn assert_archive_within_limits(archive: &Path, artifact_name: &str) -> Result<(), String> {
    let listing = Command::new("unzip")
        .arg("-Z")
        .arg("-t")
        .arg(archive)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run unzip: {error}"))?;
    if !listing.status.success() {
        let detail = [listing.stdout.as_slice(), listing.stderr.as_slice()]
            .into_iter()
            .filter(|value| !value.is_empty())
            .map(|value| String::from_utf8_lossy(value).trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!(
            "Failed to inspect Real Project Matrix artifact {artifact_name}{}",
            if detail.is_empty() {
                String::new()
            } else {
                format!(":\n{detail}")
            }
        ));
    }
    let stdout = String::from_utf8_lossy(&listing.stdout);
    let totals = Regex::new(r"(?m)^\s*(\d+)\s+files?,\s+(\d+)\s+bytes uncompressed")
        .unwrap()
        .captures(&stdout)
        .ok_or_else(|| {
            format!(
                "Real Project Matrix artifact {artifact_name} has an unreadable archive listing"
            )
        })?;
    let entry_count = totals[1]
        .parse::<usize>()
        .map_err(|error| format!("cannot parse artifact entry count: {error}"))?;
    let uncompressed_bytes = totals[2]
        .parse::<u64>()
        .map_err(|error| format!("cannot parse artifact uncompressed bytes: {error}"))?;
    if entry_count == 0 {
        return Err(format!(
            "Real Project Matrix artifact {artifact_name} is empty"
        ));
    }
    if entry_count > ARTIFACT_MAX_ENTRIES {
        return Err(format!(
            "Real Project Matrix artifact {artifact_name} declares {entry_count} entries; the limit is {ARTIFACT_MAX_ENTRIES}"
        ));
    }
    if uncompressed_bytes > ARTIFACT_MAX_UNCOMPRESSED_BYTES {
        return Err(format!(
            "Real Project Matrix artifact {artifact_name} declares {uncompressed_bytes} uncompressed bytes; the limit is {ARTIFACT_MAX_UNCOMPRESSED_BYTES}"
        ));
    }
    Ok(())
}

fn collect_text_entries(
    root: &Path,
    directory: &Path,
    entries: &mut BTreeMap<String, String>,
    total_bytes: &mut u64,
) -> Result<(), String> {
    let mut children = fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?;
    children.sort_by_key(|entry| entry.path());
    for child in children {
        let file_type = child
            .file_type()
            .map_err(|error| format!("cannot stat {}: {error}", child.path().display()))?;
        let absolute = child.path();
        if file_type.is_dir() {
            collect_text_entries(root, &absolute, entries, total_bytes)?;
        } else if file_type.is_file() {
            let payload = fs::read(&absolute)
                .map_err(|error| format!("cannot read {}: {error}", absolute.display()))?;
            if entries.len() + 1 > ARTIFACT_MAX_ENTRIES {
                return Err(format!(
                    "Real Project Matrix artifact extracted more than {ARTIFACT_MAX_ENTRIES} entries"
                ));
            }
            *total_bytes += payload.len() as u64;
            if *total_bytes > ARTIFACT_MAX_UNCOMPRESSED_BYTES {
                return Err(format!(
                    "Real Project Matrix artifact extracted more than {ARTIFACT_MAX_UNCOMPRESSED_BYTES} bytes"
                ));
            }
            let relative = absolute
                .strip_prefix(root)
                .map_err(|error| {
                    format!(
                        "cannot relativize {} from {}: {error}",
                        absolute.display(),
                        root.display()
                    )
                })?
                .to_string_lossy()
                .replace('\\', "/");
            entries.insert(relative, String::from_utf8_lossy(&payload).into_owned());
        } else {
            return Err(format!(
                "Real Project Matrix artifact contains unsupported entry: {}",
                absolute.display()
            ));
        }
    }
    Ok(())
}

fn assert_real_project_shard_artifact(
    run: &Value,
    artifact_name: &str,
    shard: usize,
    entries: &BTreeMap<String, String>,
    expected_typecheck_projects: &BTreeSet<String>,
    observed_typecheck_projects: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let run_head_sha = run_head_sha(run)?;
    let summary = read_json_entry(entries, "summary.json", artifact_name)?;
    if string_field(&summary, "schema") != Some("vize.fixtureToolMatrixReport")
        || integer_field(&summary, "version") != Some(3)
        || string_field(nested(&summary, &["evidence"]), "commitSha") != Some(run_head_sha)
        || integer_field(nested(&summary, &["command"]), "shardIndex") != Some(shard as i64)
        || integer_field(nested(&summary, &["command"]), "shardCount")
            != Some(REQUIRED_REAL_PROJECT_MATRIX_SHARD_COUNT as i64)
    {
        return Err(format!(
            "{artifact_name} summary is not exact release evidence for {run_head_sha}"
        ));
    }
    let selected_fixtures = read_text_entry(entries, "selected-fixtures.txt", artifact_name)?;
    if selected_fixtures
        .lines()
        .filter(|line| !line.is_empty())
        .count()
        == 0
    {
        return Err(format!(
            "{artifact_name} selected no authored fixture corpus"
        ));
    }

    let surface = read_json_entry(entries, "surface-verdict.json", artifact_name)?;
    if string_field(&surface, "status") != Some("success") {
        return Err(format!(
            "{artifact_name} surface verdict is {}",
            value_display(surface.get("status"))
        ));
    }
    let lint_summary = read_json_entry(entries, "lint-divergence-summary.json", artifact_name)?;
    assert_release_lint_divergence_summary(artifact_name, run, &lint_summary)?;
    assert_release_typecheck_shard_artifacts(
        artifact_name,
        run,
        entries,
        expected_typecheck_projects,
        observed_typecheck_projects,
    )
}

fn assert_release_lint_divergence_summary(
    artifact_name: &str,
    run: &Value,
    summary: &Value,
) -> Result<(), String> {
    if string_field(summary, "schema") != Some("vize.fixtureLintDivergenceIndex")
        || integer_field(summary, "version") != Some(1)
        || string_field(nested(summary, &["evidence"]), "commitSha") != Some(run_head_sha(run)?)
        || integer_field(summary, "projectCount").is_none_or(|value| value <= 0)
        || !summary
            .get("projects")
            .and_then(Value::as_array)
            .is_some_and(|projects| {
                integer_field(summary, "projectCount") == Some(projects.len() as i64)
            })
    {
        return Err(format!(
            "{artifact_name} lint divergence summary is not exact release evidence"
        ));
    }
    Ok(())
}

fn assert_release_typecheck_shard_artifacts(
    artifact_name: &str,
    run: &Value,
    entries: &BTreeMap<String, String>,
    expected_typecheck_projects: &BTreeSet<String>,
    observed_typecheck_projects: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let divergence_entries = matching_entries(entries, "-typecheck-divergence.json");
    let dependency_entries = matching_entries(entries, "-typecheck-dependencies.json");
    if divergence_entries.len() != dependency_entries.len() {
        return Err(format!(
            "{artifact_name} typecheck dependency artifact count {} does not match divergence artifact count {}",
            dependency_entries.len(),
            divergence_entries.len()
        ));
    }
    let mut dependencies = BTreeMap::new();
    for (entry_name, text) in dependency_entries {
        let dependency = parse_json_text(text, entry_name)?;
        let project = string_field(&dependency, "project")
            .ok_or_else(|| {
                format!("{artifact_name} typecheck dependency artifact is missing project")
            })?
            .to_string();
        if dependencies
            .insert(project.clone(), (dependency, sha256(text)))
            .is_some()
        {
            return Err(format!(
                "{artifact_name} duplicated typecheck dependency artifact for {project}"
            ));
        }
    }
    for (entry_name, text) in divergence_entries {
        let divergence = parse_json_text(text, entry_name)?;
        let project = string_field(&divergence, "project").ok_or_else(|| {
            format!("{artifact_name} typecheck divergence artifact is missing project")
        })?;
        let Some((dependency, dependency_sha256)) = dependencies.get(project) else {
            return Err(format!(
                "{artifact_name} has no typecheck dependency artifact for {project}"
            ));
        };
        assert_release_typecheck_divergence_artifact(
            artifact_name,
            run,
            &divergence,
            dependency,
            dependency_sha256,
        )?;
        if !expected_typecheck_projects.contains(project) {
            return Err(format!(
                "{artifact_name} includes unregistered typecheck performance project {project}"
            ));
        }
        if let Some(previous) =
            observed_typecheck_projects.insert(project.to_string(), artifact_name.to_string())
        {
            return Err(format!(
                "{artifact_name} duplicates typecheck performance release evidence for {project}; already seen in {previous}"
            ));
        }
    }
    Ok(())
}

fn assert_release_typecheck_coverage(
    expected_typecheck_projects: &BTreeSet<String>,
    observed_typecheck_projects: &BTreeMap<String, String>,
) -> Result<(), String> {
    let missing = expected_typecheck_projects
        .iter()
        .filter(|project| !observed_typecheck_projects.contains_key(*project))
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "Real Project Matrix release evidence is missing typecheck performance projects: {}",
            missing.join(", ")
        ));
    }
    Ok(())
}

fn assert_release_typecheck_divergence_artifact(
    artifact_name: &str,
    run: &Value,
    divergence: &Value,
    dependency: &Value,
    dependency_sha256: &str,
) -> Result<(), String> {
    let run_head_sha = run_head_sha(run)?;
    if string_field(divergence, "schema") != Some("vize.fixtureTypecheckDivergenceRun")
        || integer_field(divergence, "version") != Some(7)
        || string_field(nested(divergence, &["evidence"]), "commitSha") != Some(run_head_sha)
    {
        return Err(format!(
            "{artifact_name} typecheck divergence artifact is not bound to {run_head_sha}"
        ));
    }
    assert_release_typecheck_parity(artifact_name, divergence)?;
    assert_release_dependency_link(artifact_name, divergence, dependency, dependency_sha256)
}

fn assert_release_typecheck_parity(artifact_name: &str, divergence: &Value) -> Result<(), String> {
    if string_field(nested(divergence, &["enforcement"]), "budgetMode") != Some("enforce") {
        return Err(format!(
            "{artifact_name} typecheck divergence artifact used {} mode; release evidence must not be record-only",
            value_display(nested(divergence, &["enforcement"]).get("budgetMode"))
        ));
    }
    if bool_field(nested(divergence, &["budget"]), "passed") != Some(true)
        || string_field(nested(divergence, &["budget"]), "verdict") != Some("passed")
    {
        return Err(format!(
            "{artifact_name} typecheck divergence budget is {}",
            value_display(nested(divergence, &["budget"]).get("verdict"))
        ));
    }
    let summary = nested(divergence, &["divergence", "summary"]);
    if integer_field(summary, "falsePositiveCount") != Some(0)
        || integer_field(summary, "falseNegativeCount") != Some(0)
    {
        return Err(format!(
            "{artifact_name} typecheck divergence must have zero unexplained false positives and false negatives; got {} FP and {} FN",
            value_display(summary.get("falsePositiveCount")),
            value_display(summary.get("falseNegativeCount"))
        ));
    }
    assert_release_vue_coverage(artifact_name, nested(divergence, &["baseline", "coverage"]))?;
    assert_release_mutation_oracle(artifact_name, nested(divergence, &["mutationOracle"]))
}

fn assert_release_vue_coverage(artifact_name: &str, coverage: &Value) -> Result<(), String> {
    if string_field(coverage, "verdict") != Some("usable")
        || integer_field(coverage, "sharedVueFileCount").is_none()
        || integer_field(coverage, "vizeVueFileCount").is_none()
        || integer_field(coverage, "baselineVueFileCount").is_none()
        || integer_field(coverage, "sharedVueFileCount").is_some_and(|value| value <= 0)
        || integer_field(coverage, "vizeVueFileCount")
            != integer_field(coverage, "baselineVueFileCount")
        || integer_field(coverage, "sharedVueFileCount")
            != integer_field(coverage, "vizeVueFileCount")
        || string_field(coverage, "vizeVueFilesSha256")
            != string_field(coverage, "baselineVueFilesSha256")
        || !string_field(coverage, "vizeVueFilesSha256").is_some_and(is_sha256)
        || !empty_array_field(coverage, "missingVueFiles")
        || !empty_array_field(coverage, "unexpectedVueFiles")
    {
        return Err(format!(
            "{artifact_name} did not prove both tools checked the same non-empty authored Vue corpus"
        ));
    }
    Ok(())
}

fn assert_release_mutation_oracle(
    artifact_name: &str,
    mutation_oracle: &Value,
) -> Result<(), String> {
    let states = mutation_oracle
        .get("states")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let clean = states.first().unwrap_or(&Value::Null);
    let broken = states.get(1).unwrap_or(&Value::Null);
    let repaired = states.get(2).unwrap_or(&Value::Null);
    if string_field(mutation_oracle, "schema") != Some("vize.fixtureTypecheckSeededMutationOracle")
        || integer_field(mutation_oracle, "version") != Some(1)
        || bool_field(mutation_oracle, "passed") != Some(true)
        || string_field(mutation_oracle, "verdict") != Some("passed")
        || bool_field(mutation_oracle, "cleanExpectedDiagnosticPresent") != Some(false)
        || bool_field(mutation_oracle, "expectedDiagnosticMatched") != Some(true)
        || bool_field(mutation_oracle, "repairedExpectedDiagnosticPresent") != Some(false)
        || string_field(clean, "name") != Some("clean")
        || string_field(broken, "name") != Some("broken")
        || string_field(repaired, "name") != Some("repaired")
        || !has_mutation_state_evidence(clean)
        || !has_mutation_state_evidence(broken)
        || !has_mutation_state_evidence(repaired)
        || !string_field(clean, "sourceSha256").is_some_and(is_sha256)
        || !string_field(broken, "sourceSha256").is_some_and(is_sha256)
        || !string_field(repaired, "sourceSha256").is_some_and(is_sha256)
        || integer_field(clean, "sharedCount") != Some(0)
        || integer_field(clean, "falsePositiveCount") != Some(0)
        || integer_field(clean, "falseNegativeCount") != Some(0)
        || string_field(broken, "sourceSha256") == string_field(clean, "sourceSha256")
        || integer_field(broken, "sharedCount") != Some(1)
        || integer_field(broken, "messageMismatchCount") != Some(0)
        || integer_field(broken, "documentedDifferenceCount") != Some(0)
        || integer_field(broken, "falsePositiveCount") != Some(0)
        || integer_field(broken, "falseNegativeCount") != Some(0)
        || string_field(repaired, "sourceSha256") != string_field(clean, "sourceSha256")
        || integer_field(repaired, "sharedCount") != Some(0)
        || integer_field(repaired, "messageMismatchCount") != Some(0)
        || integer_field(repaired, "documentedDifferenceCount") != Some(0)
        || integer_field(repaired, "falsePositiveCount") != Some(0)
        || integer_field(repaired, "falseNegativeCount") != Some(0)
    {
        return Err(format!(
            "{artifact_name} has no passing seeded mutation oracle"
        ));
    }
    Ok(())
}

fn has_mutation_state_evidence(state: &Value) -> bool {
    let observed = nested(state, &["observed"]);
    has_summary_evidence(observed)
        && has_observed_mutation_parity(observed)
        && has_run_evidence(nested(state, &["vize"]))
        && has_run_evidence(nested(state, &["baseline"]))
}

fn has_summary_evidence(summary: &Value) -> bool {
    [
        "vizeDiagnosticCount",
        "baselineDiagnosticCount",
        "sharedCount",
        "messageMismatchCount",
        "documentedDifferenceCount",
        "falsePositiveCount",
        "falseNegativeCount",
    ]
    .into_iter()
    .all(|key| integer_field(summary, key).is_some_and(|value| value >= 0))
}

fn has_observed_mutation_parity(summary: &Value) -> bool {
    integer_field(summary, "messageMismatchCount") == Some(0)
        && integer_field(summary, "falsePositiveCount") == Some(0)
        && integer_field(summary, "falseNegativeCount") == Some(0)
}

fn has_run_evidence(run: &Value) -> bool {
    string_field(run, "command").is_some_and(|command| !command.is_empty())
        && integer_field(run, "exitCode").is_some()
        && string_field(run, "stdoutSha256").is_some_and(is_sha256)
        && string_field(run, "stderrSha256").is_some_and(is_sha256)
}

fn assert_release_dependency_link(
    artifact_name: &str,
    divergence: &Value,
    dependency: &Value,
    dependency_sha256: &str,
) -> Result<(), String> {
    if string_field(dependency, "schema") != Some("vize.fixtureTypecheckDependencyInstall")
        || integer_field(dependency, "version") != Some(2)
        || string_field(dependency, "project") != string_field(divergence, "project")
        || string_field(dependency, "revision") != string_field(divergence, "revision")
        || string_field(nested(dependency, &["evidence"]), "commitSha")
            != string_field(nested(divergence, &["evidence"]), "commitSha")
    {
        return Err(format!(
            "{artifact_name} typecheck dependency evidence is not bound to divergence"
        ));
    }
    if string_field(nested(divergence, &["preparation"]), "schema")
        != Some("vize.fixtureTypecheckPreparationEvidence")
        || integer_field(nested(divergence, &["preparation"]), "version") != Some(1)
        || string_field(nested(divergence, &["preparation"]), "payloadSha256")
            != Some(dependency_sha256)
    {
        return Err(format!(
            "{artifact_name} divergence artifact is missing dependency preparation linkage"
        ));
    }
    Ok(())
}

fn assert_artifact_bound_to_run(run: &Value, artifact: &Value) -> Result<(), String> {
    let artifact_name = string_field(artifact, "name").unwrap_or("unknown");
    if bool_field(artifact, "expired") == Some(true) {
        return Err(format!(
            "Real Project Matrix artifact {artifact_name} has expired"
        ));
    }
    let source = nested(artifact, &["workflow_run"]);
    if integer_field(source, "id") != integer_field(run, "id")
        || string_field(source, "head_sha") != string_field(run, "head_sha")
        || string_field(source, "head_branch") != string_field(run, "head_branch")
    {
        return Err(format!(
            "Real Project Matrix artifact {artifact_name} is not bound to run {}",
            value_display(run.get("id"))
        ));
    }
    Ok(())
}

fn typecheck_performance_project_ids(repo_root: &Path) -> Result<BTreeSet<String>, String> {
    let path = repo_root.join("tests/_fixtures/vue-ecosystem-fixtures.json");
    let registry = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let registry: Value = serde_json::from_str(&registry)
        .map_err(|error| format!("cannot parse {}: {error}", path.display()))?;
    let projects = registry
        .get("projects")
        .and_then(Value::as_array)
        .ok_or_else(|| "Fixture registry must list projects".to_string())?;
    let mut ids = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for project in projects {
        if bool_field(nested(project, &["typecheckPerformance"]), "enabled") != Some(true) {
            continue;
        }
        let id = string_field(project, "id")
            .ok_or_else(|| "Typecheck performance registry project is missing id".to_string())?;
        if !ids.insert(id.to_string()) {
            duplicates.insert(id.to_string());
        }
    }
    if !duplicates.is_empty() {
        return Err(format!(
            "Typecheck performance registry has duplicate project ids: {}",
            duplicates.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }
    Ok(ids)
}

fn matching_entries<'a>(
    entries: &'a BTreeMap<String, String>,
    suffix: &str,
) -> Vec<(&'a str, &'a str)> {
    entries
        .iter()
        .filter(|(name, _)| {
            name.rsplit('/')
                .next()
                .is_some_and(|base| base.len() > suffix.len() && base.ends_with(suffix))
        })
        .map(|(name, text)| (name.as_str(), text.as_str()))
        .collect()
}

fn read_json_entry(
    entries: &BTreeMap<String, String>,
    name: &str,
    artifact_name: &str,
) -> Result<Value, String> {
    parse_json_text(read_text_entry(entries, name, artifact_name)?, name)
}

fn parse_json_text(text: &str, name: &str) -> Result<Value, String> {
    serde_json::from_str(text)
        .map_err(|error| format!("Invalid release evidence JSON {name}: {error}"))
}

fn read_text_entry<'a>(
    entries: &'a BTreeMap<String, String>,
    name: &str,
    artifact_name: &str,
) -> Result<&'a str, String> {
    entries
        .get(name)
        .filter(|value| !value.is_empty())
        .map(String::as_str)
        .ok_or_else(|| format!("{artifact_name} is missing {name}"))
}

fn run_head_sha(run: &Value) -> Result<&str, String> {
    string_field(run, "head_sha")
        .ok_or_else(|| "Real Project Matrix run is missing head_sha".to_string())
}

fn nested<'a>(value: &'a Value, path: &[&str]) -> &'a Value {
    let mut current = value;
    for key in path {
        current = current.get(*key).unwrap_or(&Value::Null);
    }
    current
}

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn integer_field(value: &Value, key: &str) -> Option<i64> {
    let value = value.get(key)?;
    value.as_i64().or_else(|| {
        value
            .as_u64()
            .filter(|value| *value <= i64::MAX as u64)
            .map(|value| value as i64)
    })
}

fn bool_field(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

fn empty_array_field(value: &Value, key: &str) -> bool {
    value
        .get(key)
        .and_then(Value::as_array)
        .is_some_and(Vec::is_empty)
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn value_display(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(value)) => value.clone(),
        Some(value) => value.to_string(),
        None => "undefined".to_string(),
    }
}
