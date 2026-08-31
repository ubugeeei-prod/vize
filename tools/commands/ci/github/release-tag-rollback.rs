#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../../rust/common.rs"]
mod common;

use std::{env, process::ExitCode};

#[derive(Debug)]
struct Context {
    repository: String,
    sha: String,
    tag: String,
    token: String,
}

#[derive(Clone, Debug)]
struct RemoteTag {
    commit_sha: String,
    object_sha: String,
}

fn main() -> ExitCode {
    match rollback_unpublished_tag() {
        Ok(result) => {
            println!("{result}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn rollback_unpublished_tag() -> Result<String, String> {
    let context = assert_rollback_context()?;
    let tag_ref = format!("refs/tags/{}", context.tag);
    let remote = remote_tag_state(
        &git(&[
            "ls-remote",
            "--tags",
            "origin",
            &tag_ref,
            &format!("{tag_ref}^{{}}"),
        ])?
        .stdout,
        &context.tag,
    )?;
    let Some(remote) = remote else {
        return Ok(format!("Release tag {} was already absent.", context.tag));
    };
    if remote.commit_sha != context.sha {
        return Err(format!(
            "Refusing to delete {}: remote tag resolves to {}, not event SHA {}.",
            context.tag, remote.commit_sha, context.sha
        ));
    }

    assert_no_github_release(&context)?;
    git(&[
        "fetch",
        "--quiet",
        "--force",
        "origin",
        &format!("{tag_ref}:{tag_ref}"),
    ])?;
    let local_object_sha = git(&["rev-parse", &tag_ref])?.stdout.trim().to_string();
    let local_commit_sha = git(&["rev-parse", &format!("{tag_ref}^{{}}")])?
        .stdout
        .trim()
        .to_string();
    if local_object_sha != remote.object_sha || local_commit_sha != context.sha {
        return Err(format!(
            "Refusing to delete {}: fetched tag identity does not match the audited remote tag.",
            context.tag
        ));
    }

    assert_no_github_release(&context)?;
    git(&[
        "push",
        &format!("--force-with-lease={tag_ref}:{}", remote.object_sha),
        "origin",
        &format!(":{tag_ref}"),
    ])?;
    Ok(format!(
        "Rolled back unpublished release tag {}.",
        context.tag
    ))
}

fn assert_rollback_context() -> Result<Context, String> {
    let tag = env::var("GITHUB_REF_NAME").unwrap_or_default();
    let sha = env::var("GITHUB_SHA").unwrap_or_default();
    let repository = env::var("GITHUB_REPOSITORY").unwrap_or_default();
    let preflight_result = env::var("RELEASE_PREFLIGHT_RESULT").unwrap_or_default();
    let token = env::var("GITHUB_TOKEN").unwrap_or_default();

    if env::var("GITHUB_REF_TYPE").unwrap_or_default() != "tag" {
        return Err(format!(
            "Release rollback requires a tag event, got {}.",
            env::var("GITHUB_REF_TYPE").unwrap_or_else(|_| "unknown".to_string())
        ));
    }
    parse_release_version(&tag)?;
    if !is_strict_v_tag(&tag) {
        return Err(format!(
            "Release rollback requires a v-prefixed release tag, got {tag}."
        ));
    }
    if !is_full_sha(&sha) {
        return Err(format!(
            "Release rollback requires a full event SHA, got {sha}."
        ));
    }
    if !is_owner_repo(&repository) {
        return Err(format!(
            "Release rollback requires an owner/repository identity, got {repository}."
        ));
    }
    if !matches!(preflight_result.as_str(), "failure" | "cancelled") {
        return Err(format!(
            "Refusing to roll back {tag}: release preflight concluded {}.",
            if preflight_result.is_empty() {
                "unknown"
            } else {
                &preflight_result
            }
        ));
    }
    if token.is_empty() {
        return Err(
            "Release rollback requires GITHUB_TOKEN to check for an existing release.".to_string(),
        );
    }
    Ok(Context {
        repository,
        sha,
        tag,
        token,
    })
}

fn remote_tag_state(output: &str, tag: &str) -> Result<Option<RemoteTag>, String> {
    let tag_ref = format!("refs/tags/{tag}");
    let peeled_ref = format!("{tag_ref}^{{}}");
    let mut object_sha = None;
    let mut commit_sha = None;
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        let [sha, reference] = parts.as_slice() else {
            continue;
        };
        if *reference == tag_ref {
            object_sha = Some((*sha).to_string());
        } else if *reference == peeled_ref {
            commit_sha = Some((*sha).to_string());
        }
    }
    let Some(object_sha) = object_sha else {
        return Ok(None);
    };
    let commit_sha = commit_sha.unwrap_or_else(|| object_sha.clone());
    if !is_full_sha(&object_sha) || !is_full_sha(&commit_sha) {
        return Err(format!("Remote tag {tag} returned malformed object IDs."));
    }
    Ok(Some(RemoteTag {
        commit_sha,
        object_sha,
    }))
}

fn assert_no_github_release(context: &Context) -> Result<(), String> {
    let api_url =
        env::var("GITHUB_API_URL").unwrap_or_else(|_| "https://api.github.com".to_string());
    let url = format!(
        "{}/repos/{}/releases/tags/{}",
        api_url.trim_end_matches('/'),
        context.repository,
        context.tag
    );
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
            &format!("Authorization: Bearer {}", context.token),
            "-H",
            "X-GitHub-Api-Version: 2022-11-28",
            &url,
        ],
    )?;
    let (body, status) = output
        .stdout
        .rsplit_once('\n')
        .ok_or_else(|| "curl did not return an HTTP status".to_string())?;
    match status.trim() {
        "404" => Ok(()),
        "200" => Err(format!(
            "Refusing to delete {}: a GitHub Release already exists.",
            context.tag
        )),
        other => Err(format!(
            "Could not prove that {} is unpublished: GitHub Releases API returned {}{}.",
            context.tag,
            other,
            if body.trim().is_empty() {
                String::new()
            } else {
                format!(": {}", body.trim())
            }
        )),
    }
}

