#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! chrono = "0.4"
//! glob = "0.3"
//! libc = "0.2"
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! sha2 = "0.10"
//! tempfile = "3"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../support/common.rs"]
mod common;

use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, ExitCode, Stdio},
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

const SCHEMA: &str = "vize.fixtureToolMatrixReport";
const SUPPORTED_TOOLS: &[&str] = &["compiler", "typechecker", "linter", "formatter"];

#[derive(Clone, Debug)]
struct Args {
    dry_run: bool,
    list_fixture_paths: bool,
    heartbeat_ms: u64,
    output_dir: Option<PathBuf>,
    projects: Vec<String>,
    shard_count: usize,
    shard_index: usize,
    timeout_ms: u64,
    tools: Vec<String>,
    vize_bin: Option<PathBuf>,
}

#[derive(Clone, Debug)]
struct Launch {
    command: String,
    prefix: Vec<String>,
    label: String,
}

struct FixtureTypecheckTsconfig {
    relative_path: String,
    cleanup_path: Option<PathBuf>,
}

impl Drop for FixtureTypecheckTsconfig {
    fn drop(&mut self) {
        if let Some(path) = &self.cleanup_path {
            let _ = fs::remove_file(path);
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunEvidence {
    commit_sha: String,
    runtime: RuntimeEvidence,
    machine: MachineEvidence,
}

#[derive(Serialize)]
struct RuntimeEvidence {
    name: &'static str,
    version: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MachineEvidence {
    platform: String,
    arch: String,
    cpu_model: String,
    logical_cpu_count: usize,
    total_memory_bytes: u64,
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
    let root = repo_root()?;
    let args = parse_args(env::args().skip(1).collect())?;
    let registry_path = root.join("tests/_fixtures/vue-ecosystem-fixtures.json");
    let registry = common::read_json(&registry_path)?;
    let registry_projects = registry
        .get("projects")
        .and_then(Value::as_array)
        .ok_or_else(|| "Fixture registry must list projects".to_string())?;
    let selected = select_projects(registry_projects, &args.projects)?;
    let projects = select_shard(selected, args.shard_index, args.shard_count);
    let tools = if args.tools.is_empty() {
        SUPPORTED_TOOLS
            .iter()
            .map(|tool| (*tool).to_string())
            .collect()
    } else {
        select_tools(&args.tools)?
    };
    assert_registry_coverage(&registry, &projects, &tools)?;
    if args.list_fixture_paths {
        for project in &projects {
            println!("{}", project_string(project, "fixturePath")?);
        }
        return Ok(0);
    }

    let output_dir = args
        .output_dir
        .clone()
        .unwrap_or_else(|| root.join(format!(".vize/fixture-tool-matrix/{}", timestamp_slug())));
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("cannot create {}: {error}", output_dir.display()))?;
    let launch = resolve_vize_launch(&root, args.vize_bin.as_deref(), args.dry_run)?;
    let mut projects_report = Vec::new();
    let mut planned_runs = 0usize;
    let mut ok_runs = 0usize;
    let mut findings_runs = 0usize;
    let mut failed_runs = 0usize;
    let mut missing_fixture_runs = 0usize;

    for project in &projects {
        let mut runs = Vec::new();
        for tool in &tools {
            let run = run_tool(&root, project, tool, &args, &launch, &output_dir)?;
            match run.get("status").and_then(Value::as_str).unwrap_or("") {
                "planned" => planned_runs += 1,
                "ok" => ok_runs += 1,
                "findings" => findings_runs += 1,
                "missing-fixture" => missing_fixture_runs += 1,
                _ => failed_runs += 1,
            }
            runs.push(run);
        }
        projects_report.push(json!({
            "id": project_string(project, "id")?,
            "fixturePath": project_string(project, "fixturePath")?,
            "revision": project.get("revision").cloned().unwrap_or(Value::Null),
            "runs": runs,
        }));
    }
    let report = json!({
        "schema": SCHEMA,
        "version": 3,
        "generatedAt": chrono::Utc::now().to_rfc3339(),
        "evidence": collect_run_evidence(&root)?,
        "registryPath": "tests/_fixtures/vue-ecosystem-fixtures.json",
        "command": {
            "vize": launch.label,
            "dryRun": args.dry_run,
            "timeoutMs": args.timeout_ms,
            "tools": tools,
            "shardIndex": args.shard_index,
            "shardCount": args.shard_count,
        },
        "summary": {
            "projectCount": projects.len(),
            "toolCount": tools.len(),
            "runCount": projects.len() * tools.len(),
            "plannedRuns": planned_runs,
            "okRuns": ok_runs,
            "findingsRuns": findings_runs,
            "failedRuns": failed_runs,
            "missingFixtureRuns": missing_fixture_runs,
        },
        "projects": projects_report,
    });
    let json_path = output_dir.join("summary.json");
    let markdown_path = output_dir.join("summary.md");
    common::write_json_pretty(&json_path, &report)?;
    common::write_text(&markdown_path, &render_markdown(&root, &report))?;
    println!("Wrote {}", common::relative_path(&root, &json_path));
    println!("Wrote {}", common::relative_path(&root, &markdown_path));
    Ok(if failed_runs > 0 || missing_fixture_runs > 0 {
        1
    } else {
        0
    })
}

fn parse_args(argv: Vec<String>) -> Result<Args, String> {
    let mut args = Args {
        dry_run: false,
        list_fixture_paths: false,
        heartbeat_ms: 30_000,
        output_dir: None,
        projects: Vec::new(),
        shard_count: 1,
        shard_index: 0,
        timeout_ms: 300_000,
        tools: Vec::new(),
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
        if arg == "--dry-run" {
            args.dry_run = true;
        } else if arg == "--list-fixture-paths" {
            args.list_fixture_paths = true;
        } else if arg == "--help" || arg == "-h" {
            print_help();
            std::process::exit(0);
        } else if arg == "--heartbeat-ms" {
            args.heartbeat_ms = positive_integer(&value(&mut index)?, arg)?;
        } else if let Some(raw) = arg.strip_prefix("--heartbeat-ms=") {
            args.heartbeat_ms = positive_integer(raw, "--heartbeat-ms")?;
        } else if arg == "--output-dir" {
            args.output_dir = Some(PathBuf::from(value(&mut index)?));
        } else if let Some(raw) = arg.strip_prefix("--output-dir=") {
            args.output_dir = Some(PathBuf::from(raw));
        } else if arg == "--project" {
            args.projects.extend(split_csv(&value(&mut index)?));
        } else if let Some(raw) = arg.strip_prefix("--project=") {
            args.projects.extend(split_csv(raw));
        } else if arg == "--shard-count" {
            args.shard_count = positive_integer(&value(&mut index)?, arg)? as usize;
        } else if let Some(raw) = arg.strip_prefix("--shard-count=") {
            args.shard_count = positive_integer(raw, "--shard-count")? as usize;
        } else if arg == "--shard-index" {
            args.shard_index = non_negative_integer(&value(&mut index)?, arg)? as usize;
        } else if let Some(raw) = arg.strip_prefix("--shard-index=") {
            args.shard_index = non_negative_integer(raw, "--shard-index")? as usize;
        } else if arg == "--timeout-ms" {
            args.timeout_ms = positive_integer(&value(&mut index)?, arg)?;
        } else if let Some(raw) = arg.strip_prefix("--timeout-ms=") {
            args.timeout_ms = positive_integer(raw, "--timeout-ms")?;
        } else if arg == "--tool" {
            args.tools.extend(split_csv(&value(&mut index)?));
        } else if let Some(raw) = arg.strip_prefix("--tool=") {
            args.tools.extend(split_csv(raw));
        } else if arg == "--vize-bin" {
            args.vize_bin = Some(PathBuf::from(value(&mut index)?));
        } else if let Some(raw) = arg.strip_prefix("--vize-bin=") {
            args.vize_bin = Some(PathBuf::from(raw));
        } else {
            return Err(format!("Unknown argument: {arg}"));
        }
        index += 1;
    }
    args.projects.sort();
    args.projects.dedup();
    args.tools.sort();
    args.tools.dedup();
    if args.shard_index >= args.shard_count {
        return Err("--shard-index must be less than --shard-count".to_string());
    }
    Ok(args)
}

fn print_help() {
    println!("Usage: rust-script tools/commands/fixtures/tool-matrix-report.rs [options]\n");
    println!(
        "Exercise every registered real project with compiler, typechecker, linter, and formatter.\n"
    );
    println!("  --project <id[,id]>  Limit registry projects");
    println!("  --tool <name[,name]> Limit tool surfaces");
    println!("  --shard-index <n>    Zero-based project shard index");
    println!("  --shard-count <n>    Total balanced project shards");
    println!("  --list-fixture-paths Print selected fixture paths and exit");
    println!("  --output-dir <dir>   Report directory");
    println!("  --vize-bin <path>    Vize executable");
    println!("  --timeout-ms <n>     Per-run timeout");
    println!("  --heartbeat-ms <n>   Progress heartbeat interval for child runs");
    println!("  --dry-run            Plan without invoking Vize");
}

fn select_projects<'a>(
    projects: &'a [Value],
    selected: &[String],
) -> Result<Vec<&'a Value>, String> {
    if selected.is_empty() {
        return Ok(projects.iter().collect());
    }
    selected
        .iter()
        .map(|id| {
            projects
                .iter()
                .find(|project| project.get("id").and_then(Value::as_str) == Some(id.as_str()))
                .ok_or_else(|| format!("Unknown fixture project: {id}"))
        })
        .collect()
}

fn select_shard(projects: Vec<&Value>, shard_index: usize, shard_count: usize) -> Vec<&Value> {
    projects
        .into_iter()
        .enumerate()
        .filter_map(|(index, project)| (index % shard_count == shard_index).then_some(project))
        .collect()
}

fn select_tools(selected: &[String]) -> Result<Vec<String>, String> {
    selected
        .iter()
        .map(|tool| {
            if SUPPORTED_TOOLS.contains(&tool.as_str()) {
                Ok(tool.clone())
            } else {
                Err(format!("Unknown fixture tool: {tool}"))
            }
        })
        .collect()
}

fn assert_registry_coverage(
    registry: &Value,
    projects: &[&Value],
    tools: &[String],
) -> Result<(), String> {
    let required = registry
        .get("requiredToolCoverage")
        .and_then(Value::as_array)
        .ok_or_else(|| "Registry does not declare requiredToolCoverage".to_string())?;
    for tool in tools {
        if !required.iter().any(|value| value.as_str() == Some(tool)) {
            return Err(format!("Registry does not require tool coverage: {tool}"));
        }
    }
    for project in projects {
        let coverage = project
            .get("coverage")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                format!(
                    "{} does not declare coverage",
                    project_string(project, "id").unwrap_or_default()
                )
            })?;
        for tool in tools {
            if !coverage.iter().any(|value| value.as_str() == Some(tool)) {
                return Err(format!(
                    "{} does not declare {tool} coverage",
                    project_string(project, "id")?
                ));
            }
        }
    }
    Ok(())
}

