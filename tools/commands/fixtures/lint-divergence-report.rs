#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! glob = "0.3"
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! sha2 = "0.10"
//! ```

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    time::Instant,
};

#[path = "../../rust/common.rs"]
mod common;

const DEFAULT_PRESET: &str = "ecosystem";
const RULE_MAP_REL: &str = "tests/_fixtures/patina-eslint-vue-rule-map.json";
const LEDGER_REL: &str = "tests/_fixtures/patina-lint-documented-divergences.json";

#[derive(Clone, Debug)]
struct Args {
    measure_coverage_gap: bool,
    budget_mode: String,
    output_dir: PathBuf,
    preset: Option<String>,
    projects: Vec<String>,
    registry: PathBuf,
    shard_count: usize,
    shard_index: usize,
    timeout_ms: u64,
    vize_bin: Option<PathBuf>,
}

#[derive(Clone, Debug)]
struct Launch {
    command: String,
    prefix: Vec<String>,
}

#[derive(Clone, Debug)]
struct LintRecord {
    file: String,
    rule_id: String,
    upstream_rule_id: Option<String>,
    severity: String,
    line: u64,
    column: u64,
    end_line: u64,
    end_column: u64,
    message: String,
}

fn main() -> ExitCode {
    common::main_result(run().map(|_| ()))
}

fn run() -> Result<Vec<Value>, String> {
    let root = common::repo_root()?;
    let args = parse_args(env::args().skip(1).collect(), &root)?;
    let registry = common::read_json(&args.registry)?;
    let rule_map = common::read_json(root.join(RULE_MAP_REL))?;
    let ledger = common::read_json(root.join(LEDGER_REL)).unwrap_or_else(|_| json!([]));
    if !ledger.is_array() {
        return Err("The lint divergence ledger must be an array".to_string());
    }
    let selected = selected_projects(&registry, &args)?;
    if selected.is_empty() {
        return Err("No linter-covered project matched the selection".to_string());
    }
    let launch = resolve_vize_launch(&root, args.vize_bin.as_deref())?;
    let evidence = collect_run_evidence(&root)?;
    let comparable =
        select_comparable_rules(&rule_map, args.preset.as_deref(), args.measure_coverage_gap)?;
    fs::create_dir_all(&args.output_dir)
        .map_err(|error| format!("cannot create {}: {error}", args.output_dir.display()))?;

    let mut artifacts = Vec::new();
    for project in selected {
        if let Some(artifact) = measure_project(
            &root,
            &args,
            &launch,
            &evidence,
            &rule_map,
            &comparable,
            project,
        )? {
            artifacts.push(artifact);
        }
    }
    write_index(&root, &args, &evidence, &artifacts)?;
    assert_budgets_passed(&artifacts, &args.budget_mode)?;
    Ok(artifacts)
}

fn measure_project(
    root: &Path,
    args: &Args,
    launch: &Launch,
    evidence: &Value,
    rule_map: &Value,
    comparable: &ComparableRules,
    project: &Value,
) -> Result<Option<Value>, String> {
    let project_id = project_string(project, "id")?;
    let cwd = root.join(project_string(project, "fixturePath")?);
    if !is_hydrated(&cwd) {
        println!("Skipped {project_id}: fixture is not hydrated");
        return Ok(None);
    }
    let files = collect_vue_input_paths(&cwd, project)?;
    if files.is_empty() && project.get("expectedVueFileCount").and_then(Value::as_u64) != Some(0) {
        return Err(format!("{project_id} matched no Vue files"));
    }

    let patina_findings = run_patina(&cwd, project, args, launch, files.len())?;
    let baseline_started = Instant::now();
    let baseline = run_eslint_baseline(root, &cwd, &files, &comparable.rules)?;
    let baseline_duration_ms = baseline_started.elapsed().as_millis() as u64;
    reconcile_corpus(&project_id, &files, &baseline.results, &cwd)?;
    let divergence = compare_lint_findings(
        &project_id,
        &cwd,
        rule_map,
        &patina_findings,
        &baseline.results,
    )?;
    let mut artifact = json!({
        "schema": "vize.fixtureLintDivergenceRun",
        "version": 2,
        "project": project_id,
        "revision": project.get("revision").cloned().unwrap_or(Value::Null),
        "preset": args.preset.clone().unwrap_or_else(|| "all-mapped".to_string()),
        "evidence": evidence,
        "files": { "comparedCount": files.len() },
        "baseline": {
            "package": rule_map.pointer("/upstream/package").cloned().unwrap_or(Value::Null),
            "version": baseline.version,
            "comparedRuleCount": comparable.rules.len(),
            "mappedRuleCount": comparable.mapped_rule_count,
            "skippedByPresetCount": comparable.skipped_by_preset_count,
            "droppedConfigMessageCount": baseline.dropped_config_message_count,
            "durationMs": baseline_duration_ms,
        },
        "divergence": divergence,
    });
    let budget = evaluate_budget(&artifact);
    artifact["budget"] = budget;
    let json_path = args
        .output_dir
        .join(format!("{}-lint-divergence.json", project_id));
    let markdown_path = args
        .output_dir
        .join(format!("{}-lint-divergence.md", project_id));
    common::write_json_pretty(&json_path, &artifact)?;
    common::write_text(&markdown_path, &render_markdown(&artifact))?;
    println!("Wrote {}", common::relative_path(root, &json_path));
    Ok(Some(artifact))
}

