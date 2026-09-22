use super::{pr_ci, pr_contract, pr_github as github, pr_promote};
use serde_json::Value;
use std::{
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};

pub fn release(
    repository: &str,
    number: u64,
    tag: &str,
    bump: &str,
    base_version: &str,
    resume: bool,
    root: &Path,
) -> Result<(), String> {
    let started = Instant::now();
    let mut run_id = None;
    let mut attempted_resume = false;
    let mut promoted = false;
    loop {
        let head = github::git(&["rev-parse", "HEAD"], root)?;
        let candidate = github::candidate(repository, number, &head, tag, true, root)?;
        let main = github::fetch_main(root)?;
        if candidate.merged || github::tag_target(tag, root)?.as_deref() == Some(&head) {
            pr_ci::published_identity(&candidate, root)?;
            promoted = true;
        }
        if !candidate.merged && !promoted && github::parent(&head, root)? != main {
            if let Some(id) = run_id {
                let _ = github::output(
                    "gh",
                    &["run", "cancel", &format!("{id}"), "--repo", repository],
                    root,
                );
            }
            pr_promote::refresh(&candidate, bump, base_version, root)?;
            run_id = None;
            attempted_resume = false;
            continue;
        }
        let id = match run_id {
            Some(id) => id,
            None => {
                let id = find_or_dispatch(
                    repository,
                    number,
                    tag,
                    &head,
                    &candidate.branch,
                    candidate.merged,
                    root,
                )?;
                println!("Release validation: https://github.com/{repository}/actions/runs/{id}");
                run_id = Some(id);
                id
            }
        };
        let run = github::api(repository, &format!("actions/runs/{id}"), root)?;
        if run.get("status").and_then(Value::as_str) == Some("completed") {
            if candidate.merged && run.get("conclusion").and_then(Value::as_str) == Some("success")
            {
                let published = github::api(repository, &format!("releases/tags/{tag}"), root)?;
                if published.get("draft").and_then(Value::as_bool) != Some(false) {
                    return Err(
                        "The workflow passed but the GitHub Release is not published.".into(),
                    );
                }
                println!(
                    "Release complete: {}",
                    pr_contract::field(&published, "/html_url")?
                );
                return Ok(());
            }
            if resume && !attempted_resume {
                github::output(
                    "gh",
                    &[
                        "run",
                        "rerun",
                        &id.to_string(),
                        "--failed",
                        "--repo",
                        repository,
                    ],
                    root,
                )?;
                attempted_resume = true;
                sleep(Duration::from_secs(10));
                continue;
            }
            return Err(format!(
                "Release workflow failed: https://github.com/{repository}/actions/runs/{id}\nResume after fixing the failure: vp run release --resume {number}"
            ));
        }
        if !candidate.merged
            && !promoted
            && pr_contract::ready_job(&github::jobs(repository, id, root)?)?
            && pr_promote::checks_pass(&candidate, root)?
        {
            if !pr_promote::promote(&candidate, id, root)? {
                continue;
            }
            promoted = true;
        }
        if started.elapsed() > Duration::from_secs(28_800) {
            return Err(format!(
                "Release is still running. Resume with vp run release --resume {number}"
            ));
        }
        println!("PR #{number}: waiting for validation or publication (run {id}).");
        sleep(Duration::from_secs(20));
    }
}

fn find_or_dispatch(
    repository: &str,
    number: u64,
    tag: &str,
    head: &str,
    branch: &str,
    merged: bool,
    root: &Path,
) -> Result<u64, String> {
    let title = format!("Release {tag} PR #{number} @ {head}");
    if let Some(id) = find_run(repository, head, &title, root)? {
        return Ok(id);
    }
    if merged {
        return Err(
            "Merged release PR has no matching validation run; refusing an unverified publication."
                .into(),
        );
    }
    github::output(
        "gh",
        &[
            "workflow",
            "run",
            "release.yml",
            "--repo",
            repository,
            "--ref",
            branch,
            "--field",
            &format!("tag_name={tag}"),
            "--field",
            &format!("release_pr={number}"),
            "--field",
            &format!("expected_sha={head}"),
        ],
        root,
    )?;
    for _ in 0..30 {
        sleep(Duration::from_secs(2));
        if let Some(id) = find_run(repository, head, &title, root)? {
            return Ok(id);
        }
    }
    Err("Release workflow dispatch did not produce an identifiable run.".into())
}

fn find_run(repository: &str, head: &str, title: &str, root: &Path) -> Result<Option<u64>, String> {
    let response = github::api(
        repository,
        &format!(
            "actions/workflows/release.yml/runs?event=workflow_dispatch&head_sha={head}&per_page=100"
        ),
        root,
    )?;
    let runs = response
        .get("workflow_runs")
        .and_then(Value::as_array)
        .ok_or("Missing release runs")?;
    Ok(runs
        .iter()
        .filter(|run| {
            run.get("head_sha").and_then(Value::as_str) == Some(head)
                && run.get("display_title").and_then(Value::as_str) == Some(title)
                && run.get("event").and_then(Value::as_str) == Some("workflow_dispatch")
        })
        .filter_map(|run| run.get("id").and_then(Value::as_u64))
        .max())
}
