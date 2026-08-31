#![allow(dead_code)]

use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[path = "./common.rs"]
mod common;

pub const SCHEMA: &str = "vize.davinciCorpusBaseline";
pub const SCHEMA_VERSION: u64 = 1;
pub const UNSTABLE_SCHEMA: &str = "vize.davinciCorpusUnstableRows";
pub const REGISTRY_REL: &str = "tests/_fixtures/vue-ecosystem-fixtures.json";
pub const BASELINE_REL: &str = "tests/_fixtures/davinci-baseline.json";
pub const NOTES_REL: &str = "davinci-road/plan/corpus-baseline-notes.md";
pub const UNSTABLE_REL: &str = "davinci-road/plan/corpus-baseline-unstable.json";
pub const SURFACES: &[&str] = &["compiler", "formatter", "linter", "typechecker"];

const PAYLOAD_FAILURE_FIELDS: &[&str] = &["spawnError", "parseError", "validationError"];
const FIXTURE_ROOT: &str = "tests/_fixtures/_git";

#[derive(Clone, Debug, Serialize)]
pub struct Row {
    pub surface: String,
    pub project: String,
    pub file_count: u64,
    pub content_hash: String,
}

pub fn load_manifest(root: &Path) -> Result<Value, String> {
    let manifest = common::read_json(root.join(REGISTRY_REL))?;
    let projects = manifest
        .get("projects")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{REGISTRY_REL} lists no projects"))?;
    if projects.is_empty() {
        return Err(format!("{REGISTRY_REL} lists no projects"));
    }
    let mut seen = BTreeSet::new();
    for project in projects {
        let id = project
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{REGISTRY_REL} contains a project without id"))?;
        if !seen.insert(id.to_string()) {
            return Err(format!(
                "{REGISTRY_REL} contains duplicate project id: {id}"
            ));
        }
    }
    Ok(manifest)
}

pub fn surface_filter(values: &[String]) -> Result<Vec<String>, String> {
    let mut surfaces = if values.is_empty() {
        SURFACES
            .iter()
            .map(|surface| (*surface).to_string())
            .collect()
    } else {
        values.to_vec()
    };
    surfaces.sort();
    surfaces.dedup();
    for surface in &surfaces {
        if !SURFACES.contains(&surface.as_str()) {
            return Err(format!(
                "unknown surface: {surface} (expected one of {})",
                SURFACES.join(", ")
            ));
        }
    }
    Ok(surfaces)
}

pub fn build_artifact(rows: &[Row], manifest: &Value) -> Result<Value, String> {
    let projects = rows
        .iter()
        .map(|row| row.project.clone())
        .collect::<BTreeSet<_>>();
    let surfaces = rows
        .iter()
        .map(|row| row.surface.clone())
        .collect::<BTreeSet<_>>();
    let mut file_count_by_surface = BTreeMap::new();
    let mut total_file_count = 0u64;
    for surface in &surfaces {
        file_count_by_surface.insert(surface.clone(), 0u64);
    }
    for row in rows {
        *file_count_by_surface
            .entry(row.surface.clone())
            .or_default() += row.file_count;
        total_file_count += row.file_count;
    }
    let manifest_project_count = manifest_projects(manifest)?.len();
    Ok(json!({
        "schema": SCHEMA,
        "version": SCHEMA_VERSION,
        "registry": REGISTRY_REL,
        "notes": NOTES_REL,
        "hashed_fields": {
            "compiler": ["compilerArtifacts", "exitCode", "stdout"],
            "formatter": ["exitCode", "formatterCheck", "stdout"],
            "linter": ["exitCode", "stderr", "stdout"],
            "typechecker": ["exitCode", "stderr", "stdout", "typecheckerCoverage"],
        },
        "excluded_fields": {
            "compiler": ["stderr"],
            "formatter": ["stderr"],
        },
        "scope": {
            "manifest_project_count": manifest_project_count,
            "projects_run": projects.len(),
            "surfaces": surfaces.into_iter().collect::<Vec<_>>(),
            "surfaces_per_project": file_count_by_surface.len(),
            "row_count": rows.len(),
            "total_file_count": total_file_count,
            "file_count_by_surface": file_count_by_surface,
        },
        "rows": rows,
    }))
}