fn run_patina(
    cwd: &Path,
    project: &Value,
    args: &Args,
    launch: &Launch,
    expected_file_count: usize,
) -> Result<Vec<Value>, String> {
    let mut command_args = launch.prefix.clone();
    command_args.push("lint".to_string());
    command_args.extend(project_string_array(project, "vueGlobs")?);
    command_args.extend([
        "--format".to_string(),
        "json".to_string(),
        "--no-config".to_string(),
    ]);
    if let Some(preset) = &args.preset {
        command_args.extend(["--preset".to_string(), preset.clone()]);
    }
    let output = Command::new(&launch.command)
        .args(&command_args)
        .current_dir(cwd)
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("vize lint failed to run: {error}"))?;
    let status = output.status.code().unwrap_or(1);
    if status != 0 && status != 1 {
        return Err(format!(
            "vize lint exited with unsupported status {status}: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let envelope = serde_json::from_slice::<Value>(&output.stdout)
        .map_err(|error| format!("vize lint produced invalid JSON: {error}"))?;
    let entries = envelope
        .as_array()
        .ok_or_else(|| "vize lint envelope must be an array".to_string())?;
    if entries.len() != expected_file_count {
        return Err(format!(
            "{}: vize lint reported {} files, expected {expected_file_count}",
            project_string(project, "id")?,
            entries.len()
        ));
    }
    let mut findings = Vec::new();
    for entry in entries {
        let file = entry
            .get("file")
            .and_then(Value::as_str)
            .ok_or_else(|| "vize lint entry must carry file".to_string())?;
        for message in entry
            .get("messages")
            .and_then(Value::as_array)
            .ok_or_else(|| "vize lint entry must carry messages".to_string())?
        {
            let mut finding = message.clone();
            finding["file"] = json!(file);
            findings.push(finding);
        }
    }
    Ok(findings)
}

struct BaselineRun {
    version: String,
    results: Vec<Value>,
    dropped_config_message_count: u64,
}

fn run_eslint_baseline(
    root: &Path,
    cwd: &Path,
    files: &[String],
    rules: &BTreeMap<String, String>,
) -> Result<BaselineRun, String> {
    if rules.is_empty() {
        return Ok(BaselineRun {
            version: rule_map_version(root)?,
            results: files
                .iter()
                .map(|file| json!({ "filePath": cwd.join(file), "messages": [] }))
                .collect(),
            dropped_config_message_count: 0,
        });
    }
    let script = r#"
import { createRequire } from "node:module";
import { readFileSync } from "node:fs";
const input = JSON.parse(readFileSync(0, "utf8"));
const requireFromBench = createRequire(input.benchPackageJson);
const ESLint = requireFromBench("eslint").ESLint;
const plugin = requireFromBench("eslint-plugin-vue");
const vueParser = requireFromBench("vue-eslint-parser");
const scriptParser = requireFromBench("@typescript-eslint/parser");
const manifest = requireFromBench("eslint-plugin-vue/package.json");
const eslint = new ESLint({
  cwd: input.cwd,
  overrideConfigFile: true,
  overrideConfig: [{
    files: ["**/*.vue"],
    languageOptions: {
      parser: vueParser,
      parserOptions: {
        parser: scriptParser,
        ecmaVersion: "latest",
        sourceType: "module",
        ecmaFeatures: { jsx: true },
        extraFileExtensions: [".vue"]
      }
    },
    linterOptions: { reportUnusedDisableDirectives: "off" },
    plugins: { vue: plugin },
    rules: input.rules
  }],
  errorOnUnmatchedPattern: false
});
const results = await eslint.lintFiles(input.files);
const enabled = new Set(Object.keys(input.rules));
let droppedConfigMessageCount = 0;
const retained = results.map((result) => ({
  ...result,
  messages: result.messages.filter((message) => {
    if (message.ruleId == null || enabled.has(message.ruleId)) return true;
    droppedConfigMessageCount += 1;
    return false;
  })
}));
process.stdout.write(JSON.stringify({ version: manifest.version, results: retained, droppedConfigMessageCount }));
"#;
    let input = json!({
        "benchPackageJson": root.join("tools/benchmarks/scripts/package.json"),
        "cwd": cwd,
        "files": files,
        "rules": rules,
    });
    let mut child = Command::new("node")
        .args(["--input-type=module", "--eval", script])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("eslint baseline failed to start: {error}"))?;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| "eslint baseline stdin is unavailable".to_string())?
        .write_all(input.to_string().as_bytes())
        .map_err(|error| format!("cannot write eslint baseline input: {error}"))?;
    let output = child
        .wait_with_output()
        .map_err(|error| format!("eslint baseline failed: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "eslint baseline exited {}:\n{}{}",
            output.status.code().unwrap_or(1),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let parsed = serde_json::from_slice::<Value>(&output.stdout)
        .map_err(|error| format!("eslint baseline produced invalid JSON: {error}"))?;
    Ok(BaselineRun {
        version: parsed
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        results: parsed
            .get("results")
            .and_then(Value::as_array)
            .cloned()
            .ok_or_else(|| "eslint baseline result must list results".to_string())?,
        dropped_config_message_count: parsed
            .get("droppedConfigMessageCount")
            .and_then(Value::as_u64)
            .unwrap_or(0),
    })
}

fn compare_lint_findings(
    project_id: &str,
    cwd: &Path,
    rule_map: &Value,
    patina_findings: &[Value],
    eslint_results: &[Value],
) -> Result<Value, String> {
    let index = RuleMapIndex::new(rule_map)?;
    let patina = collect_patina_findings(patina_findings, cwd)?;
    let baseline_input = collect_baseline_findings(eslint_results, cwd)?;

    let mut unimplemented = Vec::new();
    let mut intentional_divergences = Vec::new();
    let mut comparable_baseline = Vec::new();
    for finding in baseline_input.findings {
        let entry = index.by_upstream.get(&finding.rule_id).ok_or_else(|| {
            format!(
                "baseline rule {} is absent from the pinned rule map",
                finding.rule_id
            )
        })?;
        match entry.get("status").and_then(Value::as_str).unwrap_or("") {
            "unimplemented" => {
                let mut value = finding.to_value();
                value["issue"] = entry.get("issue").cloned().unwrap_or(Value::Null);
                unimplemented.push(value);
            }
            "intentional-divergence" => {
                let mut value = finding.to_value();
                value["reason"] = entry.get("reason").cloned().unwrap_or(Value::Null);
                intentional_divergences.push(value);
            }
            "mapped" => {
                let mut mapped = finding.clone();
                mapped.upstream_rule_id = Some(finding.rule_id.clone());
                mapped.rule_id = entry
                    .get("patinaRule")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        format!("{} must name the patina rule it maps to", finding.rule_id)
                    })?
                    .to_string();
                comparable_baseline.push(mapped);
            }
            other => {
                return Err(format!(
                    "{} has unsupported status {other:?}",
                    finding.rule_id
                ));
            }
        }
    }

    let mut comparable_patina = Vec::new();
    let mut patina_only_rule_findings = Vec::new();
    for finding in patina {
        if index.patina_targets.contains(&finding.rule_id) {
            comparable_patina.push(finding);
        } else {
            patina_only_rule_findings.push(finding.to_value());
        }
    }

    let patina_groups = group_by_identity(&comparable_patina);
    let baseline_groups = group_by_identity(&comparable_baseline);
    let identities = patina_groups
        .keys()
        .chain(baseline_groups.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut shared = Vec::new();
    let mut message_differences = Vec::new();
    let mut false_positives = Vec::new();
    let mut false_negatives = Vec::new();
    for identity in identities {
        let candidates = patina_groups.get(&identity).cloned().unwrap_or_default();
        let expected = baseline_groups.get(&identity).cloned().unwrap_or_default();
        let common_count = candidates.len().min(expected.len());
        for index in 0..common_count {
            let candidate = &candidates[index];
            let expected = &expected[index];
            let pair = json!({
                "file": candidate.file,
                "ruleId": candidate.rule_id,
                "upstreamRuleId": expected.upstream_rule_id,
                "severity": candidate.severity,
                "line": candidate.line,
                "column": candidate.column,
                "endLine": candidate.end_line,
                "endColumn": candidate.end_column,
                "patinaMessage": candidate.message,
                "baselineMessage": expected.message,
            });
            if candidate.message != expected.message {
                message_differences.push(pair.clone());
            }
            shared.push(pair);
        }
        false_positives.extend(candidates.into_iter().skip(common_count));
        false_negatives.extend(expected.into_iter().skip(common_count));
    }

    let documented_divergences = Vec::<Value>::new();
    let rule_location_divergences =
        pair_rule_location_divergences(&mut false_positives, &mut false_negatives);
    sort_values(&mut shared);
    sort_values(&mut message_differences);
    false_positives.sort();
    false_negatives.sort();
    let false_positive_values = false_positives
        .iter()
        .map(LintRecord::to_value)
        .collect::<Vec<_>>();
    let false_negative_values = false_negatives
        .iter()
        .map(LintRecord::to_value)
        .collect::<Vec<_>>();
    sort_values(&mut unimplemented);
    sort_values(&mut intentional_divergences);
    sort_values(&mut patina_only_rule_findings);

    let summary = json!({
        "patinaFindingCount": comparable_patina.len() + patina_only_rule_findings.len(),
        "baselineFindingCount": comparable_baseline.len() + unimplemented.len() + intentional_divergences.len(),
        "comparableBaselineCount": comparable_baseline.len(),
        "sharedCount": shared.len(),
        "messageDifferenceCount": message_differences.len(),
        "documentedDivergenceCount": documented_divergences.len(),
        "ruleLocationDivergenceCount": rule_location_divergences.len(),
        "falsePositiveCount": false_positive_values.len(),
        "falseNegativeCount": false_negative_values.len(),
        "unimplementedCount": unimplemented.len(),
        "intentionalDivergenceCount": intentional_divergences.len(),
        "patinaOnlyRuleFindingCount": patina_only_rule_findings.len(),
        "baselineParseErrorCount": baseline_input.parse_error_count,
        "baselineExcludedNonVueCount": baseline_input.excluded_non_vue_count,
        "baselineInvalidRangeCount": baseline_input.invalid_range_count,
        "falsePositiveRatio": ratio(false_positive_values.len(), comparable_patina.len()),
        "falseNegativeRatio": ratio(false_negative_values.len(), comparable_baseline.len()),
    });
    let classified = json!({
        "shared": shared,
        "messageDifferences": message_differences,
        "falsePositives": false_positive_values,
        "falseNegatives": false_negative_values,
        "ruleLocationDivergences": rule_location_divergences,
        "unimplemented": unimplemented,
        "intentionalDivergences": intentional_divergences,
        "patinaOnlyRuleFindings": patina_only_rule_findings,
        "documentedDivergences": documented_divergences,
    });
    let hash_input = json!({
        "summary": summary,
        "shared": classified["shared"],
        "messageDifferences": classified["messageDifferences"],
        "falsePositives": classified["falsePositives"],
        "falseNegatives": classified["falseNegatives"],
        "ruleLocationDivergences": classified["ruleLocationDivergences"],
        "unimplemented": classified["unimplemented"],
        "intentionalDivergences": classified["intentionalDivergences"],
        "patinaOnlyRuleFindings": classified["patinaOnlyRuleFindings"],
        "documentedDivergences": classified["documentedDivergences"],
    });
    Ok(json!({
        "schema": "vize.fixtureLintDivergence",
        "version": 1,
        "project": project_id,
        "upstream": {
            "package": rule_map.pointer("/upstream/package").cloned().unwrap_or(Value::Null),
            "version": rule_map.pointer("/upstream/version").cloned().unwrap_or(Value::Null),
        },
        "summary": summary,
        "shared": classified["shared"],
        "messageDifferences": classified["messageDifferences"],
        "falsePositives": classified["falsePositives"],
        "falseNegatives": classified["falseNegatives"],
        "ruleLocationDivergences": classified["ruleLocationDivergences"],
        "unimplemented": classified["unimplemented"],
        "intentionalDivergences": classified["intentionalDivergences"],
        "patinaOnlyRuleFindings": classified["patinaOnlyRuleFindings"],
        "documentedDivergences": classified["documentedDivergences"],
        "sha256": sha256(&hash_input.to_string()),
    }))
}

struct BaselineInput {
    findings: Vec<LintRecord>,
    parse_error_count: u64,
    excluded_non_vue_count: u64,
    invalid_range_count: u64,
}

struct ComparableRules {
    rules: BTreeMap<String, String>,
    mapped_rule_count: usize,
    skipped_by_preset_count: usize,
}

struct RuleMapIndex {
    by_upstream: BTreeMap<String, Value>,
    patina_targets: BTreeSet<String>,
}

impl RuleMapIndex {
    fn new(rule_map: &Value) -> Result<Self, String> {
        let entries = rule_map
            .get("entries")
            .and_then(Value::as_object)
            .ok_or_else(|| "rule map must carry entries".to_string())?;
        let mut by_upstream = BTreeMap::new();
        let mut patina_targets = BTreeSet::new();
        for (rule_id, entry) in entries {
            if entry.get("status").and_then(Value::as_str) == Some("mapped") {
                let patina_rule = entry
                    .get("patinaRule")
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("{rule_id} must name the patina rule it maps to"))?;
                patina_targets.insert(patina_rule.to_string());
            }
            by_upstream.insert(rule_id.clone(), entry.clone());
        }
        Ok(Self {
            by_upstream,
            patina_targets,
        })
    }
}