struct ToolProcessOutput {
    exit_code: Option<i32>,
    signal: Option<String>,
    spawn_error: Option<String>,
    stdout: String,
    stderr: String,
}

fn spawn_with_heartbeat(
    command: &str,
    command_args: &[String],
    cwd: &Path,
    project_id: String,
    tool: &str,
    timeout_ms: u64,
    heartbeat_ms: u64,
) -> Result<ToolProcessOutput, String> {
    eprintln!("[tool-matrix] start projectId={project_id} tool={tool} timeoutMs={timeout_ms}");
    let stdout_file = tempfile::NamedTempFile::new().map_err(|error| error.to_string())?;
    let stderr_file = tempfile::NamedTempFile::new().map_err(|error| error.to_string())?;
    let mut child_command = Command::new(command);
    child_command
        .args(command_args)
        .current_dir(cwd)
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::from(
            stdout_file.reopen().map_err(|error| error.to_string())?,
        ))
        .stderr(Stdio::from(
            stderr_file.reopen().map_err(|error| error.to_string())?,
        ));
    #[cfg(unix)]
    child_command.process_group(0);

    let started = Instant::now();
    let mut child = match child_command.spawn() {
        Ok(child) => child,
        Err(error) => {
            let elapsed = started.elapsed().as_millis();
            eprintln!(
                "[tool-matrix] finish projectId={project_id} tool={tool} elapsedMs={elapsed} status=null"
            );
            return Ok(ToolProcessOutput {
                exit_code: None,
                signal: None,
                spawn_error: Some(error.to_string()),
                stdout: String::new(),
                stderr: String::new(),
            });
        }
    };
    let timeout = Duration::from_millis(timeout_ms);
    let heartbeat = Duration::from_millis(heartbeat_ms.max(1));
    let mut next_heartbeat = heartbeat;
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("failed to poll {command}: {error}"))?
        {
            let elapsed = started.elapsed().as_millis();
            let code = status.code();
            let status_field = code
                .map(|value| value.to_string())
                .unwrap_or_else(|| "null".to_string());
            eprintln!(
                "[tool-matrix] finish projectId={project_id} tool={tool} elapsedMs={elapsed} status={status_field}"
            );
            return Ok(ToolProcessOutput {
                exit_code: code,
                signal: signal_name(&status),
                spawn_error: None,
                stdout: read_named_temp(&stdout_file)?,
                stderr: read_named_temp(&stderr_file)?,
            });
        }

        let elapsed = started.elapsed();
        if elapsed >= timeout {
            terminate_child_tree(&mut child);
            let elapsed_ms = started.elapsed().as_millis();
            eprintln!(
                "[tool-matrix] finish projectId={project_id} tool={tool} elapsedMs={elapsed_ms} status=null"
            );
            return Ok(ToolProcessOutput {
                exit_code: None,
                signal: Some("SIGKILL".to_string()),
                spawn_error: Some(format!("spawn timed out after {timeout_ms}ms")),
                stdout: read_named_temp(&stdout_file)?,
                stderr: read_named_temp(&stderr_file)?,
            });
        }

        if elapsed >= next_heartbeat {
            eprintln!(
                "[tool-matrix] still-running projectId={project_id} tool={tool} elapsedMs={}",
                elapsed.as_millis()
            );
            next_heartbeat += heartbeat;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn read_named_temp(file: &tempfile::NamedTempFile) -> Result<String, String> {
    let mut text = String::new();
    fs::File::open(file.path())
        .map_err(|error| format!("cannot read {}: {error}", file.path().display()))?
        .read_to_string(&mut text)
        .map_err(|error| format!("cannot read {}: {error}", file.path().display()))?;
    Ok(text)
}

fn terminate_child_tree(child: &mut Child) {
    #[cfg(unix)]
    unsafe {
        let group = -(child.id() as i32);
        let _ = libc::kill(group, libc::SIGTERM);
        std::thread::sleep(Duration::from_millis(50));
        if child.try_wait().ok().flatten().is_none() {
            let _ = libc::kill(group, libc::SIGKILL);
        }
    }
    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }
    let _ = child.wait();
}

#[cfg(unix)]
fn signal_name(status: &std::process::ExitStatus) -> Option<String> {
    use std::os::unix::process::ExitStatusExt;
    status.signal().map(|signal| format!("SIG{signal}"))
}

#[cfg(not(unix))]
fn signal_name(_status: &std::process::ExitStatus) -> Option<String> {
    None
}