fn git(args: &[&str]) -> Result<common::CommandOutput, String> {
    common::run_capture("git", args)
}

fn parse_release_version(ref_name: &str) -> Result<(), String> {
    let name = ref_name.strip_prefix('v').unwrap_or(ref_name);
    let stable = name.split_once('-').map(|(value, _)| value).unwrap_or(name);
    let parts = stable.split('.').collect::<Vec<_>>();
    let [major, minor, patch] = parts.as_slice() else {
        return Err(format!(
            "Release tag must look like vMAJOR.MINOR.PATCH[-PRERELEASE], got {ref_name}"
        ));
    };
    major
        .parse::<u64>()
        .map_err(|_| invalid_release(ref_name))?;
    minor
        .parse::<u64>()
        .map_err(|_| invalid_release(ref_name))?;
    patch
        .parse::<u64>()
        .map_err(|_| invalid_release(ref_name))?;
    Ok(())
}

fn invalid_release(ref_name: &str) -> String {
    format!("Release tag must look like vMAJOR.MINOR.PATCH[-PRERELEASE], got {ref_name}")
}

fn is_strict_v_tag(value: &str) -> bool {
    value.strip_prefix('v').is_some_and(|rest| {
        let stable = rest.split_once('-').map(|(value, _)| value).unwrap_or(rest);
        let parts = stable.split('.').collect::<Vec<_>>();
        matches!(parts.as_slice(), [major, minor, patch] if is_u64(major) && is_u64(minor) && is_u64(patch))
    })
}

fn is_u64(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn is_full_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_owner_repo(value: &str) -> bool {
    let parts = value.split('/').collect::<Vec<_>>();
    matches!(parts.as_slice(), [owner, repo] if is_repo_part(owner) && is_repo_part(repo))
}

fn is_repo_part(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
}