impl LintRecord {
    fn identity(&self) -> String {
        format!(
            "{}\0{}\0{}\0{}\0{}\0{}\0{}",
            self.file,
            self.rule_id,
            self.severity,
            self.line,
            self.column,
            self.end_line,
            self.end_column
        )
    }

    fn to_value(&self) -> Value {
        let mut value = json!({
            "file": self.file,
            "ruleId": self.rule_id,
            "severity": self.severity,
            "line": self.line,
            "column": self.column,
            "endLine": self.end_line,
            "endColumn": self.end_column,
            "message": self.message,
        });
        if let Some(upstream_rule_id) = &self.upstream_rule_id {
            value["upstreamRuleId"] = json!(upstream_rule_id);
        }
        value
    }
}

impl PartialEq for LintRecord {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}
impl Eq for LintRecord {}
impl PartialOrd for LintRecord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for LintRecord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.file
            .cmp(&other.file)
            .then_with(|| self.line.cmp(&other.line))
            .then_with(|| self.column.cmp(&other.column))
            .then_with(|| self.end_line.cmp(&other.end_line))
            .then_with(|| self.end_column.cmp(&other.end_column))
            .then_with(|| self.rule_id.cmp(&other.rule_id))
            .then_with(|| self.severity.cmp(&other.severity))
            .then_with(|| self.message.cmp(&other.message))
    }
}