fn run_tool(
    root: &Path,
    project: &Value,
    tool: &str,
    args: &Args,
    launch: &Launch,
    output_dir: &Path,
) -> Result<Value, String> {
    let fixture_path = project_string(project, "fixturePath")?;
    let cwd = root.join(&fixture_path);
    let fixture_exists = cwd.is_dir();
    let compiler_dir = if tool == "compiler" && !args.dry_run && fixture_exists {
        Some(
            tempfile::Builder::new()
                .prefix("vize-fixture-compiler-")
                .tempdir()
                .map_err(|error| error.to_string())?,
        )
    } else {
        None
    };
    let compiler_output = compiler_dir
        .as_ref()
        .map(|dir| dir.path().to_string_lossy().into_owned())
        .unwrap_or_else(|| "<compiler-output>".to_string());
    let typechecker_fixture_tsconfig =
        prepare_typechecker_fixture_tsconfig(project, tool, args.dry_run, &cwd, fixture_exists)?;
    let mut command_args = launch.prefix.clone();
    command_args.extend(tool_args(
        project,
        tool,
        &compiler_output,
        typechecker_fixture_tsconfig
            .as_ref()
            .map(|tsconfig| tsconfig.relative_path.as_str()),
    )?);
    let base = json!({
        "tool": tool,
        "command": display_command(&launch.command, &command_args),
        "cwd": fixture_path,
        "durationMs": 0,
        "fileCount": Value::Null,
        "exitCode": Value::Null,
        "outputPath": Value::Null,
        "coverage": Value::Null,
    });
    if args.dry_run {
        return Ok(merge_base(base, json!({ "status": "planned" })));
    }
    if !fixture_exists {
        return Ok(merge_base(base, json!({ "status": "missing-fixture" })));
    }
    let expected_tool_files = if matches!(tool, "typechecker" | "linter" | "formatter") {
        Some(collect_vue_input_paths(
            &cwd,
            &if tool == "typechecker" {
                typecheck_corpus_globs(project)?
            } else {
                project_string_array(project, "vueGlobs")?
            },
        )?)
    } else {
        None
    };
    let authored_files = if tool == "typechecker" {
        Some(collect_typechecker_authored_paths(&cwd)?)
    } else {
        None
    };
    let formatter_state_before = if tool == "formatter" {
        Some(snapshot_formatter_inputs(
            &cwd,
            &project_string_array(project, "vueGlobs")?,
        )?)
    } else {
        None
    };
    let started = Instant::now();
    let output = spawn_with_heartbeat(
        &launch.command,
        &command_args,
        &cwd,
        project_string(project, "id")?,
        tool,
        args.timeout_ms,
        args.heartbeat_ms,
    )?;
    let duration_ms = started.elapsed().as_millis() as u64;
    let raw_path = output_dir.join(format!("{}-{tool}.json", project_string(project, "id")?));
    let mut payload = json!({
        "schema": "vize.fixtureToolRun",
        "version": 1,
        "project": project_string(project, "id")?,
        "tool": tool,
        "exitCode": output.exit_code,
        "stdout": output.stdout,
        "stderr": output.stderr,
    });
    if let Some(signal) = &output.signal {
        payload["signal"] = Value::String(signal.clone());
    }
    if let Some(error) = &output.spawn_error {
        payload["spawnError"] = Value::String(error.clone());
    } else if tool == "compiler" && matches!(output.exit_code, Some(0) | Some(1)) {
        match inspect_compiler_artifacts(
            &cwd,
            &project_string_array(project, "vueGlobs")?,
            project.get("expectedVueFileCount").and_then(Value::as_u64),
            compiler_dir
                .as_ref()
                .map(|dir| dir.path())
                .ok_or_else(|| "compiler output directory was not prepared".to_string())?,
        ) {
            Ok(artifacts) => payload["compilerArtifacts"] = artifacts,
            Err(error) => payload["validationError"] = Value::String(error),
        }
    } else if matches!(tool, "typechecker" | "linter")
        && matches!(output.exit_code, Some(0) | Some(1))
    {
        match serde_json::from_str::<Value>(
            payload.get("stdout").and_then(Value::as_str).unwrap_or(""),
        ) {
            Ok(parsed) => payload["parsed"] = parsed,
            Err(error) => payload["parseError"] = Value::String(error.to_string()),
        }
        if payload.get("parseError").is_none() && tool == "typechecker" {
            match validate_typechecker_output(
                project,
                &payload["parsed"],
                output.exit_code.unwrap_or(1),
                expected_tool_files.as_deref(),
                authored_files.as_deref().unwrap_or(&[]),
            ) {
                Ok(coverage) => payload["typecheckerCoverage"] = coverage,
                Err(error) => payload["validationError"] = Value::String(error),
            }
        }
        if payload.get("parseError").is_none() && tool == "linter" {
            if let Err(error) = validate_linter_output(
                project,
                &payload["parsed"],
                output.exit_code.unwrap_or(1),
                expected_tool_files.as_deref(),
            ) {
                payload["validationError"] = Value::String(error);
            }
        }
    } else if tool == "formatter" && matches!(output.exit_code, Some(0) | Some(1)) {
        match validate_formatter_output(
            project,
            payload.get("stdout").and_then(Value::as_str).unwrap_or(""),
            payload.get("stderr").and_then(Value::as_str).unwrap_or(""),
            output.exit_code.unwrap_or(1),
            formatter_state_before.as_deref().unwrap_or(""),
            &snapshot_formatter_inputs(&cwd, &project_string_array(project, "vueGlobs")?)?,
            expected_tool_files.as_deref(),
        ) {
            Ok(formatter_check) => payload["formatterCheck"] = formatter_check,
            Err(error) => payload["validationError"] = Value::String(error),
        }
    }
    common::write_json_pretty(&raw_path, &payload)?;
    let status = if output.spawn_error.is_some() {
        "failed"
    } else if !matches!(output.exit_code, Some(0) | Some(1)) {
        "failed"
    } else if payload.get("validationError").is_some() {
        "failed"
    } else if payload.get("parseError").is_some() {
        "failed"
    } else if output.exit_code == Some(0) {
        "ok"
    } else {
        "findings"
    };
    let mut run = base;
    run["durationMs"] = json!(duration_ms);
    run["exitCode"] = output.exit_code.map(Value::from).unwrap_or(Value::Null);
    run["outputPath"] = json!(common::relative_path(root, &raw_path));
    run["status"] = json!(status);
    if status == "failed" {
        run["failure"] = run_failure(&payload);
    } else {
        run["fileCount"] = json!(validated_file_count(tool, &payload));
        if tool == "typechecker" {
            run["coverage"] = summarize_typechecker_coverage(&payload["typecheckerCoverage"])?;
        }
    }
    Ok(run)
}

fn tool_args(
    project: &Value,
    tool: &str,
    compiler_output_dir: &str,
    typecheck_tsconfig_override: Option<&str>,
) -> Result<Vec<String>, String> {
    let vue_globs = project
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
        .collect::<Result<Vec<_>, _>>()?;
    match tool {
        "compiler" => Ok([
            vec!["build".to_string()],
            vue_globs,
            vec![
                "--format".to_string(),
                "json".to_string(),
                "--output".to_string(),
                compiler_output_dir.to_string(),
                "--template-syntax".to_string(),
                "quirks".to_string(),
                "--continue-on-error".to_string(),
                "--no-config".to_string(),
            ],
        ]
        .concat()),
        "linter" => Ok([
            vec!["lint".to_string()],
            vue_globs,
            vec![
                "--format".to_string(),
                "json".to_string(),
                "--preset".to_string(),
                "ecosystem".to_string(),
                "--no-config".to_string(),
            ],
        ]
        .concat()),
        "typechecker" => {
            let mut values = vec!["check".to_string()];
            values.extend(typecheck_corpus_globs(project)?);
            values.extend(
                ["--format", "json", "--no-config"]
                    .iter()
                    .map(|value| (*value).to_string()),
            );
            if let Some(tsconfig) = typecheck_tsconfig_override
                .map(str::to_string)
                .or_else(|| typecheck_tsconfig_path(project))
            {
                values.extend(["--tsconfig".to_string(), tsconfig]);
            }
            Ok(values)
        }
        "formatter" => Ok([
            vec!["fmt".to_string()],
            vue_globs,
            vec!["--check".to_string(), "--no-config".to_string()],
        ]
        .concat()),
        _ => Err(format!("Unknown fixture tool: {tool}")),
    }
}

fn typecheck_corpus_globs(project: &Value) -> Result<Vec<String>, String> {
    let values = project
        .get("typecheckPerformance")
        .and_then(|value| value.get("corpusGlobs"))
        .or_else(|| project.get("vueGlobs"))
        .and_then(Value::as_array)
        .ok_or_else(|| "project has no typecheck corpus globs".to_string())?;
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| "corpus globs must be strings".to_string())
        })
        .collect()
}

fn typecheck_tsconfig_path(project: &Value) -> Option<String> {
    typecheck_source_tsconfig_path(project)
}

fn typecheck_source_tsconfig_path(project: &Value) -> Option<String> {
    project
        .get("typecheckPerformance")
        .and_then(|value| value.get("baseline"))
        .and_then(|value| value.get("tsconfig"))
        .and_then(Value::as_str)
        .or_else(|| project.get("tsconfig").and_then(Value::as_str))
        .map(str::to_string)
}

fn prepare_typechecker_fixture_tsconfig(
    project: &Value,
    tool: &str,
    dry_run: bool,
    cwd: &Path,
    fixture_exists: bool,
) -> Result<Option<FixtureTypecheckTsconfig>, String> {
    if tool != "typechecker" || typecheck_source_tsconfig_path(project).is_some() {
        return Ok(None);
    }
    let relative_path = format!(
        ".vize-fixture-typecheck-{}.tsconfig.json",
        fixture_project_id(project)?
    );
    if !fixture_exists || dry_run {
        return Ok(Some(FixtureTypecheckTsconfig {
            relative_path,
            cleanup_path: None,
        }));
    }
    let absolute_path = cwd.join(&relative_path);
    fs::write(&absolute_path, "{\n  \"compilerOptions\": {}\n}\n").map_err(|error| {
        format!(
            "cannot write fixture-local typechecker tsconfig {}: {error}",
            absolute_path.display()
        )
    })?;
    Ok(Some(FixtureTypecheckTsconfig {
        relative_path,
        cleanup_path: Some(absolute_path),
    }))
}

fn fixture_project_id(project: &Value) -> Result<String, String> {
    Ok(project_string(project, "id")?
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-') {
                ch
            } else {
                '_'
            }
        })
        .collect())
}