pub fn expected_comparison_count(manifest: &Value, surfaces: &[String]) -> Result<usize, String> {
    Ok(manifest_projects(manifest)?.len() * surfaces.len())
}

pub fn verify_scope(
    artifact: &Value,
    manifest: &Value,
    surfaces: &[String],
    label: &str,
) -> Result<Vec<String>, String> {
    let mut reasons = Vec::new();
    let manifest_ids = manifest_projects(manifest)?
        .iter()
        .map(|project| project_string(project, "id"))
        .collect::<Result<Vec<_>, _>>()?;
    let mut manifest_ids = manifest_ids;
    manifest_ids.sort();
    let mut expected_surfaces = surfaces.to_vec();
    expected_surfaces.sort();
    let scope = artifact.get("scope").unwrap_or(&Value::Null);
    if artifact.get("schema").and_then(Value::as_str) != Some(SCHEMA)
        || artifact.get("version").and_then(Value::as_u64) != Some(SCHEMA_VERSION)
    {
        reasons.push(format!("{label}: schema is not {SCHEMA} v{SCHEMA_VERSION}"));
        return Ok(reasons);
    }
    if scope
        .get("manifest_project_count")
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        != Some(manifest_ids.len())
    {
        reasons.push(format!(
            "{label}: scope.manifest_project_count {} != manifest {}",
            display_value(scope.get("manifest_project_count")),
            manifest_ids.len()
        ));
    }
    let actual_surfaces = scope
        .get("surfaces")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if actual_surfaces != expected_surfaces {
        reasons.push(format!(
            "{label}: scope.surfaces [{}] != expected [{}]",
            actual_surfaces.join(", "),
            expected_surfaces.join(", ")
        ));
    }
    let rows = artifact
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if scope
        .get("row_count")
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        != Some(rows.len())
    {
        reasons.push(format!(
            "{label}: scope.row_count {} != {} rows",
            display_value(scope.get("row_count")),
            rows.len()
        ));
    }
    let expected_row_count = manifest_ids.len() * expected_surfaces.len();
    if rows.len() != expected_row_count {
        reasons.push(format!(
            "{label}: {} rows != {} projects x {} surfaces = {}",
            rows.len(),
            manifest_ids.len(),
            expected_surfaces.len(),
            expected_row_count
        ));
    }
    for surface in &expected_surfaces {
        let mut surface_projects = rows
            .iter()
            .filter(|row| row.get("surface").and_then(Value::as_str) == Some(surface.as_str()))
            .filter_map(|row| row.get("project").and_then(Value::as_str))
            .map(str::to_string)
            .collect::<Vec<_>>();
        surface_projects.sort();
        let missing = manifest_ids
            .iter()
            .filter(|id| !surface_projects.contains(id))
            .cloned()
            .collect::<Vec<_>>();
        let extra = surface_projects
            .iter()
            .filter(|id| !manifest_ids.contains(id))
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            reasons.push(format!(
                "{label}: surface {surface} is missing projects [{}]",
                missing.join(", ")
            ));
        }
        if !extra.is_empty() {
            reasons.push(format!(
                "{label}: surface {surface} has unknown projects [{}]",
                extra.join(", ")
            ));
        }
    }
    let mut total_file_count = 0u64;
    for row in &rows {
        match row.get("file_count").and_then(Value::as_u64) {
            Some(file_count) => total_file_count += file_count,
            None => reasons.push(format!(
                "{label}: {}/{} has invalid file_count",
                row.get("surface").and_then(Value::as_str).unwrap_or("?"),
                row.get("project").and_then(Value::as_str).unwrap_or("?")
            )),
        }
    }
    if scope.get("total_file_count").and_then(Value::as_u64) != Some(total_file_count) {
        reasons.push(format!(
            "{label}: scope.total_file_count {} != {} summed",
            display_value(scope.get("total_file_count")),
            total_file_count
        ));
    }
    if total_file_count == 0 {
        reasons.push(format!("{label}: zero-file run (total_file_count is 0)"));
    }
    let declared_zero = manifest_projects(manifest)?
        .iter()
        .filter(|project| project.get("expectedVueFileCount").and_then(Value::as_u64) == Some(0))
        .map(|project| project_string(project, "id"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    for row in &rows {
        if row.get("file_count").and_then(Value::as_u64) == Some(0) {
            let project = row.get("project").and_then(Value::as_str).unwrap_or("?");
            if !declared_zero.contains(project) {
                reasons.push(format!(
                    "{label}: {}/{} ran zero files but the manifest does not declare expectedVueFileCount 0",
                    row.get("surface").and_then(Value::as_str).unwrap_or("?"),
                    project
                ));
            }
        }
    }
    Ok(reasons)
}

pub fn diff_rows(baseline_rows: &[Value], fresh_rows: &[Row]) -> Vec<Value> {
    let baseline_by_key = baseline_rows
        .iter()
        .filter_map(|row| Some((row_key_value(row)?, row.clone())))
        .collect::<BTreeMap<_, _>>();
    let fresh_by_key = fresh_rows
        .iter()
        .map(|row| (format!("{}\0{}", row.surface, row.project), row))
        .collect::<BTreeMap<_, _>>();
    let mut drift = Vec::new();
    for (key, baseline) in &baseline_by_key {
        match fresh_by_key.get(key) {
            None => {
                let mut row = baseline.clone();
                row["kind"] = json!("missing-in-fresh");
                drift.push(row);
            }
            Some(fresh) => {
                let baseline_hash = baseline
                    .get("content_hash")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let baseline_file_count = baseline
                    .get("file_count")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                if baseline_hash != fresh.content_hash || baseline_file_count != fresh.file_count {
                    drift.push(json!({
                        "surface": baseline.get("surface").and_then(Value::as_str).unwrap_or(""),
                        "project": baseline.get("project").and_then(Value::as_str).unwrap_or(""),
                        "kind": "changed",
                        "baseline_file_count": baseline_file_count,
                        "fresh_file_count": fresh.file_count,
                        "baseline_hash": baseline_hash,
                        "fresh_hash": fresh.content_hash,
                    }));
                }
            }
        }
    }
    for (key, fresh) in fresh_by_key {
        if !baseline_by_key.contains_key(&key) {
            drift.push(json!({
                "surface": fresh.surface,
                "project": fresh.project,
                "kind": "missing-in-baseline",
                "file_count": fresh.file_count,
                "content_hash": fresh.content_hash,
            }));
        }
    }
    drift.sort_by(|left, right| {
        (
            left.get("surface").and_then(Value::as_str).unwrap_or(""),
            left.get("project").and_then(Value::as_str).unwrap_or(""),
        )
            .cmp(&(
                right.get("surface").and_then(Value::as_str).unwrap_or(""),
                right.get("project").and_then(Value::as_str).unwrap_or(""),
            ))
    });
    drift
}

pub fn reduce_shards(
    root: &Path,
    shard_dirs: &[PathBuf],
    tools: &[String],
) -> Result<Vec<Row>, String> {
    let mut rows = Vec::new();
    let mut seen_projects = BTreeSet::new();
    let mut expected_tools = tools.to_vec();
    expected_tools.sort();
    for shard_dir in shard_dirs {
        let summary_path = shard_dir.join("summary.json");
        if !summary_path.exists() {
            return Err(format!(
                "shard output has no summary.json: {}",
                shard_dir.display()
            ));
        }
        let summary = common::read_json(&summary_path)?;
        assert_clean_summary(&summary, shard_dir)?;
        let projects = summary
            .get("projects")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{} must list projects", summary_path.display()))?;
        for project in projects {
            let project_id = project_string(project, "id")?;
            if !seen_projects.insert(project_id.clone()) {
                return Err(format!(
                    "project {project_id} appears in more than one shard"
                ));
            }
            let runs = project
                .get("runs")
                .and_then(Value::as_array)
                .ok_or_else(|| format!("{project_id} must list runs"))?;
            let mut run_tools = runs
                .iter()
                .filter_map(|run| run.get("tool").and_then(Value::as_str))
                .map(str::to_string)
                .collect::<Vec<_>>();
            run_tools.sort();
            if run_tools != expected_tools {
                return Err(format!(
                    "project {project_id} ran surfaces [{}], expected [{}]",
                    run_tools.join(", "),
                    expected_tools.join(", ")
                ));
            }
            for run in runs {
                rows.push(reduce_run(root, shard_dir, &project_id, run)?);
            }
        }
    }
    rows.sort_by(|left, right| {
        left.surface
            .cmp(&right.surface)
            .then_with(|| left.project.cmp(&right.project))
    });
    Ok(rows)
}

pub fn run_matrix(
    root: &Path,
    shards: usize,
    vize_bin: &Path,
    tools: &[String],
    scratch_dir: &Path,
    timeout_ms: Option<u64>,
) -> Result<Vec<PathBuf>, String> {
    fs::create_dir_all(scratch_dir)
        .map_err(|error| format!("cannot create {}: {error}", scratch_dir.display()))?;
    let fixture_paths = list_matrix_fixture_paths(root, shards)?;
    assert_hydrated_gitlink_fixtures(root, &fixture_paths)?;
    let mut children = Vec::new();
    for index in 0..shards {
        let output_dir = scratch_dir.join(format!("shard-{index}"));
        let mut args = vec![
            "tools/commands/fixtures/tool-matrix-report.rs".to_string(),
            "--shard-index".to_string(),
            index.to_string(),
            "--shard-count".to_string(),
            shards.to_string(),
            "--vize-bin".to_string(),
            vize_bin.display().to_string(),
            "--output-dir".to_string(),
            output_dir.display().to_string(),
        ];
        if let Some(timeout_ms) = timeout_ms {
            args.extend(["--timeout-ms".to_string(), timeout_ms.to_string()]);
        }
        for tool in tools {
            args.extend(["--tool".to_string(), tool.clone()]);
        }
        let child = Command::new("rust-script")
            .args(&args)
            .current_dir(root)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("failed to spawn shard {index}: {error}"))?;
        children.push((index, output_dir, args, child));
    }
    let mut failures = Vec::new();
    for (index, output_dir, args, child) in children {
        let output = child
            .wait_with_output()
            .map_err(|error| format!("failed to wait for shard {index}: {error}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
            println!("[shard {index}] {line}");
        }
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let detail = stderr.trim();
            if !detail.is_empty() {
                println!("[shard {index}] stderr: {detail}");
            }
            failures.push(format!(
                "shard {index} exited {}{}",
                output.status.code().unwrap_or(1),
                describe_shard_failure(&output_dir)?,
            ));
            if detail.is_empty() {
                failures.push(format!("  command: rust-script {}", args.join(" ")));
            }
        }
    }
    if !failures.is_empty() {
        return Err(format!("matrix run failed:\n  {}", failures.join("\n  ")));
    }
    Ok((0..shards)
        .map(|index| scratch_dir.join(format!("shard-{index}")))
        .collect())
}