fn select_comparable_rules(
    rule_map: &Value,
    preset: Option<&str>,
    include_unimplemented: bool,
) -> Result<ComparableRules, String> {
    let entries = rule_map
        .get("entries")
        .and_then(Value::as_object)
        .ok_or_else(|| "rule map must carry entries".to_string())?;
    let mut rules = BTreeMap::new();
    let mut mapped_rule_count = 0usize;
    let mut skipped_by_preset_count = 0usize;
    for (upstream_rule, entry) in entries {
        match entry.get("status").and_then(Value::as_str) {
            Some("unimplemented") => {
                if include_unimplemented {
                    rules.insert(upstream_rule.clone(), "warn".to_string());
                }
            }
            Some("mapped") => {
                mapped_rule_count += 1;
                let patina_rule = entry
                    .get("patinaRule")
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("{upstream_rule} must name patinaRule"))?;
                if let Some(preset) = preset {
                    let enabled = entry
                        .get("patinaPresets")
                        .and_then(Value::as_array)
                        .is_some_and(|items| {
                            items.iter().any(|item| item.as_str() == Some(preset))
                        });
                    if !enabled {
                        let _ = patina_rule;
                        skipped_by_preset_count += 1;
                        continue;
                    }
                }
                let severity = match entry.get("patinaSeverity").and_then(Value::as_str) {
                    Some("error") => "error",
                    _ => "warn",
                };
                rules.insert(upstream_rule.clone(), severity.to_string());
            }
            Some("intentional-divergence") => {}
            Some(other) => return Err(format!("{upstream_rule} has unsupported status {other:?}")),
            None => return Err(format!("{upstream_rule} must record status")),
        }
    }
    Ok(ComparableRules {
        rules,
        mapped_rule_count,
        skipped_by_preset_count,
    })
}

fn collect_patina_findings(findings: &[Value], cwd: &Path) -> Result<Vec<LintRecord>, String> {
    findings
        .iter()
        .enumerate()
        .map(|(index, finding)| {
            let label = format!("patina findings[{index}]");
            let file = normalize_path(required_str(finding, "file", &label)?, cwd, "file")?;
            let rule_id = required_str(finding, "ruleId", &label)?.to_string();
            let severity = severity(finding.get("severity"), &label)?;
            let line = required_u64(finding, "line", &label)?;
            let column = required_u64(finding, "column", &label)?;
            let end_line = finding
                .get("endLine")
                .and_then(Value::as_u64)
                .unwrap_or(line);
            let end_column = finding
                .get("endColumn")
                .and_then(Value::as_u64)
                .unwrap_or(column);
            let message = normalize_message(required_str(finding, "message", &label)?);
            validate_range(&file, &rule_id, line, column, end_line, end_column)?;
            Ok(LintRecord {
                file,
                rule_id,
                upstream_rule_id: None,
                severity,
                line,
                column,
                end_line,
                end_column,
                message,
            })
        })
        .collect()
}

fn collect_baseline_findings(results: &[Value], cwd: &Path) -> Result<BaselineInput, String> {
    let mut findings = Vec::new();
    let mut parse_error_count = 0;
    let mut excluded_non_vue_count = 0;
    let mut invalid_range_count = 0;
    for (index, result) in results.iter().enumerate() {
        let label = format!("eslint results[{index}]");
        let file = normalize_path(required_str(result, "filePath", &label)?, cwd, "filePath")?;
        let messages = result
            .get("messages")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{label} must carry messages"))?;
        for message in messages {
            if message.get("ruleId").is_none_or(Value::is_null) {
                parse_error_count += 1;
                continue;
            }
            if !file.ends_with(".vue") {
                excluded_non_vue_count += 1;
                continue;
            }
            let rule_id = required_str(message, "ruleId", &label)?.to_string();
            let severity = severity(message.get("severity"), &label)?;
            let line = required_u64(message, "line", &label)?;
            let column = required_u64(message, "column", &label)?;
            let end_line = message
                .get("endLine")
                .and_then(Value::as_u64)
                .unwrap_or(line);
            let end_column = message
                .get("endColumn")
                .and_then(Value::as_u64)
                .unwrap_or(column);
            if validate_range(&file, &rule_id, line, column, end_line, end_column).is_err() {
                invalid_range_count += 1;
                continue;
            }
            findings.push(LintRecord {
                file: file.clone(),
                rule_id,
                upstream_rule_id: None,
                severity,
                line,
                column,
                end_line,
                end_column,
                message: normalize_message(required_str(message, "message", &label)?),
            });
        }
    }
    Ok(BaselineInput {
        findings,
        parse_error_count,
        excluded_non_vue_count,
        invalid_range_count,
    })
}

fn group_by_identity(records: &[LintRecord]) -> BTreeMap<String, Vec<LintRecord>> {
    let mut groups = BTreeMap::<String, Vec<LintRecord>>::new();
    for record in records {
        groups
            .entry(record.identity())
            .or_default()
            .push(record.clone());
    }
    for group in groups.values_mut() {
        group.sort();
    }
    groups
}