fn resolve_vize_launch(
    root: &Path,
    vize_bin: Option<&Path>,
    dry_run: bool,
) -> Result<Launch, String> {
    let executable = if env::consts::OS == "windows" {
        "vize.exe"
    } else {
        "vize"
    };
    let mut candidates = Vec::new();
    if let Some(vize_bin) = vize_bin {
        candidates.push(vize_bin.to_path_buf());
    }
    if let Some(vize_bin) = env::var_os("VIZE_BIN") {
        candidates.push(PathBuf::from(vize_bin));
    }
    candidates.extend([
        root.join("target/ci").join(executable),
        root.join("target/debug").join(executable),
        root.join("target/release").join(executable),
    ]);
    for candidate in candidates {
        let candidate = candidate.canonicalize().unwrap_or(candidate);
        if !candidate.exists() {
            continue;
        }
        if dry_run
            || Command::new(&candidate)
                .arg("--version")
                .stdin(Stdio::null())
                .status()
                .is_ok_and(|status| status.success())
        {
            let label = candidate.display().to_string();
            return Ok(Launch {
                command: label.clone(),
                prefix: Vec::new(),
                label,
            });
        }
    }
    if vize_bin.is_some() && !dry_run {
        return Err(format!(
            "Vize executable is not runnable: {}",
            vize_bin.unwrap().display()
        ));
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

fn collect_run_evidence(root: &Path) -> Result<RunEvidence, String> {
    let sha = match env::var("GITHUB_SHA") {
        Ok(value) => {
            if !is_full_lowercase_sha(&value) {
                return Err("GITHUB_SHA must be a full lowercase commit SHA".to_string());
            }
            value
        }
        Err(_) => Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(root)
            .stdin(Stdio::null())
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|value| value.trim().to_string())
            .unwrap_or_default(),
    };
    if !is_full_lowercase_sha(&sha) {
        return Err("git rev-parse HEAD must be a full lowercase commit SHA".to_string());
    }
    Ok(RunEvidence {
        commit_sha: sha,
        runtime: RuntimeEvidence {
            name: "rust-script",
            version: rustc_version(),
        },
        machine: MachineEvidence {
            platform: node_platform(),
            arch: node_arch(),
            cpu_model: cpu_model(),
            logical_cpu_count: std::thread::available_parallelism()
                .map(|count| count.get())
                .unwrap_or(1),
            total_memory_bytes: total_memory_bytes(),
        },
    })
}

fn is_full_lowercase_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn render_markdown(root: &Path, report: &Value) -> String {
    let summary = &report["summary"];
    let command = &report["command"];
    let evidence = &report["evidence"];
    let mut lines = vec![
        "# Vize Fixture Tool Matrix Report".to_string(),
        String::new(),
        format!("Projects: {}", summary["projectCount"]),
        format!(
            "Tools: {}",
            command["tools"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        format!("Runs: {}", summary["runCount"]),
        format!("Commit: {}", evidence["commitSha"].as_str().unwrap_or("unknown")),
        format!(
            "Runtime: {} {}",
            evidence["runtime"]["name"].as_str().unwrap_or("unknown"),
            evidence["runtime"]["version"].as_str().unwrap_or("unknown")
        ),
        format!(
            "Machine: {}/{}, {} logical CPUs, {} bytes memory",
            evidence["machine"]["platform"].as_str().unwrap_or("unknown"),
            evidence["machine"]["arch"].as_str().unwrap_or("unknown"),
            evidence["machine"]["logicalCpuCount"],
            evidence["machine"]["totalMemoryBytes"]
        ),
        String::new(),
        "| Project | Tool | Status | Exit | Files | Requested | Transitive Authored | Transitive Dependencies | Duration (ms) | Output |".to_string(),
        "| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |".to_string(),
    ];
    if let Some(projects) = report["projects"].as_array() {
        for project in projects {
            for run in project["runs"].as_array().unwrap_or(&Vec::new()) {
                let output = run["outputPath"]
                    .as_str()
                    .map(|path| format!("`{}`", common::relative_path(root, &root.join(path))))
                    .unwrap_or_else(|| "-".to_string());
                lines.push(format!(
                    "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                    project["id"].as_str().unwrap_or("-"),
                    run["tool"].as_str().unwrap_or("-"),
                    run["status"].as_str().unwrap_or("-"),
                    value_or_dash(&run["exitCode"]),
                    value_or_dash(&run["fileCount"]),
                    run["coverage"]["requestedFileCount"]
                        .as_i64()
                        .map_or("-".to_string(), |value| value.to_string()),
                    run["coverage"]["transitiveAuthoredFileCount"]
                        .as_i64()
                        .map_or("-".to_string(), |value| value.to_string()),
                    run["coverage"]["transitiveDependencyFileCount"]
                        .as_i64()
                        .map_or("-".to_string(), |value| value.to_string()),
                    value_or_dash(&run["durationMs"]),
                    output
                ));
            }
        }
    }
    format!("{}\n", lines.join("\n"))
}

fn inspect_compiler_artifacts(
    cwd: &Path,
    patterns: &[String],
    expected_file_count: Option<u64>,
    output_dir: &Path,
) -> Result<Value, String> {
    let input_paths = collect_vue_input_paths(cwd, patterns)?;
    if let Some(expected) = expected_file_count {
        if input_paths.len() as u64 != expected {
            return Err(format!(
                "compiler input count mismatch: expected {expected}, matched {}",
                input_paths.len()
            ));
        }
    }
    if input_paths.is_empty() && expected_file_count != Some(0) {
        return Err("compiler matched no Vue files".to_string());
    }

    let output_paths = collect_files(output_dir)?;
    let non_json_paths = output_paths
        .iter()
        .filter(|path| !path.ends_with(".json"))
        .cloned()
        .collect::<Vec<_>>();
    if !non_json_paths.is_empty() {
        return Err(format!(
            "compiler emitted non-JSON artifacts: {}",
            non_json_paths.join(", ")
        ));
    }
    if output_paths.len() != input_paths.len() {
        return Err(format!(
            "compiler artifact count mismatch: {} inputs, {} outputs",
            input_paths.len(),
            output_paths.len()
        ));
    }
    let input_by_output_path = expected_compiler_outputs(cwd, patterns, &input_paths)?;
    let missing_paths = input_by_output_path
        .keys()
        .filter(|path| !output_paths.contains(path))
        .cloned()
        .collect::<Vec<_>>();
    let unexpected_paths = output_paths
        .iter()
        .filter(|path| !input_by_output_path.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_paths.is_empty() || !unexpected_paths.is_empty() {
        return Err(format!(
            "compiler artifact path mismatch: missing [{}], unexpected [{}]",
            missing_paths.join(", "),
            unexpected_paths.join(", ")
        ));
    }

    let mut digest = Sha256::new();
    let mut error_count = 0usize;
    let mut warning_count = 0usize;
    let mut findings = Vec::new();
    for output_path in &output_paths {
        let source = common::read_text(output_dir.join(output_path))?;
        digest.update(output_path.as_bytes());
        digest.update(b"\0");
        digest.update(source.as_bytes());
        digest.update(b"\0");
        let artifact = serde_json::from_str::<Value>(&source).map_err(|error| {
            format!(
                "invalid compiler JSON artifact {output_path}: {}",
                error_message(error)
            )
        })?;
        let input_path = input_by_output_path
            .get(output_path)
            .ok_or_else(|| format!("compiler artifact has no input path: {output_path}"))?;
        validate_compiler_artifact(output_path, &artifact, input_path)?;
        let errors = artifact["errors"].as_array().unwrap();
        let warnings = artifact["warnings"].as_array().unwrap();
        error_count += errors.len();
        warning_count += warnings.len();
        if !errors.is_empty() || !warnings.is_empty() {
            findings.push(json!({
                "file": input_path,
                "errors": errors,
                "warnings": warnings,
            }));
        }
    }

    Ok(json!({
        "inputFileCount": input_paths.len(),
        "outputFileCount": output_paths.len(),
        "errorCount": error_count,
        "warningCount": warning_count,
        "findings": findings,
        "sha256": hex_digest(digest),
    }))
}

fn collect_vue_input_paths(cwd: &Path, patterns: &[String]) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    for pattern in patterns {
        let absolute_pattern = cwd.join(pattern).to_string_lossy().to_string();
        let entries = glob::glob(&absolute_pattern)
            .map_err(|error| format!("invalid glob pattern {pattern}: {error}"))?;
        for entry in entries {
            let path = entry.map_err(|error| format!("failed to read glob entry: {error}"))?;
            if ignored_source_path(&path) {
                continue;
            }
            if path.is_file() {
                let relative = path
                    .strip_prefix(cwd)
                    .map_err(|error| format!("compiler input is outside fixture root: {error}"))?;
                files.push(slash_path(relative));
            }
        }
    }
    files.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    files.dedup();
    Ok(files)
}

fn collect_typechecker_authored_paths(cwd: &Path) -> Result<Vec<String>, String> {
    let patterns = ["vue", "ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs"]
        .into_iter()
        .flat_map(|extension| {
            [
                format!("**/*.{extension}"),
                format!("**/.*/**/*.{extension}"),
            ]
        })
        .collect::<Vec<_>>();
    collect_vue_input_paths(cwd, &patterns)
}

fn ignored_source_path(path: &Path) -> bool {
    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy();
        name == ".yarn" || name == "node_modules"
    })
}

fn collect_files(root: &Path) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    collect_files_inner(root, root, &mut files)?;
    files.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    Ok(files)
}