pub fn cleanup_scratch(path: &Path) -> Result<(), String> {
    fs::remove_dir_all(path).map_err(|error| format!("cannot remove {}: {error}", path.display()))
}

pub fn scratch_root(root: &Path, label: &str) -> PathBuf {
    root.join(".vize/davinci-corpus").join(label)
}

pub fn resolve_vize_bin(root: &Path, vize_bin: Option<PathBuf>) -> Result<PathBuf, String> {
    let candidate = vize_bin.unwrap_or_else(|| root.join("target/release/vize"));
    let resolved = if candidate.is_absolute() {
        candidate
    } else {
        root.join(candidate)
    };
    if !resolved.exists() {
        return Err(format!(
            "vize binary not found: {} (build with: cargo build --release -p vize)",
            resolved.display()
        ));
    }
    Ok(resolved)
}

pub fn load_unstable_rows(root: &Path, manifest: &Value) -> Result<Vec<Value>, String> {
    let path = root.join(UNSTABLE_REL);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let sidecar = common::read_json(&path)?;
    if sidecar.get("schema").and_then(Value::as_str) != Some(UNSTABLE_SCHEMA)
        || sidecar.get("version").and_then(Value::as_u64) != Some(1)
    {
        return Err(format!(
            "{UNSTABLE_REL}: schema is not {UNSTABLE_SCHEMA} v1"
        ));
    }
    let rows = sidecar
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{UNSTABLE_REL}: rows must be an array"))?;
    let manifest_ids = manifest_projects(manifest)?
        .iter()
        .map(|project| project_string(project, "id"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let mut seen = BTreeSet::new();
    for row in rows {
        let surface = row
            .get("surface")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{UNSTABLE_REL}: row has no surface"))?;
        let project = row
            .get("project")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{UNSTABLE_REL}: row has no project"))?;
        if !SURFACES.contains(&surface) {
            return Err(format!("{UNSTABLE_REL}: unknown surface {surface}"));
        }
        if !manifest_ids.contains(project) {
            return Err(format!("{UNSTABLE_REL}: unknown project {project}"));
        }
        if row
            .get("reason")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            return Err(format!("{UNSTABLE_REL}: {surface}/{project} has no reason"));
        }
        let key = format!("{surface} {project}");
        if !seen.insert(key.clone()) {
            return Err(format!("{UNSTABLE_REL}: duplicate row {key}"));
        }
    }
    Ok(rows.to_vec())
}

pub fn clean_fixtures(root: &Path) -> Result<usize, String> {
    let targets = materialized_node_modules(root)?;
    for relative in &targets {
        let full = root.join(relative);
        fs::remove_dir_all(&full)
            .map_err(|error| format!("cannot remove {}: {error}", full.display()))?;
    }
    Ok(targets.len())
}

pub fn assert_fixtures_pristine(root: &Path, allow_materialized: bool) -> Result<(), String> {
    let drifted = drifted_submodules(root)?;
    let materialized = if allow_materialized {
        Vec::new()
    } else {
        materialized_node_modules(root)?
    };
    if drifted.is_empty() && materialized.is_empty() {
        return Ok(());
    }
    let mut lines = vec!["corpus fixtures are not at their pinned state:".to_string()];
    if !drifted.is_empty() {
        lines.push(format!(
            "  {} submodule(s) drifted from the recorded sha:",
            drifted.len()
        ));
        for (path, reason) in drifted.iter().take(5) {
            lines.push(format!("    {path} - {reason}"));
        }
        if drifted.len() > 5 {
            lines.push(format!("    ... and {} more", drifted.len() - 5));
        }
        lines.push("  hydrate them with:".to_string());
        lines.push(format!(
            "  git submodule update --init --checkout --force -- {FIXTURE_ROOT}"
        ));
    }
    if !materialized.is_empty() {
        lines.push(format!(
            "  {} materialized node_modules directory(ies) left by a previous run:",
            materialized.len()
        ));
        for relative in materialized.iter().take(5) {
            lines.push(format!("    {relative}"));
        }
        if materialized.len() > 5 {
            lines.push(format!("    ... and {} more", materialized.len() - 5));
        }
        lines.push("  clean them with `--clean-fixtures`, or by hand:".to_string());
        lines.push(format!(
            "  find {FIXTURE_ROOT} -type d -name node_modules -prune -exec rm -rf {{}} +"
        ));
    }
    lines.extend([
        "  a sweep over contaminated fixtures measures a different tree than the".to_string(),
        "  baseline did - see davinci-road/plan/corpus-baseline-notes.md, Re-record 2".to_string(),
        "  (pass --allow-dirty-fixtures to sweep anyway; the hashes are then not".to_string(),
        "  comparable to the committed baseline)".to_string(),
    ]);
    Err(lines.join("\n"))
}

fn manifest_projects(manifest: &Value) -> Result<&Vec<Value>, String> {
    manifest
        .get("projects")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{REGISTRY_REL} lists no projects"))
}

fn reduce_run(
    _root: &Path,
    shard_dir: &Path,
    project_id: &str,
    run: &Value,
) -> Result<Row, String> {
    let tool = run
        .get("tool")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{project_id} has a run without tool"))?;
    let payload_path = shard_dir.join(format!("{project_id}-{tool}.json"));
    if !payload_path.exists() {
        return Err(format!(
            "run payload is missing: {}",
            payload_path.display()
        ));
    }
    let payload = common::read_json(&payload_path)?;
    for field in PAYLOAD_FAILURE_FIELDS {
        if payload.get(*field).is_some_and(|value| !value.is_null()) {
            return Err(format!(
                "{project_id}/{tool} payload carries {field}: {}",
                payload.get(*field).unwrap()
            ));
        }
    }
    let hashed_fields =
        hashed_fields(tool).ok_or_else(|| format!("unsupported surface: {tool}"))?;
    let mut content = serde_json::Map::new();
    for field in hashed_fields {
        let value = payload
            .get(*field)
            .ok_or_else(|| format!("{project_id}/{tool} payload has no {field} field"))?;
        content.insert((*field).to_string(), value.clone());
    }
    let file_count = payload_file_count(tool, &payload)?;
    if run.get("fileCount").and_then(Value::as_u64) != Some(file_count) {
        return Err(format!(
            "{project_id}/{tool} summary fileCount {} != payload-derived {file_count}",
            display_value(run.get("fileCount"))
        ));
    }
    Ok(Row {
        surface: tool.to_string(),
        project: project_id.to_string(),
        file_count,
        content_hash: sha256(&canonical_json(&Value::Object(content))),
    })
}

fn hashed_fields(tool: &str) -> Option<&'static [&'static str]> {
    match tool {
        "compiler" => Some(&["compilerArtifacts", "exitCode", "stdout"]),
        "formatter" => Some(&["exitCode", "formatterCheck", "stdout"]),
        "linter" => Some(&["exitCode", "stderr", "stdout"]),
        "typechecker" => Some(&["exitCode", "stderr", "stdout", "typecheckerCoverage"]),
        _ => None,
    }
}