fn pair_rule_location_divergences(
    false_positives: &mut Vec<LintRecord>,
    false_negatives: &mut Vec<LintRecord>,
) -> Vec<Value> {
    let mut paired = Vec::new();
    let mut positive_index = 0;
    while positive_index < false_positives.len() {
        let positive = false_positives[positive_index].clone();
        let Some((negative_index, subject, reason)) =
            find_rule_location_divergence(&positive, false_negatives)
        else {
            positive_index += 1;
            continue;
        };
        let negative = false_negatives.remove(negative_index);
        false_positives.remove(positive_index);
        paired.push(json!({
            "file": positive.file,
            "ruleId": positive.rule_id,
            "upstreamRuleId": negative.upstream_rule_id,
            "severity": positive.severity,
            "subject": subject,
            "reason": reason,
            "patina": side(&positive),
            "baseline": side(&negative),
        }));
    }
    sort_values(&mut paired);
    paired
}

fn find_rule_location_divergence(
    positive: &LintRecord,
    false_negatives: &[LintRecord],
) -> Option<(usize, String, &'static str)> {
    for (negative_index, negative) in false_negatives.iter().enumerate() {
        if positive.file != negative.file
            || positive.rule_id != negative.rule_id
            || positive.severity != negative.severity
        {
            continue;
        }
        if positive.rule_id == "vue/no-unused-components" {
            let component = unused_component_name(&positive.message)?;
            if Some(component.as_str()) == unused_component_name(&negative.message).as_deref() {
                return Some((
                    negative_index,
                    format!("component:{component}"),
                    "patina reports the SFC/script anchor while eslint-plugin-vue reports the component registration property.",
                ));
            }
        }
        if positive.rule_id == "vue/require-v-for-key" && contains_span(negative, positive) {
            return Some((
                negative_index,
                "missing-v-for-key".to_string(),
                "patina reports the v-for directive range while eslint-plugin-vue reports the owning element range.",
            ));
        }
    }
    None
}

fn write_index(
    root: &Path,
    args: &Args,
    evidence: &Value,
    artifacts: &[Value],
) -> Result<(), String> {
    let summary = json!({
        "schema": "vize.fixtureLintDivergenceIndex",
        "version": 1,
        "evidence": evidence,
        "preset": args.preset.clone().unwrap_or_else(|| "all-mapped".to_string()),
        "projectCount": artifacts.len(),
        "budget": summarize_budgets(artifacts),
        "totals": sum_totals(artifacts),
        "projects": artifacts.iter().map(|artifact| {
            let mut project = serde_json::Map::new();
            project.insert("project".to_string(), artifact["project"].clone());
            if let Some(summary) = artifact.pointer("/divergence/summary").and_then(Value::as_object) {
                for (key, value) in summary {
                    project.insert(key.clone(), value.clone());
                }
            }
            Value::Object(project)
        }).collect::<Vec<_>>(),
    });
    let path = args.output_dir.join("lint-divergence-summary.json");
    common::write_json_pretty(&path, &summary)?;
    println!("Wrote {}", common::relative_path(root, &path));
    Ok(())
}

fn evaluate_budget(artifact: &Value) -> Value {
    let summary = &artifact["divergence"]["summary"];
    let unusable_reason = unusable_lint_reason(artifact);
    let false_positive_passed = summary["falsePositiveCount"].as_u64().unwrap_or(0) == 0;
    let false_negative_passed = summary["falseNegativeCount"].as_u64().unwrap_or(0) == 0;
    let verdict = if unusable_reason.is_some() {
        "unusable"
    } else if false_positive_passed && false_negative_passed {
        "passed"
    } else {
        "breached"
    };
    json!({
        "maxFalsePositiveCount": 0,
        "maxFalseNegativeCount": 0,
        "falsePositivePassed": false_positive_passed,
        "falseNegativePassed": false_negative_passed,
        "unusableReason": unusable_reason,
        "verdict": verdict,
        "passed": verdict == "passed",
    })
}

fn assert_budgets_passed(artifacts: &[Value], mode: &str) -> Result<(), String> {
    if mode != "enforce" && mode != "record-only" {
        return Err("--budget-mode must be one of: enforce, record-only".to_string());
    }
    if artifacts.is_empty() {
        let detail = "Lint divergence budget has no measured projects";
        if mode == "enforce" {
            return Err(detail.to_string());
        }
        println!("::warning title=Lint divergence budget not enforced::{detail}");
        return Ok(());
    }
    let failures = artifacts
        .iter()
        .filter(|artifact| {
            artifact.pointer("/budget/passed").and_then(Value::as_bool) != Some(true)
        })
        .collect::<Vec<_>>();
    if failures.is_empty() {
        return Ok(());
    }
    let details = failures
        .iter()
        .map(|artifact| describe_failure(artifact))
        .collect::<Vec<_>>();
    if mode == "enforce" {
        Err(format!(
            "Lint divergence budget failed for {} project(s):\n{}",
            failures.len(),
            details.join("\n")
        ))
    } else {
        for detail in details {
            println!("::warning title=Lint divergence budget not enforced::{detail}");
        }
        Ok(())
    }
}

fn summarize_budgets(artifacts: &[Value]) -> Value {
    let failures = artifacts
        .iter()
        .filter(|artifact| {
            artifact.pointer("/budget/passed").and_then(Value::as_bool) != Some(true)
        })
        .collect::<Vec<_>>();
    let unusable = artifacts
        .iter()
        .filter(|artifact| {
            artifact.pointer("/budget/verdict").and_then(Value::as_str) == Some("unusable")
        })
        .count();
    let breached = artifacts
        .iter()
        .filter(|artifact| {
            artifact.pointer("/budget/verdict").and_then(Value::as_str) == Some("breached")
        })
        .count();
    json!({
        "status": if !artifacts.is_empty() && failures.is_empty() { "success" } else { "failure" },
        "passed": !artifacts.is_empty() && failures.is_empty(),
        "projectCount": artifacts.len(),
        "passedCount": artifacts.len() - failures.len(),
        "failedCount": failures.len(),
        "unusableCount": unusable,
        "breachedCount": breached,
        "failedProjects": failures.iter().map(|artifact| artifact["project"].clone()).collect::<Vec<_>>(),
    })
}

