#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{
    env,
    process::{Command, ExitCode, Stdio},
    thread::sleep,
    time::{Duration, Instant},
};

const DEFAULT_GIT_TIMEOUT_MS: u64 = 30_000;

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
    let args = env::args().skip(1).collect::<Vec<_>>();
    let [tag] = args.as_slice() else {
        return Err(
            "Usage: rust-script tools/commands/ci/github/release-local-guard.rs <release-tag>"
                .to_string(),
        );
    };
    let timeout = git_timeout()?;
    parse_release_version(tag)?;
    if git(["branch", "--show-current"], &[0], timeout)?
        .stdout
        .trim()
        != "main"
    {
        return Err("Releases must be prepared from the local main branch.".to_string());
    }
    if !git(["status", "--porcelain"], &[0], timeout)?
        .stdout
        .trim()
        .is_empty()
    {
        return Err(
            "There are uncommitted changes. Please commit or stash them first.".to_string(),
        );
    }

    git(
        [
            "fetch",
            "--quiet",
            "--no-tags",
            "origin",
            "+refs/heads/main:refs/remotes/origin/main",
        ],
        &[0],
        timeout,
    )?;
    if git(
        [
            "merge-base",
            "--is-ancestor",
            "HEAD",
            "refs/remotes/origin/main",
        ],
        &[0, 1],
        timeout,
    )?
    .status
        != 0
    {
        return Err("HEAD is not reachable from the current origin/main.".to_string());
    }

    let head = git(["rev-parse", "HEAD"], &[0], timeout)?
        .stdout
        .trim()
        .to_string();
    let remote_main = git(["rev-parse", "refs/remotes/origin/main"], &[0], timeout)?
        .stdout
        .trim()
        .to_string();
    if head != remote_main {
        return Err(
            "HEAD must exactly match the current origin/main before preparing a release."
                .to_string(),
        );
    }

    let tag_ref = format!("refs/tags/{tag}");
    if git(
        ["rev-parse", "--verify", "--quiet", &tag_ref],
        &[0, 1],
        timeout,
    )?
    .status
        == 0
    {
        return Err(format!("Tag {tag} already exists locally."));
    }
    if git(
        ["ls-remote", "--exit-code", "--tags", "origin", &tag_ref],
        &[0, 2],
        timeout,
    )?
    .status
        == 0
    {
        return Err(format!(
            "Remote tag {tag} already exists and release tags are immutable."
        ));
    }

    println!("Local release guard passed for {tag}.");
    Ok(())
}

struct CommandResult {
    status: i32,
    stdout: String,
}

fn git<const N: usize>(
    args: [&str; N],
    accepted: &[i32],
    timeout: Duration,
) -> Result<CommandResult, String> {
    let command_text = format!("git {}", args.join(" "));
    let mut child = Command::new("git")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("{command_text} failed to start: {error}"))?;
    let started_at = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started_at.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "{command_text} timed out after {}ms.",
                    timeout.as_millis()
                ));
            }
            Ok(None) => sleep(Duration::from_millis(5)),
            Err(error) => return Err(format!("{command_text} failed while waiting: {error}")),
        }
    }
    let output = child
        .wait_with_output()
        .map_err(|error| format!("{command_text} failed to collect output: {error}"))?;
    let status = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !accepted.contains(&status) {
        let detail = format!("{stdout}\n{stderr}").trim().to_string();
        return Err(
            [format!("{command_text} failed with exit {status}"), detail]
                .into_iter()
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
    Ok(CommandResult { status, stdout })
}

fn git_timeout() -> Result<Duration, String> {
    match env::var("VIZE_RELEASE_GUARD_GIT_TIMEOUT_MS") {
        Ok(value) => {
            let timeout_ms = value.parse::<u64>().map_err(|error| {
                format!("VIZE_RELEASE_GUARD_GIT_TIMEOUT_MS must be an integer: {error}")
            })?;
            if timeout_ms == 0 {
                return Err("VIZE_RELEASE_GUARD_GIT_TIMEOUT_MS must be greater than 0".to_string());
            }
            Ok(Duration::from_millis(timeout_ms))
        }
        Err(env::VarError::NotPresent) => Ok(Duration::from_millis(DEFAULT_GIT_TIMEOUT_MS)),
        Err(error) => Err(format!(
            "failed to read VIZE_RELEASE_GUARD_GIT_TIMEOUT_MS: {error}"
        )),
    }
}

fn parse_release_version(tag: &str) -> Result<(), String> {
    let version = tag.strip_prefix('v').unwrap_or(tag);
    let (core, suffix) = version
        .split_once('-')
        .map_or((version, None), |(core, suffix)| (core, Some(suffix)));
    let parts = core.split('.').collect::<Vec<_>>();
    if parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
        && suffix.is_none_or(|suffix| {
            !suffix.is_empty()
                && suffix
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        })
    {
        return Ok(());
    }
    Err(format!(
        "Release tag must look like vMAJOR.MINOR.PATCH[-PRERELEASE], got {tag}"
    ))
}
