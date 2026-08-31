#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! chrono = "0.4"
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use serde_json::{Value, json};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

#[path = "../../rust/common.rs"]
mod common;

const SCHEMA: &str = "vize.fixtureCompilerDiffReport";
const DEFAULT_TARGETS: &[&str] = &["dom", "ssr"];

#[derive(Debug)]
struct Args {
    allow_failures: bool,
    dry_run: bool,
    max_files: Option<u64>,
    output_dir: Option<PathBuf>,
    projects: Vec<String>,
    targets: Vec<String>,
    template_syntax: String,
    timeout_ms: u64,
    vize_bin: Option<PathBuf>,
}

#[derive(Clone, Debug)]
struct Launch {
    command: String,
    prefix: Vec<String>,
    label: String,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<u8, String> {
    let root = common::repo_root()?;
    let args = parse_args(env::args().skip(1).collect())?;
    let registry_path = root.join("tests/_fixtures/vue-ecosystem-fixtures.json");
    let registry = common::read_json(&registry_path)?;
    let all_projects = registry
        .get("projects")
        .and_then(Value::as_array)
        .ok_or_else(|| "Fixture registry must list projects".to_string())?;
    let selected_projects = select_projects(all_projects, &args.projects)?;
    let targets = if args.targets.is_empty() {
        DEFAULT_TARGETS
            .iter()
            .map(|target| (*target).to_string())
            .collect::<Vec<_>>()
    } else {
        args.targets.clone()
    };
    let output_dir = args.output_dir.clone().unwrap_or_else(|| {
        root.join(format!(
            ".vize/artifacts/compiler-diff-report/{}",
            chrono::Utc::now()
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                .replace([':', '.'], "-")
        ))
    });
    let output_dir = absolutize(&root, output_dir);
    let launch = resolve_vize_launch(&root, args.vize_bin.as_deref())?;
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("cannot create {}: {error}", output_dir.display()))?;

    let mut report = json!({
        "schema": SCHEMA,
        "version": 1,
        "generatedAt": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        "registryPath": "tests/_fixtures/vue-ecosystem-fixtures.json",
        "command": {
            "vize": launch.label,
            "targets": targets,
            "templateSyntax": args.template_syntax,
            "maxFiles": args.max_files,
            "dryRun": args.dry_run,
        },
        "summary": {
            "projectCount": selected_projects.len(),
            "targetCount": targets.len(),
            "plannedTargets": 0,
            "okTargets": 0,
            "failedTargets": 0,
            "changedFiles": 0,
            "additions": 0,
            "removals": 0,
            "officialErrors": 0,
            "vizeErrors": 0,
        },
        "projects": [],
    });

    let mut projects = Vec::new();
    for project in selected_projects {
        let mut project_report = json!({
            "id": project_string(project, "id")?,
            "displayName": project.get("displayName").cloned().unwrap_or(Value::Null),
            "fixturePath": project_string(project, "fixturePath")?,
            "revision": project.get("revision").cloned().unwrap_or(Value::Null),
            "vueGlobs": project.get("vueGlobs").cloned().unwrap_or_else(|| json!([])),
            "diffMode": project.get("diff").cloned().unwrap_or(Value::Null),
            "targets": [],
        });
        let mut project_targets = Vec::new();
        for target in &targets {
            let target_report =
                run_project_target(&root, project, target, &args, &launch, &output_dir)?;
            bump_summary(&mut report, &target_report);
            project_targets.push(target_report);
        }
        project_report["targets"] = Value::Array(project_targets);
        projects.push(project_report);
    }
    report["projects"] = Value::Array(projects);

    let json_path = output_dir.join("summary.json");
    let markdown_path = output_dir.join("summary.md");
    common::write_json_pretty(&json_path, &report)?;
    common::write_text(&markdown_path, &render_markdown(&report))?;
    println!("Wrote {}", common::relative_path(&root, &json_path));
    println!("Wrote {}", common::relative_path(&root, &markdown_path));

