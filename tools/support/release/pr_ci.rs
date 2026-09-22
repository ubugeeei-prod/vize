use super::{pr_contract, pr_github as github};
use std::{
    env,
    fs::OpenOptions,
    io::Write,
    path::Path,
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