fn payload_file_count(tool: &str, payload: &Value) -> Result<u64, String> {
    match tool {
        "compiler" => payload
            .pointer("/compilerArtifacts/inputFileCount")
            .and_then(Value::as_u64)
            .ok_or_else(|| "compiler payload has no compilerArtifacts.inputFileCount".to_string()),
        "typechecker" => payload
            .pointer("/parsed/fileCount")
            .and_then(Value::as_u64)
            .ok_or_else(|| "typechecker payload has no parsed.fileCount".to_string()),
        "linter" => payload
            .get("parsed")
            .and_then(Value::as_array)
            .map(|items| items.len() as u64)
            .ok_or_else(|| "linter payload has no parsed array".to_string()),
        "formatter" => payload
            .pointer("/formatterCheck/checkedFileCount")
            .or_else(|| payload.pointer("/formatterCheck/fileCount"))
            .and_then(Value::as_u64)
            .ok_or_else(|| "formatter payload has no formatterCheck.checkedFileCount".to_string()),
        _ => Err(format!("unsupported surface: {tool}")),
    }
}

fn assert_clean_summary(summary: &Value, shard_dir: &Path) -> Result<(), String> {
    let counts = summary
        .get("summary")
        .ok_or_else(|| format!("shard summary is missing counts: {}", shard_dir.display()))?;
    let failed_runs = count(counts, "failedRuns");
    let missing_fixture_runs = count(counts, "missingFixtureRuns");
    let planned_runs = count(counts, "plannedRuns");
    let ok_runs = count(counts, "okRuns");
    let findings_runs = count(counts, "findingsRuns");
    let run_count = count(counts, "runCount");
    if failed_runs == 0
        && missing_fixture_runs == 0
        && planned_runs == 0
        && ok_runs + findings_runs == run_count
    {
        Ok(())
    } else {
        Err(format!(
            "shard summary is not clean: {} ({counts})",
            shard_dir.display()
        ))
    }
}