    if report["summary"]["failedTargets"].as_u64().unwrap_or(0) > 0 && !args.allow_failures {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn parse_args(argv: Vec<String>) -> Result<Args, String> {
    let mut args = Args {
        allow_failures: false,
        dry_run: false,
        max_files: None,
        output_dir: None,
        projects: Vec::new(),
        targets: Vec::new(),
        template_syntax: "quirks".to_string(),
        timeout_ms: 300_000,
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
        if arg == "--allow-failures" {
            args.allow_failures = true;
        } else if arg == "--dry-run" {
            args.dry_run = true;
        } else if arg == "--help" || arg == "-h" {
            print_help();
            std::process::exit(0);
        } else if arg == "--max-files" {
            args.max_files = Some(positive_integer(&value(&mut index)?, arg)?);
        } else if let Some(raw) = arg.strip_prefix("--max-files=") {
            args.max_files = Some(positive_integer(raw, "--max-files")?);
        } else if arg == "--output-dir" {
            args.output_dir = Some(PathBuf::from(value(&mut index)?));
        } else if let Some(raw) = arg.strip_prefix("--output-dir=") {
            args.output_dir = Some(PathBuf::from(raw));
        } else if arg == "--project" {
            args.projects.extend(split_csv(&value(&mut index)?));
        } else if let Some(raw) = arg.strip_prefix("--project=") {
            args.projects.extend(split_csv(raw));
        } else if arg == "--target" {
            args.targets.extend(
                split_csv(&value(&mut index)?)
                    .into_iter()
                    .map(parse_target)
                    .collect::<Result<Vec<_>, _>>()?,
            );
        } else if let Some(raw) = arg.strip_prefix("--target=") {
            args.targets.extend(
                split_csv(raw)
                    .into_iter()
                    .map(parse_target)
                    .collect::<Result<Vec<_>, _>>()?,
            );
        } else if arg == "--template-syntax" {
            args.template_syntax = parse_template_syntax(&value(&mut index)?)?;
        } else if let Some(raw) = arg.strip_prefix("--template-syntax=") {
            args.template_syntax = parse_template_syntax(raw)?;
        } else if arg == "--timeout-ms" {
            args.timeout_ms = positive_integer(&value(&mut index)?, arg)?;
        } else if let Some(raw) = arg.strip_prefix("--timeout-ms=") {
            args.timeout_ms = positive_integer(raw, "--timeout-ms")?;
        } else if arg == "--vize-bin" {
            args.vize_bin = Some(PathBuf::from(value(&mut index)?));
        } else if let Some(raw) = arg.strip_prefix("--vize-bin=") {
            args.vize_bin = Some(PathBuf::from(raw));
        } else {
            return Err(format!("Unknown argument: {arg}"));
        }
        index += 1;
    }
    args.projects = stable_unique(args.projects);
    args.targets = stable_unique(args.targets);
    Ok(args)
}

fn stable_unique(values: Vec<String>) -> Vec<String> {
    let mut unique = Vec::new();
    for value in values {
        if !unique.contains(&value) {
            unique.push(value);
        }
    }
    unique
}

fn print_help() {
    println!(
        "Usage: rust-script tools/commands/fixtures/compiler-diff-report.rs [options]\n\
\n\
Compare every Vue ecosystem fixture project against the official Vue compiler.\n\
\n\
Options:\n\
  --project <id[,id]>       Limit to one or more registry project ids.\n\
  --target <dom|ssr>        Limit target; repeat or comma-separate. Defaults to dom,ssr.\n\
  --max-files <n>           Forward a per-project file limit to vize inspector.\n\
  --template-syntax <mode>  Forward template syntax mode. Defaults to quirks.\n\
  --output-dir <dir>        Report directory. Defaults under .vize/artifacts/compiler-diff-report.\n\
  --vize-bin <path>         vize binary. Defaults to VIZE_BIN, target/ci, target/debug, or cargo.\n\
  --timeout-ms <n>          Per project/target timeout. Defaults to 300000.\n\
  --dry-run                 Write the planned report without invoking vize.\n\
  --allow-failures          Keep exit code 0 even if some project/target runs fail."
    );
}

fn run_project_target(
    root: &Path,
    project: &Value,
    target: &str,
    args: &Args,
    launch: &Launch,
    output_dir: &Path,
) -> Result<Value, String> {
    let cwd = root.join(project_string(project, "fixturePath")?);
    let mut command_args = launch.prefix.clone();
    command_args.push("inspector".to_string());
    command_args.extend(project_globs(project)?);
    command_args.extend([
        "--format".to_string(),
        "compare".to_string(),
        "--target".to_string(),
        target.to_string(),
        "--template-syntax".to_string(),
        args.template_syntax.clone(),
    ]);
    if let Some(max_files) = args.max_files {
        command_args.extend(["--max-files".to_string(), max_files.to_string()]);
    }

    let raw_path = output_dir.join(format!("{}-{target}.json", project_string(project, "id")?));
    if args.dry_run {
        return Ok(json!({
            "target": target,
            "status": "planned",
            "durationMs": 0,
            "command": common::command_line(&launch.command, &command_args),
            "cwd": common::relative_path(root, &cwd),
            "outputPath": common::relative_path(root, &raw_path),
            "summary": empty_summary(),
            "largestDiffs": [],
        }));
    }

    let started = std::time::Instant::now();
    let output = Command::new(&launch.command)
        .args(&command_args)
        .current_dir(&cwd)
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run {}: {error}", launch.command))?;
    let duration_ms = started.elapsed().as_millis() as u64;
    if !output.status.success() {
        return Ok(json!({
            "target": target,
            "status": "failed",
            "durationMs": duration_ms,
            "command": common::command_line(&launch.command, &command_args),
            "cwd": common::relative_path(root, &cwd),
            "outputPath": common::relative_path(root, &raw_path),
            "summary": empty_summary(),
            "largestDiffs": [],
            "failure": {
                "exitCode": output.status.code().unwrap_or(1),
                "stderr": truncate(&String::from_utf8_lossy(&output.stderr)),
                "stdout": truncate(&String::from_utf8_lossy(&output.stdout)),
            },
        }));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let raw_report = match serde_json::from_str::<Value>(&stdout) {
        Ok(value) => value,
        Err(error) => {
            return Ok(json!({
                "target": target,
                "status": "failed",
                "durationMs": duration_ms,
                "command": common::command_line(&launch.command, &command_args),
                "cwd": common::relative_path(root, &cwd),
                "outputPath": common::relative_path(root, &raw_path),
                "summary": empty_summary(),
                "largestDiffs": [],
                "failure": {
                    "message": error.to_string(),
                    "project": project_string(project, "id")?,
                },
            }));
        }
    };
    common::write_json_pretty(&raw_path, &raw_report)?;
    Ok(json!({
        "target": target,
        "status": "ok",
        "durationMs": duration_ms,
        "command": common::command_line(&launch.command, &command_args),
        "cwd": common::relative_path(root, &cwd),
        "outputPath": common::relative_path(root, &raw_path),
        "summary": raw_report.get("summary").cloned().unwrap_or_else(empty_summary),
        "largestDiffs": largest_diffs(&raw_report),
    }))
}

fn bump_summary(report: &mut Value, target: &Value) {
    let summary = &mut report["summary"];
    match target.get("status").and_then(Value::as_str).unwrap_or("") {
        "planned" => {
            summary["plannedTargets"] = json!(summary["plannedTargets"].as_u64().unwrap_or(0) + 1)
        }
        "ok" => {
            summary["okTargets"] = json!(summary["okTargets"].as_u64().unwrap_or(0) + 1);
            for key in [
                "changedFiles",
                "additions",
                "removals",
                "officialErrors",
                "vizeErrors",
            ] {
                summary[key] = json!(
                    summary[key].as_u64().unwrap_or(0)
                        + target["summary"][key].as_u64().unwrap_or(0)
                );
            }
        }
        _ => summary["failedTargets"] = json!(summary["failedTargets"].as_u64().unwrap_or(0) + 1),
    }
}

fn render_markdown(report: &Value) -> String {
    let summary = &report["summary"];
    let command = &report["command"];
    let targets = command["targets"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(Value::as_str)
        .map(|target| format!("`{target}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut lines = vec![
        "# Vize Fixture Compiler Diff Report".to_string(),
        String::new(),
        format!("Generated: {}", report["generatedAt"].as_str().unwrap_or("unknown")),
        format!("Registry: `{}`", report["registryPath"].as_str().unwrap_or("unknown")),
        format!("Targets: {targets}"),
        String::new(),
        "## Summary".to_string(),
        String::new(),
        "| Projects | Targets planned | Targets OK | Targets failed | Changed files | Additions | Removals | Official errors | Vize errors |".to_string(),
        "| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |".to_string(),
        format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            summary["projectCount"],
            summary["plannedTargets"],
            summary["okTargets"],
            summary["failedTargets"],
            summary["changedFiles"],
            summary["additions"],
            summary["removals"],
            summary["officialErrors"],
            summary["vizeErrors"],
        ),
        String::new(),
        "## Project Targets".to_string(),
        String::new(),
        "| Project | Target | Status | Files | Changed | Additions | Removals | Official errors | Vize errors | Report |".to_string(),
        "| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |".to_string(),
    ];
    for project in report["projects"].as_array().unwrap_or(&Vec::new()) {
        for target in project["targets"].as_array().unwrap_or(&Vec::new()) {
            lines.push(format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | `{}` |",
                project["id"].as_str().unwrap_or("-"),
                target["target"].as_str().unwrap_or("-"),
                target["status"].as_str().unwrap_or("-"),
                target["summary"]["fileCount"],
                target["summary"]["changedFiles"],
                target["summary"]["additions"],
                target["summary"]["removals"],
                target["summary"]["officialErrors"],
                target["summary"]["vizeErrors"],
                target["outputPath"].as_str().unwrap_or("-")
            ));
        }
    }
    lines.push(String::new());
    lines.push("## Largest Diffs".to_string());
    lines.push(String::new());
    for project in report["projects"].as_array().unwrap_or(&Vec::new()) {
        for target in project["targets"].as_array().unwrap_or(&Vec::new()) {
            let largest = target["largestDiffs"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            if largest.is_empty() {
                continue;
            }
            lines.push(format!(
                "### {} {}",
                project["id"].as_str().unwrap_or("-"),
                target["target"].as_str().unwrap_or("-")
            ));
            lines.push(String::new());
            for file in largest.into_iter().take(10) {
                let additions = file["additions"].as_u64().unwrap_or(0);
                let removals = file["removals"].as_u64().unwrap_or(0);
                lines.push(format!(
                    "- `{}` (+{additions}/-{removals}, total {})",
                    file["path"].as_str().unwrap_or("-"),
                    additions + removals
                ));
            }
            lines.push(String::new());
        }
    }
    if summary["failedTargets"].as_u64().unwrap_or(0) > 0 {
        lines.push("## Failures".to_string());
        lines.push(String::new());
        for project in report["projects"].as_array().unwrap_or(&Vec::new()) {
            for target in project["targets"].as_array().unwrap_or(&Vec::new()) {
                if target["status"].as_str() != Some("failed") {
                    continue;
                }
                let message = target["failure"]["message"]
                    .as_str()
                    .or_else(|| target["failure"]["stderr"].as_str())
                    .unwrap_or("unknown error");
                lines.push(format!(
                    "- {}:{} failed: {message}",
                    project["id"].as_str().unwrap_or("-"),
                    target["target"].as_str().unwrap_or("-")
                ));
            }
        }
        lines.push(String::new());
    }
    format!("{}\n", lines.join("\n"))
}

fn resolve_vize_launch(root: &Path, vize_bin: Option<&Path>) -> Result<Launch, String> {
    let executable = if env::consts::OS == "windows" {
        "vize.exe"
    } else {
        "vize"
    };
    let current = env::current_dir().map_err(|error| error.to_string())?;
    let mut candidates = Vec::new();
    if let Some(vize_bin) = vize_bin {
        candidates.push(if vize_bin.is_absolute() {
            vize_bin.to_path_buf()
        } else {
            current.join(vize_bin)
        });
    }
    if let Some(env_bin) = env::var_os("VIZE_BIN") {
        let env_bin = PathBuf::from(env_bin);
        candidates.push(if env_bin.is_absolute() {
            env_bin
        } else {
            current.join(env_bin)
        });
    }
    candidates.extend([
        root.join("target/ci").join(executable),
        root.join("target/debug").join(executable),
        root.join("target/release").join(executable),
    ]);
    for candidate in candidates {
        if !candidate.exists() {
            continue;
        }
        let resolved = candidate.canonicalize().unwrap_or(candidate);
        if Command::new(&resolved)
            .arg("--version")
            .current_dir(root)
            .stdin(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
        {
            let label = resolved.display().to_string();
            return Ok(Launch {
                command: label.clone(),
                prefix: Vec::new(),
                label,
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
        label: "cargo run -q -p vize --".to_string(),
    })
}

fn select_projects<'a>(
    projects: &'a [Value],
    selected_ids: &[String],
) -> Result<Vec<&'a Value>, String> {
    if selected_ids.is_empty() {
        return Ok(projects.iter().collect());
    }
    selected_ids
        .iter()
        .map(|id| {
            projects
                .iter()
                .find(|project| project.get("id").and_then(Value::as_str) == Some(id.as_str()))
                .ok_or_else(|| format!("Unknown fixture project: {id}"))
        })
        .collect()
}

fn project_globs(project: &Value) -> Result<Vec<String>, String> {
    project
        .get("vueGlobs")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            format!(
                "{} has no vueGlobs",
                project_string(project, "id").unwrap_or_default()
            )
        })?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| "vueGlobs entries must be strings".to_string())
        })
        .collect()
}