fn collect_files_inner(
    root: &Path,
    directory: &Path,
    files: &mut Vec<String>,
) -> Result<(), String> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("cannot stat {}: {error}", path.display()))?;
        if file_type.is_dir() {
            collect_files_inner(root, &path, files)?;
        } else if file_type.is_file() {
            files.push(slash_path(path.strip_prefix(root).map_err(|error| {
                format!("compiler output is outside output root: {error}")
            })?));
        } else {
            return Err(format!(
                "compiler emitted unsupported artifact: {}",
                entry.file_name().to_string_lossy()
            ));
        }
    }
    Ok(())
}

fn expected_compiler_outputs(
    cwd: &Path,
    patterns: &[String],
    input_paths: &[String],
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let mut pattern_roots = patterns
        .iter()
        .filter_map(|pattern| compiler_input_root(cwd, pattern))
        .collect::<Vec<_>>();
    if pattern_roots.is_empty() {
        pattern_roots = input_paths
            .iter()
            .filter_map(|input| cwd.join(input).parent().map(Path::to_path_buf))
            .collect();
    }
    let root = common_path_root(pattern_roots)?;
    let mut outputs = std::collections::BTreeMap::new();
    for input_path in input_paths {
        let absolute = cwd.join(input_path);
        let relative = absolute
            .strip_prefix(&root)
            .map_err(|_| format!("compiler input is outside its output root: {input_path}"))?;
        let relative_input = slash_path(relative);
        let output_path = relative_input
            .strip_suffix(".vue")
            .map(|prefix| format!("{prefix}.json"))
            .unwrap_or(relative_input);
        if outputs.insert(output_path, input_path.clone()).is_some() {
            return Err("compiler inputs map to duplicate output paths".to_string());
        }
    }
    Ok(outputs)
}

fn compiler_input_root(cwd: &Path, pattern: &str) -> Option<PathBuf> {
    let literal = cwd.join(pattern);
    if literal.exists() {
        if literal.is_file() {
            return literal.parent().map(Path::to_path_buf);
        }
        if literal.is_dir() {
            return Some(literal);
        }
        return None;
    }

    let normalized = pattern.replace('\\', "/");
    let metacharacter = normalized.find(['*', '?', '['])?;
    let prefix = &normalized[..metacharacter];
    let root = prefix
        .rfind('/')
        .map(|separator| cwd.join(&prefix[..separator]))
        .unwrap_or_else(|| cwd.to_path_buf());
    root.is_dir().then_some(root)
}

fn common_path_root(paths: Vec<PathBuf>) -> Result<PathBuf, String> {
    let Some(mut root) = paths.first().cloned() else {
        return Ok(PathBuf::new());
    };
    while paths.iter().any(|path| !path.starts_with(&root)) {
        let parent = root
            .parent()
            .ok_or_else(|| "compiler inputs have no common output root".to_string())?;
        if parent == root {
            return Err("compiler inputs have no common output root".to_string());
        }
        root = parent.to_path_buf();
    }
    Ok(root)
}

fn validate_compiler_artifact(
    output_path: &str,
    artifact: &Value,
    input_path: &str,
) -> Result<(), String> {
    let object = artifact
        .as_object()
        .ok_or_else(|| format!("invalid compiler artifact envelope: {output_path}"))?;
    let actual_keys = object.keys().cloned().collect::<Vec<_>>();
    let mut sorted_keys = actual_keys.clone();
    sorted_keys.sort();
    let expected_keys = [
        "code",
        "css",
        "errors",
        "filename",
        "macro_artifacts",
        "script_lang",
        "warnings",
    ];
    if sorted_keys != expected_keys {
        return Err(format!(
            "invalid compiler artifact keys in {output_path}: {}",
            sorted_keys.join(", ")
        ));
    }
    let expected_filename = input_path.rsplit('/').next().unwrap_or(input_path);
    if artifact.get("filename").and_then(Value::as_str) != Some(expected_filename) {
        return Err(format!(
            "compiler filename mismatch in {output_path}: expected {expected_filename}, received {}",
            artifact
                .get("filename")
                .and_then(Value::as_str)
                .unwrap_or("<non-string>")
        ));
    }
    if !artifact.get("code").is_some_and(Value::is_string) {
        return Err(format!("invalid compiler code in {output_path}"));
    }
    let css = artifact.get("css").unwrap_or(&Value::Null);
    if !css.is_null() && !css.is_string() {
        return Err(format!("invalid compiler css in {output_path}"));
    }
    if !artifact.get("script_lang").is_some_and(Value::is_string) {
        return Err(format!("invalid compiler script_lang in {output_path}"));
    }
    for field in ["errors", "warnings"] {
        let Some(items) = artifact.get(field).and_then(Value::as_array) else {
            return Err(format!("invalid compiler {field} in {output_path}"));
        };
        if items.iter().any(|entry| !entry.is_string()) {
            return Err(format!("invalid compiler {field} in {output_path}"));
        }
    }
    if !artifact.get("macro_artifacts").is_some_and(Value::is_array) {
        return Err(format!("invalid compiler macro_artifacts in {output_path}"));
    }
    Ok(())
}

fn snapshot_formatter_inputs(cwd: &Path, patterns: &[String]) -> Result<String, String> {
    let input_paths = collect_vue_input_paths(cwd, patterns)?;
    let mut digest = Sha256::new();
    for input_path in input_paths {
        let absolute = cwd.join(&input_path);
        let metadata = fs::metadata(&absolute)
            .map_err(|error| format!("cannot stat {}: {error}", absolute.display()))?;
        if !metadata.is_file() {
            continue;
        }
        digest.update(input_path.as_bytes());
        digest.update(b"\0");
        digest.update(format!("{:?}", metadata.permissions()).as_bytes());
        digest.update(b"\0");
        digest.update(format!("{:?}", metadata.modified().ok()).as_bytes());
        digest.update(b"\0");
        digest.update(
            fs::read(&absolute)
                .map_err(|error| format!("cannot read {}: {error}", absolute.display()))?,
        );
        digest.update(b"\0");
    }
    let status = Command::new("git")
        .args([
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--ignore-submodules=none",
            "--",
            ".",
        ])
        .current_dir(cwd)
        .stdin(Stdio::null())
        .output()
        .map_err(|_| "failed to snapshot formatter working tree".to_string())?;
    if !status.status.success() {
        return Err("failed to snapshot formatter working tree".to_string());
    }
    digest.update(&status.stdout);
    Ok(hex_digest(digest))
}

