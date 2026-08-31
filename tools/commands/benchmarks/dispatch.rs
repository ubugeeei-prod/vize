#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

const BENCH_TASKS: &[(&str, &str)] = &[
    ("run", "tools/benchmarks/scripts/run.ts"),
    ("generate", "tools/benchmarks/scripts/generate.mjs"),
    ("lint", "tools/benchmarks/scripts/lint.ts"),
    ("fmt", "tools/benchmarks/scripts/fmt.ts"),
    ("check", "tools/benchmarks/scripts/check.ts"),
    ("vite", "tools/benchmarks/scripts/vite.ts"),
    ("musea", "tools/benchmarks/scripts/musea.mjs"),
    (
        "compare-tools",
        "tools/benchmarks/scripts/compare-tools.mjs",
    ),
];

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("benchmark-dispatch: {error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<u8, String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let Some((task, forwarded)) = args.split_first() else {
        print_usage();
        return Ok(1);
    };
    let Some(script) = BENCH_TASKS
        .iter()
        .find_map(|(name, script)| (*name == task).then_some(*script))
    else {
        eprintln!("Unknown benchmark task: {task}");
        return Ok(1);
    };

    let status = Command::new("node")
        .arg(script)
        .args(forwarded)
        .current_dir(repo_root()?)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| format!("failed to run node {script}: {error}"))?;

    Ok(status.code().unwrap_or(1).try_into().unwrap_or(1))
}

fn print_usage() {
    let tasks = BENCH_TASKS
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join("|");
    eprintln!("Usage: rust-script tools/commands/benchmarks/dispatch.rs -- <{tasks}> [args...]");
}

fn repo_root() -> Result<PathBuf, String> {
    let current =
        env::current_dir().map_err(|error| format!("cannot read current dir: {error}"))?;
    current
        .ancestors()
        .find(|candidate| is_repo_root(candidate))
        .map(Path::to_path_buf)
        .ok_or_else(|| format!("cannot find repository root from {}", current.display()))
}

fn is_repo_root(dir: &Path) -> bool {
    dir.join("Cargo.toml").is_file() && dir.join("pnpm-workspace.yaml").is_file()
}
