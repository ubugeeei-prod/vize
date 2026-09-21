#!/usr/bin/env rust-script
//! TS-42 — incremental ≡ clean over a corpus (Davinci P5-9).
//!
//! Runs the committed edit scripts (`incremental-equivalence/scripts/*.edits`)
//! through the resident tier's salsa database, one long-lived database per
//! file, and compares every served artifact and diagnostic with a
//! from-scratch run after every step (`vize_resident::equivalence`).
//!
//! ```text
//! rust-script tools/commands/davinci/incremental-equivalence.rs --corpus-shard [--fixtures <dir>] [--seeded] [--report <path>]
//! rust-script tools/commands/davinci/incremental-equivalence.rs --fixtures <dir> [--seeded] [--report <path>]
//! ```
//!
//! Scope proof: files, steps, comparisons and compared blocks are counted,
//! and a run that compared nothing fails. `--seeded` builds the harness with
//! the `seeded-stale-cache` defect and passes only when TS-42 catches it.
//! Exit status: 0 pass, 1 TS-42 failed, 2 usage or setup error.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// The P0-13 corpus shard (`tools/support/davinci/fpfn.rs`).
const CORPUS_SHARD: [&str; 3] = ["splitpanes", "layoutit-grid", "cssgridgenerator"];
const SCRIPTS: &str = "tools/commands/davinci/incremental-equivalence/scripts";
const USAGE: &str = "usage: rust-script tools/commands/davinci/incremental-equivalence.rs (--corpus-shard | --fixtures <dir>)... [--seeded] [--report <path>]";

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<bool, String> {
    let root = std::env::current_dir().map_err(|error| error.to_string())?;
    if !root.join(SCRIPTS).is_dir() {
        return Err(format!("run from the repository root ({SCRIPTS} not found)\n{USAGE}"));
    }
    let mut roots: Vec<(String, PathBuf)> = Vec::new();
    let mut seeded = false;
    let mut report_path = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--corpus-shard" => {
                for id in CORPUS_SHARD {
                    let dir = root.join("tests/_fixtures/_git").join(id);
                    if !has_vue_file(&dir) {
                        return Err(format!(
                            "corpus shard project {id} is not hydrated. Run:\n  git submodule update --init --depth 1 -- tests/_fixtures/_git/{id}"
                        ));
                    }
                    roots.push((id.to_string(), dir));
                }
            }
            "--fixtures" => {
                let dir = PathBuf::from(args.next().ok_or(USAGE)?);
                let label = dir.to_string_lossy().replace('/', "_");
                roots.push((label, root.join(dir)));
            }
            "--seeded" => seeded = true,
            "--report" => report_path = Some(PathBuf::from(args.next().ok_or(USAGE)?)),
            "--help" | "-h" => {
                println!("{USAGE}");
                return Ok(true);
            }
            other => return Err(format!("unknown argument {other}\n{USAGE}")),
        }
    }
    if roots.is_empty() {
        return Err(USAGE.to_string());
    }

    let mut command = Command::new("cargo");
    command.current_dir(&root).args([
        "run",
        "--release",
        "--locked",
        "-p",
        "vize_resident",
        "--example",
        "incremental_equivalence",
    ]);
    if seeded {
        command.args(["--features", "seeded-stale-cache"]);
    }
    command.args(["--", "--scripts", SCRIPTS]);
    for (label, dir) in &roots {
        command.arg("--root").arg(format!("{label}={}", dir.display()));
    }
    let output = command.output().map_err(|error| format!("cannot run cargo: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let counts = parse_counts(&stdout);
    let code = output.status.code();
    if let Some(path) = &report_path {
        std::fs::write(path, stdout.as_bytes()).map_err(|error| error.to_string())?;
    }
    print!("{stdout}");

    let count = |key: &str| counts.get(key).copied().unwrap_or(0);
    let scope = ["files", "steps_applied", "comparisons", "blocks_compared"]
        .into_iter()
        .all(|key| count(key) > 0);
    if !scope {
        eprintln!("{stderr}");
        eprintln!("TS-42 FAIL: the run compared nothing (scope proof missing)");
        return Ok(false);
    }
    let mismatches = count("mismatches");
    if seeded {
        let caught = code == Some(1) && mismatches > 0;
        println!(
            "TS-42 seeded stale cache: {} ({mismatches} stale states)",
            if caught { "caught" } else { "MISSED" }
        );
        return Ok(caught);
    }
    let pass = code == Some(0) && mismatches == 0;
    if !pass {
        eprintln!("{stderr}");
    }
    println!(
        "TS-42 {}: {} files, {} steps, {} comparisons, {} blocks, {mismatches} mismatches",
        if pass { "PASS" } else { "FAIL" },
        count("files"),
        count("steps_applied"),
        count("comparisons"),
        count("blocks_compared"),
    );
    Ok(pass)
}

fn parse_counts(stdout: &str) -> BTreeMap<String, u64> {
    stdout
        .lines()
        .filter_map(|line| line.split_once('='))
        .filter_map(|(key, value)| Some((key.to_string(), value.parse().ok()?)))
        .collect()
}

fn has_vue_file(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let path = entry.path();
        if path.is_dir() {
            has_vue_file(&path)
        } else {
            path.extension().is_some_and(|ext| ext == "vue")
        }
    })
}