fn list_matrix_fixture_paths(root: &Path, shards: usize) -> Result<Vec<String>, String> {
    let mut fixture_paths = Vec::new();
    for index in 0..shards {
        let args = vec![
            "tools/commands/fixtures/tool-matrix-report.rs".to_string(),
            "--list-fixture-paths".to_string(),
            "--shard-index".to_string(),
            index.to_string(),
            "--shard-count".to_string(),
            shards.to_string(),
        ];
        let output = common::run_capture_in("rust-script", &args, root)
            .map_err(|error| format!("fixture path selection failed for shard {index}: {error}"))?;
        fixture_paths.extend(
            output
                .stdout
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string),
        );
    }
    Ok(fixture_paths)
}

fn assert_hydrated_gitlink_fixtures(root: &Path, fixture_paths: &[String]) -> Result<(), String> {
    let missing = fixture_paths
        .iter()
        .filter(|relative| !is_hydrated(root.join(relative)))
        .take(10)
        .cloned()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "fixture corpus is not hydrated; first missing fixture(s): {}\n  git submodule update --init --checkout --force -- {}",
            missing.join(", "),
            FIXTURE_ROOT
        ))
    }
}

fn is_hydrated(path: PathBuf) -> bool {
    if !path.is_dir() {
        return false;
    }
    fs::read_dir(path)
        .ok()
        .is_some_and(|mut entries| entries.any(|entry| entry.is_ok()))
}