fn sum_totals(artifacts: &[Value]) -> Value {
    let keys = [
        "patinaFindingCount",
        "baselineFindingCount",
        "comparableBaselineCount",
        "sharedCount",
        "messageDifferenceCount",
        "documentedDivergenceCount",
        "ruleLocationDivergenceCount",
        "falsePositiveCount",
        "falseNegativeCount",
        "unimplementedCount",
        "intentionalDivergenceCount",
        "patinaOnlyRuleFindingCount",
        "baselineParseErrorCount",
        "baselineInvalidRangeCount",
    ];
    let mut totals = serde_json::Map::new();
    for key in keys {
        let total = artifacts
            .iter()
            .map(|artifact| {
                artifact
                    .pointer(&format!("/divergence/summary/{key}"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0)
            })
            .sum::<u64>();
        totals.insert(key.to_string(), json!(total));
    }
    Value::Object(totals)
}

fn render_markdown(artifact: &Value) -> String {
    let summary = &artifact["divergence"]["summary"];
    let mut lines = vec![
        format!(
            "## {} lint divergence",
            artifact["project"].as_str().unwrap_or("-")
        ),
        String::new(),
        format!(
            "Commit: {}",
            artifact
                .pointer("/evidence/commitSha")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        ),
        format!(
            "Revision: {}",
            artifact["revision"].as_str().unwrap_or("unknown")
        ),
        format!(
            "Preset: {}",
            artifact["preset"].as_str().unwrap_or("unknown")
        ),
        format!(
            "Baseline: {} {}",
            artifact
                .pointer("/baseline/package")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            artifact
                .pointer("/baseline/version")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        ),
        format!(
            "Compared rules: {} of {} mapped",
            artifact
                .pointer("/baseline/comparedRuleCount")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            artifact
                .pointer("/baseline/mappedRuleCount")
                .and_then(Value::as_u64)
                .unwrap_or(0)
        ),
        format!(
            "Files: {}",
            artifact
                .pointer("/files/comparedCount")
                .and_then(Value::as_u64)
                .unwrap_or(0)
        ),
        format!(
            "Baseline messages dropped as foreign-rule directives: {}",
            artifact
                .pointer("/baseline/droppedConfigMessageCount")
                .and_then(Value::as_u64)
                .unwrap_or(0)
        ),
        String::new(),
        format!("Patina findings: {}", summary["patinaFindingCount"]),
        format!("Baseline findings: {}", summary["baselineFindingCount"]),
        format!(
            "Comparable baseline findings: {}",
            summary["comparableBaselineCount"]
        ),
        format!("Shared: {}", summary["sharedCount"]),
        format!("Message differences: {}", summary["messageDifferenceCount"]),
        format!(
            "Documented divergences: {}",
            summary["documentedDivergenceCount"]
        ),
        format!(
            "Rule location divergences: {}",
            summary["ruleLocationDivergenceCount"]
        ),
        format!(
            "False positives: {} ({})",
            summary["falsePositiveCount"], summary["falsePositiveRatio"]
        ),
        format!(
            "False negatives: {} ({})",
            summary["falseNegativeCount"], summary["falseNegativeRatio"]
        ),
        format!(
            "Unimplemented upstream findings: {}",
            summary["unimplementedCount"]
        ),
        format!(
            "Intentional divergences: {}",
            summary["intentionalDivergenceCount"]
        ),
        format!(
            "Patina-only rule findings: {}",
            summary["patinaOnlyRuleFindingCount"]
        ),
        format!(
            "Baseline parse errors: {}",
            summary["baselineParseErrorCount"]
        ),
        format!(
            "Baseline invalid ranges: {}",
            summary["baselineInvalidRangeCount"]
        ),
        format!(
            "Budget verdict: {}",
            artifact
                .pointer("/budget/verdict")
                .and_then(Value::as_str)
                .unwrap_or("not-evaluated")
        ),
        format!(
            "Budget passed: {}",
            artifact
                .pointer("/budget/passed")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        ),
        String::new(),
    ];
    if artifact
        .pointer("/baseline/comparedRuleCount")
        .and_then(Value::as_u64)
        == Some(0)
    {
        lines.push(
            "> No mapped rule was comparable under this preset: nothing was measured.".to_string(),
        );
        lines.push(String::new());
    }
    lines.extend(rule_table(
        "False positives",
        artifact
            .pointer("/divergence/falsePositives")
            .and_then(Value::as_array),
    ));
    lines.extend(rule_table(
        "False negatives",
        artifact
            .pointer("/divergence/falseNegatives")
            .and_then(Value::as_array),
    ));
    lines.extend(rule_table(
        "Rule location divergences",
        artifact
            .pointer("/divergence/ruleLocationDivergences")
            .and_then(Value::as_array),
    ));
    lines.extend(rule_table(
        "Unimplemented upstream rules",
        artifact
            .pointer("/divergence/unimplemented")
            .and_then(Value::as_array),
    ));
    format!("{}\n", lines.join("\n"))
}

fn rule_table(title: &str, findings: Option<&Vec<Value>>) -> Vec<String> {
    let findings = findings.cloned().unwrap_or_default();
    if findings.is_empty() {
        return vec![format!("### {title}: none"), String::new()];
    }
    let mut counts = BTreeMap::<String, usize>::new();
    for finding in &findings {
        let key = finding
            .get("upstreamRuleId")
            .or_else(|| finding.get("ruleId"))
            .and_then(Value::as_str)
            .unwrap_or("-");
        *counts.entry(key.to_string()).or_default() += 1;
    }
    let mut rows = counts.into_iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    [
        vec![
            format!("### {title}: {}", findings.len()),
            String::new(),
            "| Rule | Findings |".to_string(),
            "| --- | ---: |".to_string(),
        ],
        rows.into_iter()
            .map(|(rule, count)| format!("| `{rule}` | {count} |"))
            .collect::<Vec<_>>(),
        vec![String::new()],
    ]
    .concat()
}

fn selected_projects<'a>(registry: &'a Value, args: &Args) -> Result<Vec<&'a Value>, String> {
    let projects = registry
        .get("projects")
        .and_then(Value::as_array)
        .ok_or_else(|| "registry must list projects".to_string())?;
    let mut selected = projects
        .iter()
        .filter(|project| project_covers(project, "linter"))
        .filter(|project| {
            args.projects.is_empty()
                || project
                    .get("id")
                    .and_then(Value::as_str)
                    .is_some_and(|id| args.projects.iter().any(|selected| selected == id))
        })
        .collect::<Vec<_>>();
    selected = selected
        .into_iter()
        .enumerate()
        .filter_map(|(index, project)| {
            (index % args.shard_count == args.shard_index).then_some(project)
        })
        .collect();
    Ok(selected)
}