fn validate_formatter_output(
    project: &Value,
    stdout: &str,
    stderr: &str,
    exit_code: i32,
    before: &str,
    after: &str,
    expected_files: Option<&[String]>,
) -> Result<Value, String> {
    if !stdout.is_empty() {
        formatter_invalid("stdout must be empty")?;
    }
    if before != after {
        formatter_invalid("formatter check modified its working tree or input metadata")?;
    }
    let normalized_stderr = stderr.replace("\r\n", "\n");
    if !normalized_stderr.ends_with('\n') {
        formatter_invalid("stderr must end with a newline")?;
    }
    if project.get("expectedVueFileCount").and_then(Value::as_u64) == Some(0) {
        let no_files_message = "No .vue, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, .tsx, .json, .jsonc, .yaml, .yml, .md, or .markdown files found matching the patterns";
        if normalized_stderr != format!("{no_files_message}\n") {
            formatter_invalid("zero-file fixture emitted an unexpected report")?;
        }
        if exit_code != 1 {
            formatter_invalid(&format!(
                "zero-file exit code {exit_code} does not match expected 1"
            ))?;
        }
        return Ok(create_formatter_change_evidence(0, &[]));
    }
    let lines = normalized_stderr
        .trim_end_matches('\n')
        .split('\n')
        .collect::<Vec<_>>();
    let found = parse_prefixed_count(lines.first().copied(), "Found ", " file(s)", "found count")?;
    if found == 0 {
        formatter_invalid("non-empty fixture formatted zero files")?;
    }
    if let Some(expected) = expected_files {
        if found != expected.len() {
            formatter_invalid(&format!(
                "found count {found} does not match {} inputs",
                expected.len()
            ))?;
        }
    }
    let mut changed_paths = Vec::new();
    let mut index = 1usize;
    while let Some(line) = lines
        .get(index)
        .and_then(|line| line.strip_prefix("Would reformat: "))
    {
        changed_paths.push(normalize_formatter_path(line)?);
        index += 1;
    }
    if lines.get(index).copied() != Some("") {
        formatter_invalid("missing blank line before formatter summary")?;
    }
    index += 1;
    let checked = parse_prefixed_count(
        lines.get(index).copied(),
        "Checked ",
        " file(s)",
        "checked count",
    )?;
    index += 1;
    let mut changed = 0usize;
    if let Some(line) = lines.get(index) {
        if let Some(value) = line
            .strip_prefix("  ")
            .and_then(|line| line.strip_suffix(" file(s) would be reformatted"))
        {
            changed = safe_count(value, "changed count")?;
            index += 1;
        }
    }
    let mut unchanged = 0usize;
    if let Some(line) = lines.get(index) {
        if let Some(value) = line
            .strip_prefix("  ")
            .and_then(|line| line.strip_suffix(" file(s) already formatted"))
        {
            unchanged = safe_count(value, "unchanged count")?;
            index += 1;
        }
    }
    if index != lines.len() {
        formatter_invalid("formatter report contains unexpected lines")?;
    }
    let mut unique = changed_paths.clone();
    unique.sort();
    unique.dedup();
    if unique.len() != changed_paths.len() {
        formatter_invalid("formatter report contains duplicate changed paths")?;
    }
    if let Some(expected) = expected_files {
        let unexpected = changed_paths
            .iter()
            .filter(|file| !expected.contains(file))
            .cloned()
            .collect::<Vec<_>>();
        if !unexpected.is_empty() {
            formatter_invalid(&format!(
                "changed files are not fixture inputs: {}",
                unexpected.join(", ")
            ))?;
        }
    }
    if changed != changed_paths.len() {
        formatter_invalid(&format!(
            "changed count {changed} does not match {} paths",
            changed_paths.len()
        ))?;
    }
    if found != checked || checked != changed + unchanged {
        formatter_invalid(&format!(
            "file counts do not reconcile: found {found}, checked {checked}, changed {changed}, unchanged {unchanged}"
        ))?;
    }
    let expected_exit_code = if changed > 0 { 1 } else { 0 };
    if exit_code != expected_exit_code {
        formatter_invalid(&format!(
            "exit code {exit_code} does not match expected {expected_exit_code}"
        ))?;
    }
    Ok(create_formatter_change_evidence(checked, &changed_paths))
}

fn create_formatter_change_evidence(checked_file_count: usize, changed_paths: &[String]) -> Value {
    let mut sorted = changed_paths.to_vec();
    sorted.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    let mut digest = Sha256::new();
    for input_path in &sorted {
        digest.update(input_path.as_bytes());
        digest.update(b"\0");
    }
    json!({
        "checkedFileCount": checked_file_count,
        "changedFileCount": changed_paths.len(),
        "unchangedFileCount": checked_file_count - changed_paths.len(),
        "changedPathsSha256": hex_digest(digest),
    })
}

fn parse_prefixed_count(
    line: Option<&str>,
    prefix: &str,
    suffix: &str,
    label: &str,
) -> Result<usize, String> {
    let Some(value) = line
        .and_then(|line| line.strip_prefix(prefix))
        .and_then(|line| line.strip_suffix(suffix))
    else {
        return formatter_invalid(&format!("missing {label}"));
    };
    safe_count(value, label)
}

fn safe_count(value: &str, label: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|_| format!("invalid formatter check output: {label} is not a safe integer"))
}

fn normalize_formatter_path(value: &str) -> Result<String, String> {
    let bare = value.strip_prefix("./").unwrap_or(value);
    if !is_normalized_relative_path(bare) || !bare.ends_with(".vue") {
        formatter_invalid(&format!(
            "changed file is not a normalized Vue SFC: {value}"
        ))?;
    }
    Ok(bare.to_string())
}

fn formatter_invalid<T>(message: &str) -> Result<T, String> {
    Err(format!("invalid formatter check output: {message}"))
}

fn validate_linter_output(
    project: &Value,
    output: &Value,
    exit_code: i32,
    expected_files: Option<&[String]>,
) -> Result<(), String> {
    let files = output
        .as_array()
        .ok_or_else(|| "invalid linter JSON output: envelope must be an array".to_string())?;
    if project.get("expectedVueFileCount").and_then(Value::as_u64) == Some(0) && !files.is_empty() {
        linter_invalid(&format!(
            "expected zero checked files, received {}",
            files.len()
        ))?;
    }
    if project.get("expectedVueFileCount").and_then(Value::as_u64) != Some(0) && files.is_empty() {
        linter_invalid("non-empty fixture linted zero Vue files")?;
    }
    if let Some(expected) = expected_files {
        if files.len() != expected.len() {
            linter_invalid(&format!(
                "checked file count {} does not match {} inputs",
                files.len(),
                expected.len()
            ))?;
        }
    }
    let mut seen_files = Vec::new();
    let mut total_errors = 0usize;
    for (file_index, file) in files.iter().enumerate() {
        let object = require_record(file, &format!("files[{file_index}]"), "linter")?;
        require_exact_keys(
            object,
            &["errorCount", "file", "messages", "warningCount"],
            &format!("files[{file_index}]"),
            "linter",
        )?;
        let file_path = file
            .get("file")
            .and_then(Value::as_str)
            .ok_or_else(|| "invalid linter JSON output: file must be a string".to_string())?;
        require_normalized_path(file_path, &format!("files[{file_index}].file"), "linter")?;
        if !file_path.ends_with(".vue") {
            linter_invalid(&format!("checked file is not a Vue SFC: {file_path}"))?;
        }
        if seen_files.iter().any(|seen| seen == file_path) {
            linter_invalid(&format!("duplicate file entry: {file_path}"))?;
        }
        seen_files.push(file_path.to_string());
        let expected_error_count = non_negative_json_int(
            &file["errorCount"],
            &format!("files[{file_index}].errorCount"),
            "linter",
        )?;
        let expected_warning_count = non_negative_json_int(
            &file["warningCount"],
            &format!("files[{file_index}].warningCount"),
            "linter",
        )?;
        let messages = file["messages"].as_array().ok_or_else(|| {
            format!("invalid linter JSON output: files[{file_index}].messages must be an array")
        })?;
        let mut error_count = 0usize;
        let mut warning_count = 0usize;
        for (message_index, message) in messages.iter().enumerate() {
            let label = format!("files[{file_index}].messages[{message_index}]");
            let object = require_record(message, &label, "linter")?;
            let expected_keys = if object.contains_key("help") {
                &[
                    "column",
                    "endColumn",
                    "endLine",
                    "help",
                    "line",
                    "message",
                    "ruleDocsPath",
                    "ruleId",
                    "severity",
                ][..]
            } else {
                &[
                    "column",
                    "endColumn",
                    "endLine",
                    "message",
                    "ruleDocsPath",
                    "ruleId",
                    "severity",
                ][..]
            };
            require_exact_keys(object, expected_keys, &label, "linter")?;
            for field in ["ruleId", "ruleDocsPath", "message"] {
                require_non_empty_string(&message[field], &format!("{label}.{field}"), "linter")?;
            }
            require_normalized_path(
                message["ruleDocsPath"].as_str().unwrap_or(""),
                &format!("{label}.ruleDocsPath"),
                "linter",
            )?;
            if object.contains_key("help") {
                require_non_empty_string(&message["help"], &format!("{label}.help"), "linter")?;
            }
            match message["severity"].as_i64() {
                Some(2) => error_count += 1,
                Some(1) => warning_count += 1,
                _ => linter_invalid(&format!("{label}.severity must be 1 or 2"))?,
            }
            let line = positive_json_int(&message["line"], &format!("{label}.line"), "linter")?;
            let column =
                positive_json_int(&message["column"], &format!("{label}.column"), "linter")?;
            let end_line =
                positive_json_int(&message["endLine"], &format!("{label}.endLine"), "linter")?;
            let end_column = positive_json_int(
                &message["endColumn"],
                &format!("{label}.endColumn"),
                "linter",
            )?;
            if end_line < line || (end_line == line && end_column < column) {
                linter_invalid(&format!("{label} has an inverted source range"))?;
            }
        }
        if expected_error_count != error_count {
            linter_invalid(&format!(
                "{file_path} errorCount {expected_error_count} does not match {error_count} messages"
            ))?;
        }
        if expected_warning_count != warning_count {
            linter_invalid(&format!(
                "{file_path} warningCount {expected_warning_count} does not match {warning_count} messages"
            ))?;
        }
        total_errors += error_count;
    }
    if let Some(expected) = expected_files {
        let missing = expected
            .iter()
            .filter(|file| !seen_files.contains(file))
            .cloned()
            .collect::<Vec<_>>();
        let unexpected = seen_files
            .iter()
            .filter(|file| !expected.contains(file))
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() || !unexpected.is_empty() {
            linter_invalid(&format!(
                "checked files do not match inputs: missing [{}], unexpected [{}]",
                missing.join(", "),
                unexpected.join(", ")
            ))?;
        }
    }
    let expected_exit_code = if total_errors > 0 { 1 } else { 0 };
    if exit_code != expected_exit_code {
        linter_invalid(&format!(
            "exit code {exit_code} does not match expected {expected_exit_code}"
        ))?;
    }
    Ok(())
}