fn describe_shard_failure(output_dir: &Path) -> Result<String, String> {
    let summary_path = output_dir.join("summary.json");
    if !summary_path.exists() {
        return Ok(" (no summary.json written)".to_string());
    }
    let summary = common::read_json(summary_path)?;
    let mut failed = Vec::new();
    for project in summary
        .get("projects")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let id = project.get("id").and_then(Value::as_str).unwrap_or("?");
        for run in project
            .get("runs")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let status = run.get("status").and_then(Value::as_str).unwrap_or("");
            if matches!(status, "failed" | "missing-fixture") {
                failed.push(format!(
                    "{id}/{}: {status}",
                    run.get("tool").and_then(Value::as_str).unwrap_or("?")
                ));
            }
        }
    }
    if failed.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!(" ({})", failed.join(", ")))
    }
}

fn drifted_submodules(root: &Path) -> Result<Vec<(String, String)>, String> {
    let output = Command::new("git")
        .args(["submodule", "status", "--", FIXTURE_ROOT])
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to inspect fixture submodules: {error}"))?;
    if !output.status.success() {
        return Ok(Vec::new());
    }
    let mut drifted = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let reason = match line.as_bytes().first().copied() {
            Some(b'-') => "not initialized",
            Some(b'+') => "checked out at a different sha",
            Some(b'U') => "has merge conflicts",
            _ => continue,
        };
        let fields = line[1..].split_whitespace().collect::<Vec<_>>();
        if fields.len() >= 2 {
            drifted.push((fields[1].to_string(), reason.to_string()));
        }
    }
    drifted.sort();
    Ok(drifted)
}