fn parse_args(argv: Vec<String>, root: &Path) -> Result<Args, String> {
    let mut args = Args {
        measure_coverage_gap: false,
        budget_mode: "enforce".to_string(),
        output_dir: root.join(".vize/lint-divergence"),
        preset: Some(DEFAULT_PRESET.to_string()),
        projects: Vec::new(),
        registry: root.join("tests/_fixtures/vue-ecosystem-fixtures.json"),
        shard_count: 1,
        shard_index: 0,
        timeout_ms: 600_000,
        vize_bin: None,
    };
    let mut index = 0;
    while index < argv.len() {
        let arg = &argv[index];
        let value = |index: &mut usize| -> Result<String, String> {
            *index += 1;
            argv.get(*index)
                .cloned()
                .ok_or_else(|| format!("{arg} requires a value"))
        };
        match arg.as_str() {
            "--output-dir" => args.output_dir = absolutize(root, PathBuf::from(value(&mut index)?)),
            "--budget-mode" => args.budget_mode = parse_budget_mode(&value(&mut index)?)?,
            "--preset" => args.preset = Some(value(&mut index)?),
            "--all-mapped-rules" => args.preset = None,
            "--measure-coverage-gap" => args.measure_coverage_gap = true,
            "--project" => args.projects.extend(split_csv(&value(&mut index)?)),
            "--registry" => args.registry = absolutize(root, PathBuf::from(value(&mut index)?)),
            "--shard-count" => {
                args.shard_count = positive_integer(&value(&mut index)?, arg)? as usize
            }
            "--shard-index" => {
                args.shard_index = non_negative_integer(&value(&mut index)?, arg)? as usize
            }
            "--timeout-ms" => args.timeout_ms = positive_integer(&value(&mut index)?, arg)?,
            "--vize-bin" => args.vize_bin = Some(PathBuf::from(value(&mut index)?)),
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => return Err(format!("Unknown argument: {arg}")),
        }
        index += 1;
    }
    if args.shard_index >= args.shard_count {
        return Err("--shard-index must be less than --shard-count".to_string());
    }
    Ok(args)
}

fn print_help() {
    println!(
        "Usage: rust-script tools/commands/fixtures/lint-divergence-report.rs [options]\n\
\n\
Classify vize lint against eslint-plugin-vue over the pinned real projects.\n\
\n\
  --project <id[,id]>     Limit registry projects\n\
  --preset <name>         Patina preset under test (default: ecosystem)\n\
  --all-mapped-rules      Compare every mapped rule, ignoring preset membership\n\
  --measure-coverage-gap  Also enable unimplemented upstream rules\n\
  --shard-index <n>       Zero-based project shard index\n\
  --shard-count <n>       Total balanced project shards\n\
  --output-dir <dir>      Report directory\n\
  --budget-mode <mode>    enforce or record-only (default: enforce)\n\
  --vize-bin <path>       Vize executable\n\
  --timeout-ms <n>        Per-project vize lint timeout"
    );
}

fn collect_vue_input_paths(cwd: &Path, project: &Value) -> Result<Vec<String>, String> {
    let mut files = BTreeSet::new();
    for pattern in project_string_array(project, "vueGlobs")? {
        let absolute_pattern = cwd.join(pattern).to_string_lossy().into_owned();
        for entry in glob::glob(&absolute_pattern).map_err(|error| error.to_string())? {
            let path = entry.map_err(|error| error.to_string())?;
            if path.is_file() {
                files.insert(common::relative_path(cwd, &path));
            }
        }
    }
    Ok(files.into_iter().collect())
}

fn reconcile_corpus(
    project_id: &str,
    files: &[String],
    eslint_results: &[Value],
    cwd: &Path,
) -> Result<(), String> {
    let linted = eslint_results
        .iter()
        .filter_map(|result| result.get("filePath").and_then(Value::as_str))
        .map(|file| normalize_path(file, cwd, "filePath"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let missing = files
        .iter()
        .filter(|file| !linted.contains(*file))
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "{project_id}: baseline skipped {} of {} files, starting with {}",
            missing.len(),
            files.len(),
            missing[0]
        ));
    }
    Ok(())
}

fn resolve_vize_launch(root: &Path, vize_bin: Option<&Path>) -> Result<Launch, String> {
    let executable = if env::consts::OS == "windows" {
        "vize.exe"
    } else {
        "vize"
    };
    let mut candidates = Vec::new();
    if let Some(vize_bin) = vize_bin {
        candidates.push(if vize_bin.is_absolute() {
            vize_bin.to_path_buf()
        } else {
            root.join(vize_bin)
        });
    }
    if let Some(env_bin) = env::var_os("VIZE_BIN") {
        let env_bin = PathBuf::from(env_bin);
        candidates.push(if env_bin.is_absolute() {
            env_bin
        } else {
            root.join(env_bin)
        });
    }
    candidates.extend([
        root.join("target/ci").join(executable),
        root.join("target/debug").join(executable),
        root.join("target/release").join(executable),
    ]);
    for candidate in candidates {
        if candidate.exists() {
            let resolved = candidate.canonicalize().unwrap_or(candidate);
            return Ok(Launch {
                command: resolved.display().to_string(),
                prefix: Vec::new(),
            });
        }
    }
    Ok(Launch {
        command: "cargo".to_string(),
        prefix: vec![
            "run".to_string(),
            "-q".to_string(),
            "-p".to_string(),
            "vize".to_string(),
            "--".to_string(),
        ],
    })
}

fn collect_run_evidence(root: &Path) -> Result<Value, String> {
    let commit_sha = env::var("GITHUB_SHA").unwrap_or_else(|_| {
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(root)
            .stdin(Stdio::null())
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|value| value.trim().to_string())
            .unwrap_or_default()
    });
    Ok(json!({
        "commitSha": commit_sha,
        "runtime": { "name": "rust-script", "version": rustc_version() },
        "machine": {
            "platform": match env::consts::OS { "macos" => "darwin", other => other },
            "arch": match env::consts::ARCH { "x86_64" => "x64", "aarch64" => "arm64", other => other },
            "logicalCpuCount": std::thread::available_parallelism().map(|value| value.get()).unwrap_or(1),
        },
    }))
}

fn rule_map_version(root: &Path) -> Result<String, String> {
    common::read_json(root.join(RULE_MAP_REL))?
        .pointer("/upstream/version")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "rule map must record upstream version".to_string())
}

