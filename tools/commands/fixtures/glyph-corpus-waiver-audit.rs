#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../support/common.rs"]
mod common;

use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

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
    let output = parse_args(env::args().skip(1).collect())?;
    let root = repo_root()?;
    let entries = load_known_violation_ledger(&root)?;
    let artifact = create_waiver_issue_audit(&entries)?;
    common::write_json_pretty(&output, &artifact)?;
    println!(
        "audited {} formatter waiver(s) across {} open Issue(s)",
        artifact["waiverCount"].as_u64().unwrap_or(0),
        artifact["issues"].as_array().map_or(0, Vec::len)
    );
    Ok(())
}

fn parse_args(args: Vec<String>) -> Result<PathBuf, String> {
    match args.as_slice() {
        [flag, output] if flag == "--output" && !output.starts_with('-') => Ok(PathBuf::from(output)),
        _ => Err("usage: rust-script tools/commands/fixtures/glyph-corpus-waiver-audit.rs --output <path>".to_string()),
    }
}

fn load_known_violation_ledger(root: &Path) -> Result<Vec<Value>, String> {
    let path = root.join("tests/_fixtures/glyph-corpus-known-violations.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let entries = common::read_json(path)?;
    entries
        .as_array()
        .cloned()
        .ok_or_else(|| "glyph-corpus-known-violations.json must be an array".to_string())
}

fn create_waiver_issue_audit(entries: &[Value]) -> Result<Value, String> {
    let mut issue_numbers = entries
        .iter()
        .filter_map(|entry| entry.get("trackingIssue").and_then(Value::as_u64))
        .collect::<Vec<_>>();
    issue_numbers.sort();
    issue_numbers.dedup();
    let mut counts = BTreeMap::new();
    for entry in entries {
        if let Some(number) = entry.get("trackingIssue").and_then(Value::as_u64) {
            *counts.entry(number).or_insert(0usize) += 1;
        }
    }
    let mut issues = Vec::new();
    for number in issue_numbers {
        let issue = resolve_github_issue(number)?;
        if issue.get("number").and_then(Value::as_u64) != Some(number)
            || issue.get("state").and_then(Value::as_str).is_none()
        {
            return Err(format!(
                "tracking Issue #{number} returned invalid evidence"
            ));
        }
        let state = issue
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_uppercase();
        if state != "OPEN" {
            return Err(format!("tracking Issue #{number} is {state}"));
        }
        issues.push(json!({
            "number": number,
            "state": state,
            "title": issue.get("title").cloned().unwrap_or(Value::Null),
            "url": issue.get("html_url").cloned().unwrap_or(Value::Null),
            "updatedAt": issue.get("updated_at").cloned().unwrap_or(Value::Null),
            "waiverCount": counts.get(&number).copied().unwrap_or(0),
        }));
    }
    Ok(json!({
        "schema": "vize.glyphCorpusWaiverIssueAudit",
        "version": 1,
        "repository": env::var("GITHUB_REPOSITORY").unwrap_or_else(|_| "ubugeeei-prod/vize".to_string()),
        "sourceCommit": env::var("GITHUB_SHA").ok(),
        "generatedAt": chrono_like_now(),
        "waiverCount": entries.len(),
        "issues": issues,
    }))
}

fn resolve_github_issue(number: u64) -> Result<Value, String> {
    let repository =
        env::var("GITHUB_REPOSITORY").unwrap_or_else(|_| "ubugeeei-prod/vize".to_string());
    let api_url =
        env::var("GITHUB_API_URL").unwrap_or_else(|_| "https://api.github.com".to_string());
    let url = format!(
        "{}/repos/{repository}/issues/{number}",
        api_url.trim_end_matches('/')
    );
    let mut command = Command::new("curl");
    command.args([
        "--fail-with-body",
        "--silent",
        "--show-error",
        "--header",
        "Accept: application/vnd.github+json",
        "--header",
        "X-GitHub-Api-Version: 2022-11-28",
    ]);
    if let Ok(token) = env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            command.args(["--header", &format!("Authorization: Bearer {token}")]);
        }
    }
    let output = command
        .arg(&url)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("tracking Issue #{number} lookup failed: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "tracking Issue #{number} lookup failed with HTTP {}",
            output.status.code().unwrap_or(1)
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("tracking Issue #{number} returned invalid JSON: {error}"))
}

fn chrono_like_now() -> String {
    let output = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .stdin(Stdio::null())
        .output();
    output
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
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
