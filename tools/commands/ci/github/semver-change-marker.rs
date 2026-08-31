#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../../support/common.rs"]
mod common;

use serde_json::Value;
use std::{env, process::ExitCode, thread, time::Duration};

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let [output_path] = args.as_slice() else {
        return Err(
            "Usage: rust-script tools/commands/ci/github/semver-change-marker.rs <output-path> with GITHUB_EVENT_PATH"
                .to_string(),
        );
    };
    let event_path = env::var("GITHUB_EVENT_PATH").map_err(|_| {
        "Usage: rust-script tools/commands/ci/github/semver-change-marker.rs <output-path> with GITHUB_EVENT_PATH"
            .to_string()
    })?;
    let event = common::read_json(event_path)?;
    let marker = resolve_marker(
        env::var("GITHUB_API_URL").unwrap_or_else(|_| "https://api.github.com".to_string()),
        env::var("GITHUB_EVENT_NAME").unwrap_or_default(),
        env::var("GITHUB_REPOSITORY").ok(),
        env::var("GITHUB_SHA").ok(),
        env::var("GITHUB_TOKEN").ok(),
        &event,
    )?;
    common::write_text(output_path, &format!("{marker}\n"))
}

fn resolve_marker(
    api_url: String,
    event_name: String,
    repository: Option<String>,
    sha: Option<String>,
    token: Option<String>,
    event: &Value,
) -> Result<String, String> {
    if event_name == "pull_request" {
        return pull_request_marker(event.get("pull_request"));
    }
    if event_name != "push" {
        return Err(format!("Unsupported SemVer event: {event_name}"));
    }

    let push_sha = sha
        .filter(|value| !value.is_empty())
        .or_else(|| {
            event
                .get("after")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_default();
    let pull_request = merged_pull_request_for_commit(
        &api_url,
        repository.filter(|value| !value.is_empty()).or_else(|| {
            event
                .pointer("/repository/full_name")
                .and_then(Value::as_str)
                .map(str::to_string)
        }),
        &push_sha,
        token.filter(|value| !value.is_empty()),
    )?;
    match pull_request {
        Some(value) => pull_request_marker(Some(&value)),
        None => push_commit_marker(event),
    }
}

fn pull_request_marker(pull_request: Option<&Value>) -> Result<String, String> {
    let pull_request = pull_request
        .ok_or_else(|| "Pull request title is missing from the GitHub event".to_string())?;
    let title = pull_request
        .get("title")
        .and_then(Value::as_str)
        .ok_or_else(|| "Pull request title is missing from the GitHub event".to_string())?;
    let body = pull_request
        .get("body")
        .and_then(Value::as_str)
        .unwrap_or("");
    Ok(format!("{title}\n{body}"))
}

fn push_commit_marker(event: &Value) -> Result<String, String> {
    let mut messages = Vec::new();
    if let Some(commits) = event.get("commits").and_then(Value::as_array) {
        for commit in commits {
            if let Some(message) = commit.get("message").and_then(Value::as_str) {
                messages.push(message.to_string());
            }
        }
    }
    if let Some(head_message) = event
        .pointer("/head_commit/message")
        .and_then(Value::as_str)
    {
        if !messages.iter().any(|message| message == head_message) {
            messages.push(head_message.to_string());
        }
    }
    if messages.is_empty() {
        if event.get("deleted").and_then(Value::as_bool) == Some(true) {
            return Ok(String::new());
        }
        return Err("Push event contains no commit message for the SemVer fallback".to_string());
    }
    Ok(messages.join("\n"))
}

fn merged_pull_request_for_commit(
    api_url: &str,
    repository: Option<String>,
    sha: &str,
    token: Option<String>,
) -> Result<Option<Value>, String> {
    if !sha.is_empty() && sha.bytes().all(|byte| byte == b'0') {
        return Ok(None);
    }
    let repository = repository.ok_or_else(missing_push_metadata)?;
    let token = token.ok_or_else(missing_push_metadata)?;
    if sha.is_empty() {
        return Err(missing_push_metadata());
    }

    let url = format!(
        "{}/repos/{}/commits/{}/pulls",
        api_url.trim_end_matches('/'),
        repository,
        sha
    );
    let body = get_github_json_with_retry(&url, &token)?;
    let associated: Value = serde_json::from_str(&body)
        .map_err(|error| format!("GitHub associated-pulls response is invalid JSON: {error}"))?;
    let array = associated
        .as_array()
        .ok_or_else(|| "GitHub associated-pulls response is not an array".to_string())?;
    let exact = array
        .iter()
        .filter(|pull_request| {
            pull_request.get("merge_commit_sha").and_then(Value::as_str) == Some(sha)
                && pull_request.get("merged_at").is_some()
                && !pull_request.get("merged_at").unwrap().is_null()
        })
        .cloned()
        .collect::<Vec<_>>();
    if exact.len() > 1 {
        return Err(format!(
            "Commit {sha} has multiple exact merged pull requests"
        ));
    }
    Ok(exact.into_iter().next())
}

fn missing_push_metadata() -> String {
    "GITHUB_REPOSITORY, GITHUB_SHA, and GITHUB_TOKEN are required for push events".to_string()
}

fn get_github_json_with_retry(url: &str, token: &str) -> Result<String, String> {
    let delays = [100_u64, 200];
    for attempt in 0..=delays.len() {
        match get_github_json(url, token) {
            Ok((status, _)) if status >= 500 && status < 600 && attempt < delays.len() => {
                thread::sleep(Duration::from_millis(delays[attempt]));
            }
            Ok((status, body)) if status == 200 => return Ok(body),
            Ok((status, _)) => {
                return Err(format!(
                    "GitHub associated-pulls request failed with HTTP {status}"
                ));
            }
            Err(error) if attempt < delays.len() => {
                let _ = error;
                thread::sleep(Duration::from_millis(delays[attempt]));
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!("retry loop always returns")
}

fn get_github_json(url: &str, token: &str) -> Result<(u16, String), String> {
    let output = common::run_capture(
        "curl",
        &[
            "-sS",
            "-L",
            "-w",
            "\n%{http_code}",
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            &format!("Authorization: Bearer {token}"),
            "-H",
            "X-GitHub-Api-Version: 2022-11-28",
            url,
        ],
    )?;
    let (body, status) = output
        .stdout
        .rsplit_once('\n')
        .ok_or_else(|| "curl did not return an HTTP status".to_string())?;
    Ok((
        status
            .trim()
            .parse::<u16>()
            .map_err(|_| format!("curl returned malformed HTTP status: {status}"))?,
        body.to_string(),
    ))
}