fn unusable_lint_reason(artifact: &Value) -> Option<String> {
    if artifact
        .pointer("/files/comparedCount")
        .and_then(Value::as_u64)
        == Some(0)
    {
        return Some("the project selected no Vue files".to_string());
    }
    if artifact
        .pointer("/baseline/comparedRuleCount")
        .and_then(Value::as_u64)
        == Some(0)
    {
        return Some(
            "no mapped eslint-plugin-vue rule was comparable under the selected preset".to_string(),
        );
    }
    let parse_errors = artifact
        .pointer("/divergence/summary/baselineParseErrorCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if parse_errors > 0 {
        return Some(format!(
            "eslint-plugin-vue could not parse {parse_errors} compared file(s)"
        ));
    }
    let invalid_ranges = artifact
        .pointer("/divergence/summary/baselineInvalidRangeCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if invalid_ranges > 0 {
        return Some(format!(
            "eslint-plugin-vue reported {invalid_ranges} finding(s) with invalid source ranges"
        ));
    }
    None
}

fn describe_failure(artifact: &Value) -> String {
    let project = artifact
        .get("project")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let budget = &artifact["budget"];
    let summary = &artifact["divergence"]["summary"];
    if budget.get("verdict").and_then(Value::as_str) == Some("unusable") {
        return format!(
            "Lint divergence baseline is unusable for {project}: {}",
            budget
                .get("unusableReason")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        );
    }
    let mut breaches = Vec::new();
    if budget.get("falsePositivePassed").and_then(Value::as_bool) != Some(true) {
        breaches.push(format!(
            "{} false positives exceed maxFalsePositiveCount 0",
            summary["falsePositiveCount"]
        ));
    }
    if budget.get("falseNegativePassed").and_then(Value::as_bool) != Some(true) {
        breaches.push(format!(
            "{} false negatives exceed maxFalseNegativeCount 0",
            summary["falseNegativeCount"]
        ));
    }
    format!(
        "Lint divergence budget breached for {project}: {}",
        breaches.join("; ")
    )
}

fn normalize_path(value: &str, cwd: &Path, label: &str) -> Result<String, String> {
    let mut normalized = value.replace('\\', "/");
    let path = Path::new(&normalized);
    if path.is_absolute() {
        normalized = common::relative_path(cwd, path);
    }
    if let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped.to_string();
    }
    if normalized.is_empty()
        || normalized.starts_with('/')
        || normalized
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(format!("{label} must stay inside the fixture workspace"));
    }
    Ok(normalized)
}

fn validate_range(
    file: &str,
    rule_id: &str,
    line: u64,
    column: u64,
    end_line: u64,
    end_column: u64,
) -> Result<(), String> {
    if line == 0 || column == 0 || end_line == 0 || end_column == 0 {
        return Err(format!(
            "finding range must be positive safe integers: {file} {rule_id}"
        ));
    }
    if end_line < line || (end_line == line && end_column < column) {
        return Err(format!(
            "finding has an inverted source range: {file} {rule_id}"
        ));
    }
    Ok(())
}

fn side(record: &LintRecord) -> Value {
    json!({
        "line": record.line,
        "column": record.column,
        "endLine": record.end_line,
        "endColumn": record.end_column,
        "message": record.message,
    })
}

fn contains_span(outer: &LintRecord, inner: &LintRecord) -> bool {
    (outer.line, outer.column) <= (inner.line, inner.column)
        && (outer.end_line, outer.end_column) >= (inner.end_line, inner.end_column)
}

fn unused_component_name(message: &str) -> Option<String> {
    let prefix = "Component '";
    if let Some(rest) = message.strip_prefix(prefix) {
        if let Some((name, _)) = rest.split_once("' is registered but never used") {
            return Some(name.to_string());
        }
    }
    let prefix = "The \"";
    if let Some(rest) = message.strip_prefix(prefix) {
        if let Some((name, _)) = rest.split_once("\" component has been registered but not used.") {
            return Some(name.to_string());
        }
    }
    None
}

fn sort_values(values: &mut [Value]) {
    values.sort_by(|left, right| left.to_string().cmp(&right.to_string()));
}

fn ratio(count: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        count as f64 / total as f64
    }
}

fn severity(value: Option<&Value>, label: &str) -> Result<String, String> {
    match value {
        Some(Value::Number(number)) if number.as_u64() == Some(2) => Ok("error".to_string()),
        Some(Value::Number(number)) if number.as_u64() == Some(1) => Ok("warning".to_string()),
        Some(Value::String(value)) if value == "error" => Ok("error".to_string()),
        Some(Value::String(value)) if value == "warning" => Ok("warning".to_string()),
        _ => Err(format!(
            "{label}.severity must be an ESLint severity (1 or 2)"
        )),
    }
}

fn normalize_message(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn required_str<'a>(value: &'a Value, key: &str, label: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{label}.{key} must be non-empty"))
}

fn required_u64(value: &Value, key: &str, label: &str) -> Result<u64, String> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("{label}.{key} must be a positive integer"))
}

fn project_covers(project: &Value, tool: &str) -> bool {
    project
        .get("coverage")
        .and_then(Value::as_array)
        .is_some_and(|values| values.iter().any(|value| value.as_str() == Some(tool)))
}

fn project_string(project: &Value, field: &str) -> Result<String, String> {
    project
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("project is missing {field}"))
}

fn project_string_array(project: &Value, field: &str) -> Result<Vec<String>, String> {
    project
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| {
            format!(
                "{} has no {field}",
                project_string(project, "id").unwrap_or_default()
            )
        })?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("{field} entries must be strings"))
        })
        .collect()
}

fn parse_budget_mode(value: &str) -> Result<String, String> {
    if value == "enforce" || value == "record-only" {
        Ok(value.to_string())
    } else {
        Err("--budget-mode must be one of: enforce, record-only".to_string())
    }
}

fn positive_integer(value: &str, name: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("{name} must be a positive integer"))
}

fn non_negative_integer(value: &str, name: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|_| format!("{name} must be a non-negative integer"))
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

fn absolutize(root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

fn is_hydrated(cwd: &Path) -> bool {
    cwd.is_dir()
        && fs::read_dir(cwd)
            .ok()
            .is_some_and(|mut entries| entries.any(|entry| entry.is_ok()))
}

fn rustc_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn sha256(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