fn linter_invalid<T>(message: &str) -> Result<T, String> {
    Err(format!("invalid linter JSON output: {message}"))
}

fn validate_typechecker_output(
    project: &Value,
    output: &Value,
    exit_code: i32,
    expected_files: Option<&[String]>,
    authored_files: &[String],
) -> Result<Value, String> {
    let object = require_record(output, "envelope", "typechecker")?;
    let expected_output_keys = if output.get("programs").is_some_and(Value::is_array) {
        &[
            "errorCount",
            "fileCount",
            "files",
            "programs",
            "warningCount",
        ][..]
    } else {
        &["errorCount", "fileCount", "files", "warningCount"][..]
    };
    require_exact_keys(object, expected_output_keys, "envelope", "typechecker")?;
    let output_error_count =
        non_negative_json_int(&output["errorCount"], "errorCount", "typechecker")?;
    let output_warning_count =
        non_negative_json_int(&output["warningCount"], "warningCount", "typechecker")?;
    let file_count = non_negative_json_int(&output["fileCount"], "fileCount", "typechecker")?;
    let files = output["files"]
        .as_array()
        .ok_or_else(|| "invalid typechecker JSON output: files must be an array".to_string())?;
    if file_count > files.len() {
        typechecker_invalid(&format!(
            "fileCount {file_count} exceeds {} file entries",
            files.len()
        ))?;
    }
    if project.get("expectedVueFileCount").and_then(Value::as_u64) == Some(0) && file_count != 0 {
        typechecker_invalid(&format!(
            "expected zero checked files, received {file_count}"
        ))?;
    }
    if project.get("expectedVueFileCount").and_then(Value::as_u64) != Some(0) && file_count == 0 {
        typechecker_invalid("non-empty fixture checked zero Vue files")?;
    }
    let mut seen_files = Vec::new();
    let mut error_count = 0usize;
    let mut warning_count = 0usize;
    for (index, file) in files.iter().enumerate() {
        let object = require_record(file, &format!("files[{index}]"), "typechecker")?;
        require_exact_keys(
            object,
            &["diagnostics", "file"],
            &format!("files[{index}]"),
            "typechecker",
        )?;
        let file_path = file["file"].as_str().ok_or_else(|| {
            format!("invalid typechecker JSON output: files[{index}].file must be a normalized relative path")
        })?;
        require_normalized_path(file_path, &format!("files[{index}].file"), "typechecker")?;
        if seen_files.iter().any(|seen| seen == file_path) {
            typechecker_invalid(&format!("duplicate file entry: {file_path}"))?;
        }
        seen_files.push(file_path.to_string());
        let diagnostics = file["diagnostics"].as_array().ok_or_else(|| {
            format!("invalid typechecker JSON output: files[{index}].diagnostics must be an array")
        })?;
        if index < file_count && expected_files.is_none() && !file_path.ends_with(".vue") {
            typechecker_invalid(&format!("checked file is not a Vue SFC: {file_path}"))?;
        }
        if index < file_count && !is_typecheck_source(file_path) {
            typechecker_invalid(&format!(
                "checked file has an unsupported typecheck extension: {file_path}"
            ))?;
        }
        if index >= file_count && diagnostics.is_empty() {
            typechecker_invalid(&format!(
                "project-level file entry has no diagnostics: {file_path}"
            ))?;
        }
        for (diagnostic_index, diagnostic) in diagnostics.iter().enumerate() {
            let diagnostic = diagnostic.as_str().ok_or_else(|| {
                format!("invalid typechecker JSON output: files[{index}].diagnostics[{diagnostic_index}] must be a non-empty string")
            })?;
            if diagnostic.is_empty() {
                typechecker_invalid(&format!(
                    "files[{index}].diagnostics[{diagnostic_index}] must be a non-empty string"
                ))?;
            }
            if diagnostic.starts_with("error:") {
                error_count += 1;
            } else if diagnostic.starts_with("warning:") {
                warning_count += 1;
            } else {
                typechecker_invalid(&format!(
                    "diagnostic has no error or warning prefix: {file_path}"
                ))?;
            }
        }
    }
    if let Some(programs) = output.get("programs").and_then(Value::as_array) {
        for (index, program) in programs.iter().enumerate() {
            let object = require_record(program, &format!("programs[{index}]"), "typechecker")?;
            let expected_keys = if program.get("tsconfig").is_some_and(Value::is_string) {
                &["compilerOptions", "files", "root", "tsconfig"][..]
            } else {
                &["files", "root"][..]
            };
            require_exact_keys(
                object,
                expected_keys,
                &format!("programs[{index}]"),
                "typechecker",
            )?;
            require_non_empty_string(
                &program["root"],
                &format!("programs[{index}].root"),
                "typechecker",
            )?;
            if object.contains_key("tsconfig") {
                require_non_empty_string(
                    &program["tsconfig"],
                    &format!("programs[{index}].tsconfig"),
                    "typechecker",
                )?;
            }
            if object.contains_key("compilerOptions") {
                require_record(
                    &program["compilerOptions"],
                    &format!("programs[{index}].compilerOptions"),
                    "typechecker",
                )?;
            }
            let program_files = program["files"].as_array().ok_or_else(|| {
                format!("invalid typechecker JSON output: programs[{index}].files must be an array")
            })?;
            for (file_index, file) in program_files.iter().enumerate() {
                let file = file.as_str().ok_or_else(|| {
                    format!("invalid typechecker JSON output: programs[{index}].files[{file_index}] must be a normalized relative path")
                })?;
                require_normalized_path(
                    file,
                    &format!("programs[{index}].files[{file_index}]"),
                    "typechecker",
                )?;
            }
        }
    }
    let checked_files = seen_files.into_iter().take(file_count).collect::<Vec<_>>();
    let mut sorted_files = checked_files.clone();
    sorted_files.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    if checked_files != sorted_files {
        typechecker_invalid("checked file entries are not sorted")?;
    }
    let mut requested_files = checked_files.clone();
    let mut transitive_authored_files = Vec::new();
    let mut transitive_dependency_files = Vec::new();
    if let Some(expected_files) = expected_files {
        validate_manifest_input(expected_files, "requested fixture inputs", is_vue_sfc)?;
        validate_manifest_input(
            authored_files,
            "authored fixture sources",
            is_typecheck_source,
        )?;
        let missing = expected_files
            .iter()
            .filter(|file| !checked_files.contains(file))
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            typechecker_invalid(&format!(
                "checked files are missing requested fixture inputs: [{}]",
                missing.join(", ")
            ))?;
        }
        let missing_authored_inputs = expected_files
            .iter()
            .filter(|file| !authored_files.contains(file))
            .cloned()
            .collect::<Vec<_>>();
        if !missing_authored_inputs.is_empty() {
            typechecker_invalid(&format!(
                "requested fixture inputs are not authored sources: [{}]",
                missing_authored_inputs.join(", ")
            ))?;
        }
        let transitive_files = checked_files
            .iter()
            .filter(|file| !expected_files.contains(file))
            .cloned()
            .collect::<Vec<_>>();
        transitive_authored_files = transitive_files
            .iter()
            .filter(|file| !is_dependency_source(file))
            .cloned()
            .collect();
        transitive_dependency_files = transitive_files
            .iter()
            .filter(|file| is_dependency_source(file))
            .cloned()
            .collect();
        let unclassified = transitive_authored_files
            .iter()
            .filter(|file| !authored_files.contains(file))
            .cloned()
            .collect::<Vec<_>>();
        if !unclassified.is_empty() {
            typechecker_invalid(&format!(
                "checked transitive files are not authored fixture sources: [{}]",
                unclassified.join(", ")
            ))?;
        }
        requested_files = expected_files.to_vec();
    }
    if output_error_count != error_count {
        typechecker_invalid(&format!(
            "errorCount {output_error_count} does not match {error_count} diagnostics"
        ))?;
    }
    if output_warning_count != warning_count {
        typechecker_invalid(&format!(
            "warningCount {output_warning_count} does not match {warning_count} diagnostics"
        ))?;
    }
    let expected_exit_code = if error_count > 0 { 1 } else { 0 };
    if exit_code != expected_exit_code {
        typechecker_invalid(&format!(
            "exit code {exit_code} does not match expected {expected_exit_code}"
        ))?;
    }
    Ok(json!({
        "schema": "vize.fixtureTypecheckerCoverage",
        "version": 2,
        "requested": create_manifest(&requested_files),
        "transitiveAuthored": create_manifest(&transitive_authored_files),
        "transitiveDependencies": create_manifest(&transitive_dependency_files),
        "checked": create_manifest(&checked_files),
    }))
}