fn project_string(project: &Value, field: &str) -> Result<String, String> {
    project
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("project is missing {field}"))
}

fn largest_diffs(raw_report: &Value) -> Value {
    let mut diffs = raw_report
        .get("files")
        .and_then(Value::as_array)
        .unwrap_or(&Vec::new())
        .iter()
        .filter(|file| file.get("changed").and_then(Value::as_bool) == Some(true))
        .map(|file| {
            json!({
                "path": file.get("path").and_then(Value::as_str).unwrap_or(""),
                "additions": file.pointer("/stats/additions").and_then(Value::as_u64).unwrap_or(0),
                "removals": file.pointer("/stats/removals").and_then(Value::as_u64).unwrap_or(0),
                "officialError": file.pointer("/official/error").cloned().unwrap_or(Value::Null),
                "vizeError": file.pointer("/vize/error").cloned().unwrap_or(Value::Null),
            })
        })
        .collect::<Vec<_>>();
    diffs.sort_by(|left, right| {
        let left_total =
            left["additions"].as_u64().unwrap_or(0) + left["removals"].as_u64().unwrap_or(0);
        let right_total =
            right["additions"].as_u64().unwrap_or(0) + right["removals"].as_u64().unwrap_or(0);
        right_total.cmp(&left_total)
    });
    Value::Array(diffs.into_iter().take(20).collect())
}

fn empty_summary() -> Value {
    json!({
        "fileCount": 0,
        "changedFiles": 0,
        "additions": 0,
        "removals": 0,
        "officialErrors": 0,
        "vizeErrors": 0,
    })
}

fn parse_target(target: String) -> Result<String, String> {
    if target == "dom" || target == "ssr" {
        Ok(target)
    } else {
        Err(format!("Unsupported target: {target}"))
    }
}

fn parse_template_syntax(value: &str) -> Result<String, String> {
    if matches!(value, "standard" | "strict" | "quirks") {
        Ok(value.to_string())
    } else {
        Err(format!("Unsupported template syntax mode: {value}"))
    }
}

fn positive_integer(value: &str, name: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("{name} must be a positive integer"))
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

fn truncate(value: &str) -> String {
    if value.len() <= 4000 {
        value.to_string()
    } else {
        format!("{}\n...<truncated>", &value[..4000])
    }
}