fn materialized_node_modules(root: &Path) -> Result<Vec<String>, String> {
    let mut found = Vec::new();
    walk_node_modules(root, &root.join(FIXTURE_ROOT), 0, 3, &mut found)?;
    found.sort();
    Ok(found)
}

fn walk_node_modules(
    root: &Path,
    dir: &Path,
    depth: usize,
    max_depth: usize,
    found: &mut Vec<String>,
) -> Result<(), String> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read {}: {error}", dir.display()))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "node_modules" {
            found.push(common::relative_path(root, &path));
            continue;
        }
        if name == ".git" || depth >= max_depth {
            continue;
        }
        walk_node_modules(root, &path, depth + 1, max_depth, found)?;
    }
    Ok(())
}

fn row_key_value(row: &Value) -> Option<String> {
    Some(format!(
        "{}\0{}",
        row.get("surface")?.as_str()?,
        row.get("project")?.as_str()?
    ))
}

fn count(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn project_string(project: &Value, field: &str) -> Result<String, String> {
    project
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("project is missing {field}"))
}

fn display_value(value: Option<&Value>) -> String {
    value
        .map(Value::to_string)
        .unwrap_or_else(|| "undefined".to_string())
}

pub fn canonical_json(value: &Value) -> String {
    match value {
        Value::Array(items) => format!(
            "[{}]",
            items
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Object(map) => {
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort();
            format!(
                "{{{}}}",
                keys.into_iter()
                    .map(|key| format!(
                        "{}:{}",
                        serde_json::to_string(key).unwrap(),
                        canonical_json(&map[key])
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        _ => serde_json::to_string(value).unwrap(),
    }
}

pub fn sha256(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    format!("{digest:x}")
}

pub fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn positive_integer(value: &str, name: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("{name} must be a positive integer"))
}