fn summarize_typechecker_coverage(coverage: &Value) -> Result<Value, String> {
    let requested = coverage
        .get("requested")
        .ok_or_else(|| "typechecker coverage has no requested manifest".to_string())?;
    let transitive_authored = coverage
        .get("transitiveAuthored")
        .ok_or_else(|| "typechecker coverage has no transitiveAuthored manifest".to_string())?;
    let transitive_dependencies = coverage
        .get("transitiveDependencies")
        .ok_or_else(|| "typechecker coverage has no transitiveDependencies manifest".to_string())?;
    let checked = coverage
        .get("checked")
        .ok_or_else(|| "typechecker coverage has no checked manifest".to_string())?;
    Ok(json!({
        "requestedFileCount": requested["fileCount"],
        "requestedSha256": requested["sha256"],
        "transitiveAuthoredFileCount": transitive_authored["fileCount"],
        "transitiveAuthoredSha256": transitive_authored["sha256"],
        "transitiveDependencyFileCount": transitive_dependencies["fileCount"],
        "transitiveDependencySha256": transitive_dependencies["sha256"],
        "checkedFileCount": checked["fileCount"],
        "checkedSha256": checked["sha256"],
    }))
}

fn validate_manifest_input(
    files: &[String],
    label: &str,
    accepts_file: fn(&str) -> bool,
) -> Result<(), String> {
    let mut seen = Vec::new();
    for (index, file) in files.iter().enumerate() {
        require_normalized_path(file, &format!("{label}[{index}]"), "typechecker")?;
        if !accepts_file(file) {
            typechecker_invalid(&format!(
                "{label}[{index}] has an unsupported source extension"
            ))?;
        }
        if seen.contains(file) {
            typechecker_invalid(&format!("{label} contains duplicate file: {file}"))?;
        }
        seen.push(file.clone());
    }
    let mut sorted = files.to_vec();
    sorted.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    if files != sorted {
        typechecker_invalid(&format!("{label} are not sorted"))?;
    }
    Ok(())
}

fn create_manifest(files: &[String]) -> Value {
    let mut digest = Sha256::new();
    for file in files {
        digest.update(file.as_bytes());
        digest.update(b"\0");
    }
    json!({
        "fileCount": files.len(),
        "files": files,
        "sha256": hex_digest(digest),
    })
}

fn typechecker_invalid<T>(message: &str) -> Result<T, String> {
    Err(format!("invalid typechecker JSON output: {message}"))
}

fn require_record<'a>(
    value: &'a Value,
    label: &str,
    kind: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("invalid {kind} JSON output: {label} must be an object"))
}

fn require_exact_keys(
    object: &serde_json::Map<String, Value>,
    expected: &[&str],
    label: &str,
    kind: &str,
) -> Result<(), String> {
    let mut actual = object.keys().map(String::as_str).collect::<Vec<_>>();
    actual.sort();
    if actual != expected {
        return Err(format!(
            "invalid {kind} JSON output: {label} keys must be {}; received {}",
            expected.join(", "),
            actual.join(", ")
        ));
    }
    Ok(())
}

fn require_normalized_path(value: &str, label: &str, kind: &str) -> Result<(), String> {
    if !is_normalized_relative_path(value) {
        return Err(format!(
            "invalid {kind} JSON output: {label} must be a normalized relative path"
        ));
    }
    Ok(())
}

fn require_non_empty_string(value: &Value, label: &str, kind: &str) -> Result<(), String> {
    if !value.as_str().is_some_and(|value| !value.is_empty()) {
        return Err(format!(
            "invalid {kind} JSON output: {label} must be non-empty"
        ));
    }
    Ok(())
}

fn non_negative_json_int(value: &Value, label: &str, kind: &str) -> Result<usize, String> {
    value
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| {
            format!("invalid {kind} JSON output: {label} must be a non-negative safe integer")
        })
}

fn positive_json_int(value: &Value, label: &str, kind: &str) -> Result<usize, String> {
    non_negative_json_int(value, label, kind).and_then(|value| {
        if value > 0 {
            Ok(value)
        } else {
            Err(format!(
                "invalid {kind} JSON output: {label} must be a positive safe integer"
            ))
        }
    })
}

fn is_normalized_relative_path(value: &str) -> bool {
    !value.is_empty()
        && !Path::new(value).is_absolute()
        && !value.contains('\\')
        && !value.starts_with("./")
        && !value
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
}

fn is_vue_sfc(file: &str) -> bool {
    file.ends_with(".vue")
}

fn is_typecheck_source(file: &str) -> bool {
    ["vue", "ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs"]
        .iter()
        .any(|extension| file.ends_with(&format!(".{extension}")))
}

fn is_dependency_source(file: &str) -> bool {
    file.split('/').any(|segment| segment == "node_modules")
}

fn project_string_array(project: &Value, field: &str) -> Result<Vec<String>, String> {
    project
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("project is missing {field}"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("{field} entries must be strings"))
        })
        .collect()
}

fn display_command(command: &str, args: &[String]) -> String {
    let mut parts = vec![display_shell_quote(command)];
    parts.extend(args.iter().map(|arg| display_shell_quote(arg)));
    parts.join(" ")
}

fn display_shell_quote(value: &str) -> String {
    if !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_./:=@*-".contains(&byte))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn hex_digest(digest: Sha256) -> String {
    digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn error_message(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn merge_base(mut base: Value, extra: Value) -> Value {
    for (key, value) in extra.as_object().unwrap() {
        base[key] = value.clone();
    }
    base
}

fn validated_file_count(tool: &str, payload: &Value) -> Value {
    match tool {
        "compiler" => payload["compilerArtifacts"]["inputFileCount"].clone(),
        "typechecker" => payload["parsed"]["fileCount"].clone(),
        "linter" => payload["parsed"]
            .as_array()
            .map(|items| json!(items.len()))
            .unwrap_or(Value::Null),
        "formatter" => payload["formatterCheck"]["checkedFileCount"].clone(),
        _ => Value::Null,
    }
}

fn run_failure(payload: &Value) -> Value {
    if let Some(error) = payload.get("spawnError").and_then(Value::as_str) {
        return Value::String(error.to_string());
    }
    if let Some(error) = payload.get("validationError").and_then(Value::as_str) {
        return Value::String(error.to_string());
    }
    if let Some(error) = payload.get("parseError").and_then(Value::as_str) {
        return Value::String(format!("invalid JSON output: {error}"));
    }
    failure_output(payload)
}

fn failure_output(payload: &Value) -> Value {
    json!({
        "stdout": truncate(payload.get("stdout").and_then(Value::as_str).unwrap_or("")),
        "stderr": truncate(payload.get("stderr").and_then(Value::as_str).unwrap_or("")),
    })
}

fn truncate(value: &str) -> String {
    if value.len() <= 4000 {
        value.to_string()
    } else {
        format!("{}\n...<truncated>", &value[..4000])
    }
}

fn project_string(project: &Value, field: &str) -> Result<String, String> {
    project
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("project is missing {field}"))
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
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

fn timestamp_slug() -> String {
    chrono::Utc::now()
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        .replace([':', '.'], "-")
}

fn value_or_dash(value: &Value) -> String {
    if value.is_null() {
        "-".to_string()
    } else {
        value.to_string()
    }
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

fn node_platform() -> String {
    match env::consts::OS {
        "macos" => "darwin",
        "windows" => "win32",
        other => other,
    }
    .to_string()
}

fn node_arch() -> String {
    match env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    }
    .to_string()
}

fn cpu_model() -> String {
    common::read_optional_text("/proc/cpuinfo")
        .lines()
        .find_map(|line| {
            line.strip_prefix("model name")
                .and_then(|line| line.split_once(':'))
                .map(|(_, value)| value.trim().to_string())
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn total_memory_bytes() -> u64 {
    common::read_optional_text("/proc/meminfo")
        .lines()
        .find_map(|line| {
            let value = line
                .strip_prefix("MemTotal:")?
                .split_whitespace()
                .next()?
                .parse::<u64>()
                .ok()?;
            Some(value * 1024)
        })
        .unwrap_or(1)
}

fn repo_root() -> Result<PathBuf, String> {
    common::repo_root().or_else(|_| {
        Path::new(file!())
            .ancestors()
            .find(|candidate| {
                candidate.join("Cargo.toml").is_file()
                    && candidate.join("pnpm-workspace.yaml").is_file()
            })
            .map(Path::to_path_buf)
            .ok_or_else(|| "cannot resolve Vize repository root from script path".to_string())
    })
}
