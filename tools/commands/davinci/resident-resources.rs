#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! [dependencies]
//! vize_tool_support = { path = "../../support" }
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! toml = "0.9"
//! ```
//! P5-4b's synthetic resident database resource probe. Alpha exports are fixture
//! inputs; this does not measure the LSP, Corsa or a production alpha producer.
//! Uses the same Linux process-tree sampler as TS-44, and compares absolute RSS
//! to the existing reference preset without changing its measured methodology.

use vize_tool_support::common;
mod resource_session;

use resource_session::ProcSampler;
use serde_json::{Value, json};
use std::{
    path::Path,
    process::{Command, ExitCode, Stdio},
    thread,
    time::{Duration, Instant},
};

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|at| args.get(at + 1))
        .cloned()
}

fn positive(args: &[String], name: &str, default: usize) -> Result<usize, String> {
    match flag(args, name) {
        Some(value) => value
            .parse()
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| format!("{name} needs a positive integer")),
        None => Ok(default),
    }
}

fn measure(
    binary: &Path,
    files: usize,
    edits: usize,
    sampler: &ProcSampler,
) -> Result<Value, String> {
    let mut child = Command::new(binary)
        .args([
            "--files",
            &files.to_string(),
            "--edits",
            &edits.to_string(),
            "--hold-seconds",
            "10",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    let start = Instant::now();
    let mut peak = 0u64;
    let mut tail = std::collections::VecDeque::new();
    let mut samples = 0usize;
    let mut max_processes = 0usize;
    loop {
        if let Some(sample) = sampler.sample(child.id()) {
            peak = peak.max(sample.rss_bytes);
            // Ignore an exited Linux zombie with zero RSS.
            if sample.rss_bytes > 0 {
                tail.push_back(sample.rss_bytes);
                if tail.len() > 100 {
                    tail.pop_front();
                }
            }
            samples += 1;
            max_processes = max_processes.max(sample.processes);
        }
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if start.elapsed() < Duration::from_secs(300) => {}
            result => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("resident probe did not finish: {result:?}"));
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    let output = child
        .wait_with_output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "resident probe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let last = tail.iter().copied().max().unwrap_or(0);
    let session: Value =
        serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())?;
    if session.get("files") != Some(&json!(files))
        || session.get("edits") != Some(&json!(edits))
        || [
            "stage_equivalence",
            "summary_equivalence",
            "stale_config_rejected",
        ]
        .iter()
        .any(|key| session.get(*key) != Some(&json!(true)))
        || session.get("body_edit_dependent_executions") != Some(&json!(0))
        || session.get("signature_edit_affected_consumers") != Some(&json!(1))
        || samples == 0
        || peak == 0
        || last == 0
    {
        return Err(format!(
            "incomplete resident session or RSS evidence: {session}"
        ));
    }
    Ok(
        json!({ "rss_peak_mib": peak as f64 / 1048576.0, "rss_idle_mib": last as f64 / 1048576.0,
        "samples": samples, "max_processes": max_processes, "elapsed_seconds": start.elapsed().as_secs_f64(), "session": session }),
    )
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let repo = common::repo_root()?;
    let binary =
        flag(&args, "--server").ok_or("--server <resource_session example binary> is required")?;
    let binary = Path::new(&binary)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let files = positive(&args, "--files", 10_000)?;
    let edits = positive(&args, "--edits", 32)?;
    let runs = positive(&args, "--runs", 3)?;
    let sampler =
        ProcSampler::new().ok_or("resident RSS measurement requires the Linux reference runner")?;
    let budgets: toml::Table = common::read_text(repo.join("docs/davinci/plan/budgets.toml"))?
        .parse()
        .map_err(|error: toml::de::Error| error.to_string())?;
    let preset = "linux-x64-ci";
    let ceiling = |metric: &str| -> Result<f64, String> {
        let value = budgets
            .get("resource")
            .and_then(|value| value.get(preset))
            .and_then(|value| value.get(metric))
            .and_then(|value| value.get("ceiling"))
            .and_then(|value| {
                value
                    .as_float()
                    .or_else(|| value.as_integer().map(|value| value as f64))
            });
        value
            .filter(|value| value.is_finite() && *value > 0.0)
            .ok_or_else(|| format!("missing positive {preset}.{metric} ceiling"))
    };
    let peak_cap = ceiling("rss_peak_mib")?;
    let idle_cap = ceiling("rss_idle_mib")?;
    let mut measurements = Vec::new();
    for run in 0..runs {
        let measurement = measure(&binary, files, edits, &sampler)?;
        eprintln!("resident resource run {}/{runs}: {measurement}", run + 1);
        measurements.push(measurement);
    }
    let maximum = |key: &str| {
        measurements
            .iter()
            .filter_map(|row| row.get(key).and_then(Value::as_f64))
            .fold(0.0, f64::max)
    };
    let (peak, idle) = (maximum("rss_peak_mib"), maximum("rss_idle_mib"));
    let report = json!({
        "scope": "synthetic resident database; fixture alpha inputs; no language server or Corsa",
        "reference_preset": preset, "files": files, "edits_per_run": edits, "runs": runs,
        "sampler": "Linux /proc/<pid>/stat RSS summed over the process tree every 50 ms",
        "idle_hold_seconds": 10, "idle_sampling": "maximum of the final 100 nonzero RSS samples (5 seconds)",
        "statistic": "maximum over fresh-process runs",
        "rss_peak_mib": peak, "rss_idle_mib": idle,
        "reference_rss_peak_ceiling_mib": peak_cap, "reference_rss_idle_ceiling_mib": idle_cap,
        "measurements": measurements,
    });
    let text = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    if let Some(out) = flag(&args, "--out") {
        common::write_text(&out, &format!("{text}\n"))?;
    }
    println!("{text}");
    if peak > peak_cap || idle > idle_cap {
        return Err(format!(
            "resident RSS exceeded the unchanged reference ceilings: peak {peak}/{peak_cap}, idle {idle}/{idle_cap} MiB"
        ));
    }
    Ok(())
}

fn main() -> ExitCode {
    common::main_result(run())
}
