#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! libc = "0.2"
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! sha2 = "0.10"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../support/common.rs"]
mod common;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, ExitCode, Stdio},
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[derive(Debug)]
struct Args {
    output_dir: PathBuf,
    registry: PathBuf,
    shard_count: usize,
    shard_index: usize,
    timeout_ms: u64,
}

#[derive(Clone, Debug)]
struct Runner {
    manager: String,
    command: String,
    prefix_args: Vec<String>,
    label: String,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let root = repo_root()?;
    let args = parse_args(&root, env::args().skip(1).collect())?;
    require_directory(&args.output_dir)?;
    let registry = common::read_json(&args.registry)?;
    let selected =
        select_typecheck_performance_projects(&registry, args.shard_index, args.shard_count)?;
    let commit_sha = require_commit_sha(&env::var("GITHUB_SHA").unwrap_or_default())?;
    if selected.is_empty() {
        println!(
            "No typecheck performance projects selected for shard {}/{}",
            args.shard_index, args.shard_count
        );
        return Ok(());
    }
    for project in selected {
        prepare_project_dependencies(&root, &args, &commit_sha, project)?;
    }
    Ok(())
}

fn parse_args(root: &Path, argv: Vec<String>) -> Result<Args, String> {
    let mut output_dir = None;
    let mut registry = root.join("tests/_fixtures/vue-ecosystem-fixtures.json");
    let mut shard_count = 1usize;
    let mut shard_index = 0usize;
    let mut timeout_ms = 600_000u64;
    let mut index = 0;
    while index < argv.len() {
        let arg = &argv[index];
        let mut value = || -> Result<String, String> {
            index += 1;
            argv.get(index)
                .cloned()
                .ok_or_else(|| format!("{arg} requires a value"))
        };
        match arg.as_str() {
            "--output-dir" => output_dir = Some(root.join(value()?)),
            "--registry" => registry = root.join(value()?),
            "--shard-count" => shard_count = integer(&value()?, arg, 1)? as usize,
            "--shard-index" => shard_index = integer(&value()?, arg, 0)? as usize,
            "--timeout-ms" => timeout_ms = integer(&value()?, arg, 1)?,
            _ => return Err(format!("Unknown argument: {arg}")),
        }
        index += 1;
    }
    if shard_index >= shard_count {
        return Err("--shard-index must be less than --shard-count".to_string());
    }
    Ok(Args {
        output_dir: output_dir.ok_or_else(|| "--output-dir is required".to_string())?,
        registry,
        shard_count,
        shard_index,
        timeout_ms,
    })
}

fn prepare_project_dependencies(
    root: &Path,
    args: &Args,
    commit_sha: &str,
    project: &Value,
) -> Result<(), String> {
    let id = project_string(project, "id")?;
    let fixture_root = root.join(project_string(project, "fixturePath")?);
    validate_typecheck_performance_target(project, &fixture_root, false)?;
    let performance = project
        .get("typecheckPerformance")
        .ok_or_else(|| format!("{id} has no typecheckPerformance"))?;
    let manager = project_string(performance, "packageManager")?;
    let manager_version = project_string(performance, "packageManagerVersion")?;
    let lockfile_rel = project_string(performance, "lockfile")?;
    let lockfile_path = fixture_root.join(&lockfile_rel);
    let lockfile_before = fs::read(&lockfile_path)
        .map_err(|error| format!("cannot read {}: {error}", lockfile_path.display()))?;
    require_clean_fixture(&fixture_root, "before dependency installation")?;
    let runner = package_manager_runner(&manager, &manager_version);
    let probe = run_package_manager(&runner, &["--version".to_string()], &fixture_root, 10_000)?;
    if probe.status != 0 {
        return Err(format!(
            "{} is not runnable: {}",
            runner.label,
            probe.stderr.trim()
        ));
    }
    let detected_version = probe.stdout.trim().to_string();
    if detected_version != manager_version {
        return Err(format!(
            "Detected {manager} version {detected_version} does not match {manager_version}"
        ));
    }
    let install_args = install_arguments(&manager)?;
    let started = Instant::now();
    let install = run_package_manager(&runner, &install_args, &fixture_root, args.timeout_ms)
        .map_err(|error| format!("{} install failed to run: {error}", runner.label))?;
    let duration_ms = started.elapsed().as_millis() as u64;
    if install.status != 0 {
        return Err(format!(
            "{} install exited with status {}",
            runner.label, install.status
        ));
    }
    let lockfile_after = fs::read(&lockfile_path)
        .map_err(|error| format!("cannot read {}: {error}", lockfile_path.display()))?;
    if lockfile_before != lockfile_after {
        return Err(format!(
            "{manager} install modified frozen lockfile {lockfile_rel}"
        ));
    }
    require_clean_fixture(&fixture_root, "after dependency installation")?;
    let baseline_prepare = run_baseline_prepare(project, &fixture_root, args.timeout_ms, &runner)?;
    validate_typecheck_performance_target(project, &fixture_root, true)?;
    require_clean_fixture(&fixture_root, "after baseline preparation")?;
    let install_command = [vec![manager.clone()], install_args.clone()].concat();
    let artifact = json!({
        "schema": "vize.fixtureTypecheckDependencyInstall",
        "version": 2,
        "project": id,
        "revision": project.get("revision").cloned().unwrap_or(Value::Null),
        "evidence": {
            "commitSha": commit_sha,
            "runtime": { "name": "rust-script", "version": rustc_version() },
        },
        "packageManager": { "name": manager, "version": detected_version },
        "lockfile": {
            "path": lockfile_rel,
            "sizeBytes": lockfile_after.len(),
            "sha256": sha256(&lockfile_after),
        },
        "install": {
            "command": install_command,
            "durationMs": duration_ms,
            "exitCode": install.status,
            "stdoutSha256": sha256(install.stdout.as_bytes()),
            "stderrSha256": sha256(install.stderr.as_bytes()),
        },
        "baselinePrepare": baseline_prepare,
    });
    let artifact_path = args
        .output_dir
        .join(format!("{id}-typecheck-dependencies.json"));
    common::write_json_pretty(&artifact_path, &artifact)?;
    println!("Wrote {}", common::relative_path(root, &artifact_path));
    Ok(())
}

