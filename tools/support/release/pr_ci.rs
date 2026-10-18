use super::{
    pr_budget::{self, Budget},
    pr_contract, pr_github as github,
};
use std::{
    env,
    fs::OpenOptions,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    thread::sleep,
    time::{Duration, Instant},
};

pub fn validate(number: u64, head: &str, tag: &str, root: &Path) -> Result<(), String> {
    let repository = github::repository(root)?;
    let candidate = github::candidate(&repository, number, head, tag, true, root)?;
    if github::git(&["rev-parse", "HEAD"], root)? != head {
        return Err("The workflow checkout must be the exact release PR head.".into());
    }
    if env::var("GITHUB_SHA").is_ok_and(|sha| sha != head) {
        return Err("The dispatched workflow SHA does not match its expected SHA.".into());
    }
    if format!("v{}", github::version(root)?) != tag {
        return Err("The release tag and workspace version disagree.".into());
    }
    let main = github::fetch_main(root)?;
    if candidate.merged {
        published_identity(&candidate, root)?;
    } else {
        pr_contract::current_parent(&github::parent(head, root)?, &main)?;
        if github::tag_target(tag, root)?.is_some() {
            return Err("An open release PR must not already have a remote tag.".into());
        }
    }
    if let Ok(path) = env::var("GITHUB_OUTPUT") {
        let mut file = OpenOptions::new()
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        writeln!(
            file,
            "tag={tag}\nhead={head}\nbase={}\npr={number}",
            github::parent(head, root)?
        )
        .map_err(|e| e.to_string())?;
    }
    println!(
        "Release PR #{number} by {} is authorized at {head}.",
        candidate.author
    );
    Ok(())
}

pub fn published_identity(candidate: &pr_contract::Candidate, root: &Path) -> Result<(), String> {
    if github::tag_target(&candidate.tag, root)?.as_deref() != Some(&candidate.head) {
        return Err("The immutable release tag does not point at the validated PR head.".into());
    }
    let history = github::git(&["rev-list", "--first-parent", "origin/main"], root)?;
    if !history.lines().any(|sha| sha == candidate.head) {
        return Err("The validated release commit is not on main's first-parent history.".into());
    }
    let main_cargo = github::git(&["show", "origin/main:Cargo.toml"], root)?;
    let section = main_cargo
        .split("[workspace.package]")
        .nth(1)
        .ok_or("Missing main workspace metadata")?;
    let version = section
        .lines()
        .map(str::trim)
        .find_map(|line| {
            line.strip_prefix("version = \"")
                .and_then(|s| s.strip_suffix('"'))
        })
        .ok_or("Missing main workspace version")?;
    if format!("v{version}") != candidate.tag {
        return Err("A newer release owns main; refusing to publish the older candidate.".into());
    }
    Ok(())
}

/// Publishing uses the artifacts already built in this workflow. No token that
/// can publish or create a tag is needed while the PR is being validated.
pub fn wait_for_promotion(number: u64, head: &str, tag: &str, root: &Path) -> Result<(), String> {
    let repository = github::repository(root)?;
    let started = Instant::now();
    loop {
        let candidate = github::candidate(&repository, number, head, tag, true, root)?;
        github::fetch_main(root)?;
        if candidate.merged {
            published_identity(&candidate, root)?;
            println!(
                "PR #{number} and {tag} promoted the validated commit {head}; publishing its existing artifacts."
            );
            return Ok(());
        }
        // A newer base or head invalidates this run; the release command
        // refreshes the candidate and dispatches a replacement run.
        if github::tag_target(tag, root)?.as_deref() != Some(head) {
            pr_contract::current_parent(
                &github::parent(head, root)?,
                &github::git(&["rev-parse", "origin/main"], root)?,
            )?;
        }
        if started.elapsed() > Duration::from_secs(20_400) {
            return Err("Timed out waiting for PR promotion; no publication was attempted.".into());
        }
        println!("Validated PR #{number}: waiting for atomic main/tag promotion.");
        sleep(Duration::from_secs(20));
    }
}

pub fn validate_pinned(number: u64, head: &str, tag: &str, root: &Path) -> Result<(), String> {
    super::pr_pin::validate(number, head, tag, root)
}

pub fn wait_for_promotion_pinned(
    number: u64,
    head: &str,
    tag: &str,
    root: &Path,
) -> Result<(), String> {
    super::pr_pin::wait_for_promotion(number, head, tag, root)
}

/// First-publication checks apply only to a new R. Existing R resumes return
/// before this function in both dispatch paths, preserving historical custody.
pub(super) fn corsa_first_publication(
    repository: &str,
    number: u64,
    branch: &str,
    head: &str,
    cut: &str,
    tag: &str,
    budget: Option<&Budget>,
    root: &Path,
) -> Result<(), String> {
    pr_budget::check(budget)?;
    let catalog = github::git(
        &[
            "--no-replace-objects",
            "show",
            &format!("{head}:tools/support/release/publish_crates/main.ts"),
        ],
        root,
    )?;
    // Older releases without the new catalogue keep their original operator.
    if !catalog.contains("\"vize_corsa") {
        return Ok(());
    }
    github::git(
        &["--no-replace-objects", "cat-file", "-e", &format!("{head}:tools/support/release/corsa_bootstrap/main.ts")],
        root,
    ).map_err(|_| "Corsa catalogue has no reviewed first-publication gate at H; no new R was dispatched. Prepare a qualified new source instead of bypassing the gate.".to_string())?;
    let mut command = Command::new("node");
    command
        .arg("tools/support/release/corsa_bootstrap/main.ts")
        .args([repository, &number.to_string(), branch, head, cut, tag])
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    pr_budget::child_command(&mut command, budget)?;
    if !command
        .status()
        .map_err(|error| error.to_string())?
        .success()
    {
        return Err("Corsa first-publication precondition is paused or failed. Preserve the same H and operator evidence; no new Release was dispatched.".into());
    }
    pr_budget::check(budget)
}