fn select_typecheck_performance_projects(
    registry: &Value,
    shard_index: usize,
    shard_count: usize,
) -> Result<Vec<&Value>, String> {
    if shard_count == 0 || shard_index >= shard_count {
        return Err(format!(
            "Typecheck performance shard index must be in [0, {shard_count}), got {shard_index}"
        ));
    }
    let projects = registry
        .get("projects")
        .and_then(Value::as_array)
        .ok_or_else(|| "Fixture registry must list projects".to_string())?;
    Ok(projects
        .iter()
        .enumerate()
        .filter_map(|(index, project)| {
            (index % shard_count == shard_index
                && project
                    .get("typecheckPerformance")
                    .and_then(|value| value.get("enabled"))
                    .and_then(Value::as_bool)
                    == Some(true))
            .then_some(project)
        })
        .collect())
}

fn validate_typecheck_performance_target(
    project: &Value,
    fixture_root: &Path,
    require_baseline: bool,
) -> Result<(), String> {
    let Some(performance) = project.get("typecheckPerformance") else {
        return Ok(());
    };
    if performance.get("enabled").and_then(Value::as_bool) != Some(true) {
        return Ok(());
    }
    let id = project_string(project, "id")?;
    if performance.get("compareTo").and_then(Value::as_str) != Some("vue-tsc") {
        return Err(format!(
            "Invalid typecheck performance target for {id}: compareTo must be vue-tsc"
        ));
    }
    require_file(
        &id,
        fixture_root,
        project.get("tsconfig").and_then(Value::as_str),
        "tsconfig",
    )?;
    if let Some(corpus) = performance.get("corpusGlobs") {
        let corpus = corpus.as_array().ok_or_else(|| {
            format!(
                "Invalid typecheck performance target for {id}: corpusGlobs must be a non-empty array when present"
            )
        })?;
        if corpus.is_empty() {
            return Err(format!(
                "Invalid typecheck performance target for {id}: corpusGlobs must be a non-empty array when present"
            ));
        }
        for glob in corpus {
            require_relative_path(&id, glob.as_str(), "corpusGlobs entry", false)?;
        }
    }
    let manager = project_string(performance, "packageManager")?;
    let expected_lockfile = match manager.as_str() {
        "npm" => "package-lock.json",
        "pnpm" => "pnpm-lock.yaml",
        "yarn" => "yarn.lock",
        _ => {
            return Err(format!(
                "Invalid typecheck performance target for {id}: packageManager must be npm, pnpm, or yarn"
            ));
        }
    };
    if project_string(performance, "lockfile")? != expected_lockfile {
        return Err(format!(
            "Invalid typecheck performance target for {id}: lockfile must be {expected_lockfile} for {manager}"
        ));
    }
    require_file(&id, fixture_root, Some(expected_lockfile), "lockfile")?;
    if !RegexLike::semver(&project_string(performance, "packageManagerVersion")?) {
        return Err(format!(
            "Invalid typecheck performance target for {id}: packageManagerVersion must be an exact semantic version"
        ));
    }
    if let Some(baseline) = performance.get("baseline") {
        let baseline_tsconfig = baseline.get("tsconfig").and_then(Value::as_str);
        require_relative_path(&id, baseline_tsconfig, "baseline tsconfig", true)?;
        if baseline.get("prepare").is_none() || require_baseline {
            require_file(&id, fixture_root, baseline_tsconfig, "baseline tsconfig")?;
        }
        if let Some(prepare) = baseline.get("prepare") {
            let prepare = prepare.as_array().ok_or_else(|| {
                format!("Invalid typecheck performance target for {id}: baseline prepare must be a {manager} command argument array")
            })?;
            if prepare.len() < 2
                || prepare.first().and_then(Value::as_str) != Some(manager.as_str())
                || prepare.iter().any(|argument| {
                    argument
                        .as_str()
                        .is_none_or(|value| value.is_empty() || value.contains('\0'))
                })
            {
                return Err(format!(
                    "Invalid typecheck performance target for {id}: baseline prepare must be a {manager} command argument array"
                ));
            }
        }
    }
    Ok(())
}

fn run_baseline_prepare(
    project: &Value,
    fixture_root: &Path,
    timeout_ms: u64,
    runner: &Runner,
) -> Result<Value, String> {
    let Some(command) = project
        .get("typecheckPerformance")
        .and_then(|value| value.get("baseline"))
        .and_then(|value| value.get("prepare"))
        .and_then(Value::as_array)
    else {
        return Ok(Value::Null);
    };
    let command = command
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| "baseline prepare arguments must be strings".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let started = Instant::now();
    let result = (if command.first() == Some(&runner.manager) {
        run_package_manager(runner, &command[1..], fixture_root, timeout_ms)
    } else {
        run_command(&command[0], &command[1..], fixture_root, timeout_ms)
    })
    .map_err(|error| format!("baseline prepare failed to run: {error}"))?;
    let duration_ms = started.elapsed().as_millis() as u64;
    if result.status != 0 {
        return Err(format!(
            "baseline prepare exited with status {}",
            result.status
        ));
    }
    Ok(json!({
        "command": command,
        "durationMs": duration_ms,
        "exitCode": result.status,
        "stdoutSha256": sha256(result.stdout.as_bytes()),
        "stderrSha256": sha256(result.stderr.as_bytes()),
    }))
}

fn install_arguments(manager: &str) -> Result<Vec<String>, String> {
    match manager {
        "npm" => Ok([
            "ci",
            "--ignore-scripts",
            "--prefer-offline",
            "--no-audit",
            "--no-fund",
        ]
        .iter()
        .map(|value| (*value).to_string())
        .collect()),
        "pnpm" => Ok([
            "install",
            "--frozen-lockfile",
            "--ignore-scripts",
            "--prefer-offline",
        ]
        .iter()
        .map(|value| (*value).to_string())
        .collect()),
        "yarn" => Ok(["install", "--immutable", "--mode=skip-build"]
            .iter()
            .map(|value| (*value).to_string())
            .collect()),
        _ => Err(format!("unsupported package manager {manager}")),
    }
}

fn package_manager_runner(manager: &str, version: &str) -> Runner {
    if matches!(manager, "pnpm" | "yarn") {
        Runner {
            manager: manager.to_string(),
            command: "corepack".to_string(),
            prefix_args: vec![format!("{manager}@{version}")],
            label: format!("{manager}@{version}"),
        }
    } else {
        Runner {
            manager: manager.to_string(),
            command: manager.to_string(),
            prefix_args: Vec::new(),
            label: manager.to_string(),
        }
    }
}

fn run_package_manager(
    runner: &Runner,
    args: &[String],
    cwd: &Path,
    timeout_ms: u64,
) -> Result<common::CommandOutput, String> {
    let mut command_args = runner.prefix_args.clone();
    command_args.extend(args.iter().cloned());
    let mut command = Command::new(&runner.command);
    command.args(&command_args);
    if !runner.prefix_args.is_empty() {
        command.env("COREPACK_ENABLE_PROJECT_SPEC", "0");
    }
    run_prepared_command(command, cwd, timeout_ms)
}

fn run_command(
    command: &str,
    args: &[String],
    cwd: &Path,
    timeout_ms: u64,
) -> Result<common::CommandOutput, String> {
    let mut command = Command::new(command);
    command.args(args);
    run_prepared_command(command, cwd, timeout_ms)
}