/// New H overlaps Q/R; legacy H keeps its original blocking qualification API.
pub(super) fn start_check(
    repository: &str,
    branch: &str,
    head: &str,
    cut: &str,
    budget: Option<&Budget>,
    root: &Path,
) -> Result<Option<u64>, String> {
    pr_budget::check(budget)?;
    let workflow = github::git(
        &[
            "--no-replace-objects",
            "show",
            &format!("{head}:.github/workflows/release.yml"),
        ],
        root,
    )?;
    let mut capability = Command::new("node");
    capability
        .args([
            "--input-type=module-typescript",
            "--eval",
            include_str!("qualify_check/capability.ts"),
        ])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    pr_budget::child_command(&mut capability, budget)?;
    let mut child = capability.spawn().map_err(|error| error.to_string())?;
    child
        .stdin
        .take()
        .ok_or("Missing workflow parser stdin")?
        .write_all(workflow.as_bytes())
        .map_err(|error| error.to_string())?;
    let parsed = child
        .wait_with_output()
        .map_err(|error| error.to_string())?;
    if !parsed.status.success() {
        return Err(
            "Immutable H has an unknown release dispatch declaration; no Q or R was dispatched"
                .into(),
        );
    }
    let overlap = match parsed.stdout.as_slice() {
        b"overlap\n" => true,
        b"legacy\n" => false,
        _ => return Err("Unknown release workflow capability receipt".into()),
    };
    pr_budget::check(budget)?;
    let mut command = Command::new("node");
    command.arg("tools/support/release/qualify_check/main.ts");
    if overlap {
        command.arg("--start");
    }
    command
        .args([repository, branch, head, cut])
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(if overlap {
            Stdio::piped()
        } else {
            Stdio::inherit()
        })
        .stderr(Stdio::inherit());
    pr_budget::child_command(&mut command, budget)?;
    let output = command
        .output()
        .map_err(|error| format!("Could not start exact-source Check: {error}"))?;
    if !output.status.success() {
        return Err("Exact-source Check could not start; no Release was dispatched. Preserve its source and evidence.".into());
    }
    pr_budget::check(budget)?;
    if !overlap {
        // This source's original four-argument API blocks through all Check gates.
        return Ok(None);
    }
    let receipt: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    started_check_deadline(&receipt, repository, branch, head, cut).map(Some)
}

fn started_check_deadline(
    receipt: &serde_json::Value,
    repository: &str,
    branch: &str,
    head: &str,
    cut: &str,
) -> Result<u64, String> {
    let object = receipt
        .as_object()
        .ok_or("Check startup receipt must be an object")?;
    if object.len() != 9
        || receipt["kind"] != "vize-release-check-start-v1"
        || receipt["repository"] != repository
        || receipt["branch"] != branch
        || receipt["head"] != head
        || receipt["cut"] != cut
        || receipt["createdAt"].as_str().is_none_or(str::is_empty)
    {
        return Err("Check startup receipt does not match the exact source".into());
    }
    for key in ["runId", "runAttempt", "qualificationDeadline"] {
        if receipt[key]
            .as_u64()
            .is_none_or(|n| n == 0 || n > 9_007_199_254_740_991)
        {
            return Err(format!("Check startup receipt has an invalid {key}"));
        }
    }
    receipt["qualificationDeadline"]
        .as_u64()
        .ok_or_else(|| "Missing qualification deadline".into())
}

#[cfg(test)]
mod tests {
    use super::started_check_deadline;
    use serde_json::json;

    #[test]
    fn startup_receipt_preserves_the_exact_source_and_refuses_foreign_or_malformed_fields() {
        let valid = json!({"kind":"vize-release-check-start-v1", "repository":"owner/repo",
            "branch":"release/v1.2.3", "head":"a".repeat(40), "cut":"b".repeat(40),
            "runId":71, "runAttempt":1, "createdAt":"2026-10-17T00:00:00Z", "qualificationDeadline":123456});
        let check = |value: &serde_json::Value| {
            started_check_deadline(
                value,
                "owner/repo",
                "release/v1.2.3",
                &"a".repeat(40),
                &"b".repeat(40),
            )
        };
        assert_eq!(check(&valid), Ok(123456));
        for (key, value) in [
            ("kind", json!("foreign")),
            ("repository", json!("other/repo")),
            ("branch", json!("main")),
            ("head", json!("c".repeat(40))),
            ("cut", json!("c".repeat(40))),
            ("createdAt", json!("")),
            ("runId", json!(0)),
            ("runId", json!("71")),
            ("runAttempt", json!(-1)),
            ("runAttempt", json!(1.5)),
            ("qualificationDeadline", json!(0)),
            ("qualificationDeadline", json!(9_007_199_254_740_992_u64)),
            ("qualificationDeadline", json!("123456")),
            ("foreign", json!(true)),
        ] {
            let mut changed = valid.clone();
            changed[key] = value;
            assert!(check(&changed).is_err(), "{key}");
        }
        let mut missing = valid.clone();
        missing
            .as_object_mut()
            .expect("receipt fixture must be an object")
            .remove("createdAt");
        assert!(check(&missing).is_err());
    }
}