fn run_prepared_command(
    mut command: Command,
    cwd: &Path,
    timeout_ms: u64,
) -> Result<common::CommandOutput, String> {
    command
        .current_dir(cwd)
        .env("CI", "true")
        .env("npm_config_ignore_scripts", "true")
        .env("YARN_ENABLE_SCRIPTS", "false")
        .env("NUXT_TELEMETRY_DISABLED", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    command.process_group(0);

    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to run command: {error}"))?;
    let stdout = read_child_pipe(child.stdout.take());
    let stderr = read_child_pipe(child.stderr.take());
    let timeout = Duration::from_millis(timeout_ms);
    let started = Instant::now();
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("failed to wait for command: {error}"))?
        {
            return Ok(common::CommandOutput {
                status: status.code().unwrap_or(1),
                stdout: join_pipe(stdout)?,
                stderr: join_pipe(stderr)?,
            });
        }
        if started.elapsed() >= timeout {
            terminate_child_tree(&mut child);
            return Err(format!("spawn timed out after {timeout_ms}ms"));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn read_child_pipe(pipe: Option<impl Read + Send + 'static>) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut text = String::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_string(&mut text);
        }
        text
    })
}

fn join_pipe(handle: thread::JoinHandle<String>) -> Result<String, String> {
    handle
        .join()
        .map_err(|_| "failed to read command output".to_string())
}

fn terminate_child_tree(child: &mut Child) {
    signal_child_tree(child, TerminationSignal::Term);
    let wait_started = Instant::now();
    while wait_started.elapsed() < Duration::from_millis(500) {
        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(_) => return,
        }
    }
    signal_child_tree(child, TerminationSignal::Kill);
    let _ = child.wait();
}

enum TerminationSignal {
    Term,
    Kill,
}

#[cfg(unix)]
fn signal_child_tree(child: &mut Child, signal: TerminationSignal) {
    let raw_signal = match signal {
        TerminationSignal::Term => libc::SIGTERM,
        TerminationSignal::Kill => libc::SIGKILL,
    };
    let pid = child.id() as i32;
    unsafe {
        libc::kill(-pid, raw_signal);
    }
}

#[cfg(not(unix))]
fn signal_child_tree(child: &mut Child, signal: TerminationSignal) {
    if matches!(signal, TerminationSignal::Kill) {
        let _ = child.kill();
    }
}

fn require_clean_fixture(fixture_root: &Path, phase: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["status", "--porcelain=v1", "--untracked-files=no"])
        .current_dir(fixture_root)
        .stdin(Stdio::null())
        .output()
        .map_err(|_| format!("Unable to inspect fixture source {phase}"))?;
    if !output.status.success() {
        return Err(format!("Unable to inspect fixture source {phase}"));
    }
    if !output.stdout.is_empty() {
        return Err(format!("Fixture tracked source changed {phase}"));
    }
    Ok(())
}

fn require_directory(path: &Path) -> Result<(), String> {
    let metadata = fs::metadata(path)
        .map_err(|_| format!("Output directory does not exist: {}", path.display()))?;
    if !metadata.is_dir() {
        return Err(format!(
            "Output path is not a directory: {}",
            path.display()
        ));
    }
    Ok(())
}

fn require_file(
    project_id: &str,
    fixture_root: &Path,
    target: Option<&str>,
    label: &str,
) -> Result<(), String> {
    require_relative_path(project_id, target, label, true)?;
    let target = target.unwrap();
    let path = fixture_root.join(target);
    if !path.is_file() {
        return Err(format!(
            "Invalid typecheck performance target for {project_id}: {label} does not exist: {target}"
        ));
    }
    Ok(())
}

fn require_relative_path(
    project_id: &str,
    target: Option<&str>,
    label: &str,
    reject_glob: bool,
) -> Result<(), String> {
    let Some(target) = target else {
        return Err(format!(
            "Invalid typecheck performance target for {project_id}: {label} must be a normalized relative path"
        ));
    };
    let bad = target.is_empty()
        || Path::new(target).is_absolute()
        || target.contains('\\')
        || target.starts_with("./")
        || target
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
        || (reject_glob && target.contains('*'));
    if bad {
        return Err(format!(
            "Invalid typecheck performance target for {project_id}: {label} must be a normalized relative path"
        ));
    }
    Ok(())
}

fn integer(value: &str, name: &str, minimum: u64) -> Result<u64, String> {
    value
        .parse::<u64>()
        .ok()
        .filter(|value| *value >= minimum)
        .ok_or_else(|| format!("{name} must be a safe integer >= {minimum}"))
}

fn require_commit_sha(value: &str) -> Result<String, String> {
    if value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(value.to_string())
    } else {
        Err("GITHUB_SHA must be a full lowercase commit SHA".to_string())
    }
}

fn sha256(value: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value);
    format!("{:x}", hasher.finalize())
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

fn project_string(value: &Value, field: &str) -> Result<String, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("{field} must be a string"))
}

struct RegexLike;

impl RegexLike {
    fn semver(value: &str) -> bool {
        let mut parts = value.splitn(2, ['-', '+']);
        let core = parts.next().unwrap_or("");
        let core_parts = core.split('.').collect::<Vec<_>>();
        core_parts.len() == 3
            && core_parts
                .iter()
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    }
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
