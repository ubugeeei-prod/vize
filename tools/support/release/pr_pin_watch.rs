use super::super::{
    pr_budget::{self, Budget},
    pr_contract, pr_github as github, pr_promote,
};
use super::{Source, body, candidate, delivery, dispatch, lock, queue, run_identity, runs, source};
use serde_json::{Value, json};
use std::{
    env, fs,
    io::Write,
    path::Path,
    time::{Duration, Instant},
};

fn early_integration_candidate(
    source: &Source,
    root: &Path,
) -> Result<pr_contract::Candidate, String> {
    let repository = &source.candidate.repository;
    let integration = source.integration.to_string();
    let pr = github::api(repository, &format!("pulls/{integration}"), root)?;
    let head = pr_contract::field(&pr, "/head/sha")?;
    let identity = candidate::integration_fields(&pr, repository, source.integration, head)?;
    if identity
        != (
            source.candidate.number,
            source.candidate.head.clone(),
            source.cut.clone(),
            source.candidate.tag.clone(),
        )
    {
        return Err("Early integration admission does not bind the exact immutable cut".into());
    }
    body::qualify(
        pr_contract::field(&pr, "/body")?,
        &body::generated(identity.0, &identity.2, &identity.1, &identity.3),
        repository,
        source.integration,
    )?;
    pr_contract::maintainer(&github::author(
        repository,
        pr_contract::field(&pr, "/user/login")?,
        root,
    )?)?;
    let mut candidate = source.candidate.clone();
    candidate.number = source.integration;
    candidate.head = head.to_owned();
    candidate.branch = format!("release-integration/{}", candidate.tag);
    candidate.author = pr_contract::field(&pr, "/user/login")?.to_owned();
    candidate.base = pr_contract::field(&pr, "/base/sha")?.to_owned();
    pr_contract::sha(&candidate.base)?;
    Ok(candidate)
}

/// Pending jobs keep the original overlap. A known failure cannot wait behind
/// unrelated artifact builds or a candidate-ready job that has not appeared.
pub(super) fn release_progress(
    run: &Value,
    jobs: &[Value],
    pinned: &Source,
    id: u64,
    attempt: u64,
) -> Result<(), String> {
    run_identity(run, pinned, id)?;
    if attempt == 0 || run["run_attempt"].as_u64() != Some(attempt) {
        return Err("Original Release attempt changed".into());
    }
    if run["status"] == "completed" && run["conclusion"] != "success" {
        return Err(format!(
            "Pinned R={id} failed; preserve its original source and artifacts"
        ));
    }
    if !matches!(
        run["status"].as_str(),
        Some("completed" | "queued" | "in_progress" | "requested" | "waiting" | "pending")
    ) || (run["status"] != "completed" && !run["conclusion"].is_null())
    {
        return Err("Unknown Release progress state".into());
    }
    let mut seen = std::collections::BTreeSet::new();
    for job in jobs {
        if job["run_id"].as_u64() != Some(id)
            || job["head_sha"] != pinned.candidate.head
            || job["run_attempt"]
                .as_u64()
                .is_none_or(|a| a == 0 || a > attempt)
            || job["id"]
                .as_u64()
                .is_none_or(|id| id == 0 || !seen.insert(id))
        {
            return Err("Release jobs do not bind the exact original run/head/attempt".into());
        }
        if !matches!(
            job["status"].as_str(),
            Some("completed" | "queued" | "in_progress" | "waiting" | "pending")
        ) || (job["status"] != "completed" && !job["conclusion"].is_null())
        {
            return Err("Unknown Release job progress state".into());
        }
        if job["status"] == "completed"
            && job["conclusion"] != "success"
            && job["conclusion"] != "skipped"
        {
            return Err(format!(
                "Pinned R={id} job {} did not succeed: {}; preserve its original evidence",
                job["name"], job["conclusion"]
            ));
        }
    }
    Ok(())
}

/// A rerun acknowledgement authorizes only the next attempt. Old GET responses
/// may propagate during the existing ten-second resume window, never admission.
pub(super) fn await_resumed_attempt(
    previous: &Value,
    pinned: &Source,
    id: u64,
    mut read: impl FnMut() -> Result<Value, String>,
    elapsed: impl Fn() -> Duration,
    mut pause: impl FnMut(Duration) -> Result<(), String>,
) -> Result<u64, String> {
    run_identity(previous, pinned, id)?;
    let old = previous["run_attempt"]
        .as_u64()
        .filter(|a| *a > 0)
        .ok_or("Missing original Release attempt")?;
    let next = old
        .checked_add(1)
        .ok_or("Invalid resumed Release attempt")?;
    let window = Duration::from_secs(10);
    loop {
        if elapsed() >= window {
            return Err("Acknowledged Release rerun did not expose its exact next attempt within the existing resume window; preserve the original run".into());
        }
        let observed = read()?;
        run_identity(&observed, pinned, id)?;
        if elapsed() >= window {
            return Err("Release rerun observation exceeded the existing resume window".into());
        }
        match observed["run_attempt"].as_u64() {
            Some(a) if a == next => return Ok(next),
            Some(a) if a == old => {
                pause(Duration::from_secs(1).min(window.saturating_sub(elapsed())))?
            }
            _ => return Err("Acknowledged Release rerun exposed a foreign attempt".into()),
        }
    }
}

fn observe_release(
    pinned: &Source,
    id: u64,
    attempt: u64,
    root: &Path,
) -> Result<(Value, Vec<Value>), String> {
    let repository = &pinned.candidate.repository;
    let run = github::api(repository, &format!("actions/runs/{id}"), root)?;
    let jobs = github::jobs(repository, id, root)?;
    if let Err(error) = release_progress(&run, &jobs, pinned, id, attempt) {
        retain_queue("release-failure", &json!({"run":run,"jobs":jobs}))?;
        return Err(error);
    }
    Ok((run, jobs))
}

fn authenticate_queue(
    expected: &pr_contract::Candidate,
    pinned: &Source,
    operator: &lock::Operator,
    budget: &Budget,
    release: (u64, u64),
    root: &Path,
) -> Result<queue::Authority, String> {
    budget.check()?;
    let fresh = source(
        &pinned.candidate.repository,
        pinned.candidate.number,
        &pinned.candidate.head,
        &pinned.candidate.tag,
        true,
        root,
    )?;
    if fresh.integration != expected.number || fresh.cut != pinned.cut {
        return Err("The queued immutable source/pin identity changed".into());
    }
    // Reuse the original complete I authentication, including signed delivery
    // if the owned queue entry has already been consumed by an actual merge.
    delivery::integration(&fresh, None, root)?;
    let pr = github::api(
        &expected.repository,
        &format!("pulls/{}", expected.number),
        root,
    )?;
    if pr_contract::field(&pr, "/head/sha")? != expected.head {
        return Err("The owned integration head changed".into());
    }
    body::qualify(
        pr_contract::field(&pr, "/body")?,
        &body::generated(
            fresh.candidate.number,
            &fresh.cut,
            &fresh.candidate.head,
            &fresh.candidate.tag,
        ),
        &expected.repository,
        expected.number,
    )?;
    let authority = operator.queue_authority(&fresh.cut, root)?;
    // Last authority observation immediately precedes the queue mutation.
    observe_release(&fresh, release.0, release.1, root)?;
    budget.check()?;
    Ok(authority)
}

pub(super) fn retain_queue(stage: &str, value: &Value) -> Result<(), String> {
    let event = json!({"stage":stage,"response":value});
    // Original operator logs remain available even when its exact lease drops.
    eprintln!("Release integration queue: {event}");
    if let Some(path) = env::var_os("VIZE_RELEASE_INTEGRATION_QUEUE_EVIDENCE") {
        let path = Path::new(&path);
        if !path.is_absolute() {
            return Err("Queue evidence requires an absolute operator-owned path".into());
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        writeln!(file, "{event}").map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub(super) use super::rollback::{Rollback, finish_owned_watch};

pub(super) fn watch(
    repository: &str,
    number: u64,
    head: &str,
    tag: &str,
    resume: bool,
    operator: &lock::Operator,
    budget: Option<&Budget>,
    root: &Path,
) -> Result<(), String> {
    pr_budget::check(budget)?;
    let initial = source(repository, number, head, tag, true, root)?;
    if github::tag_target(tag, root)?.is_none() {
        dispatch::retire_legacy(&initial, operator, root)?;
    }
    let id = runs::find_or_dispatch(&initial, operator, budget, root)?;
    let initial_run = github::api(repository, &format!("actions/runs/{id}"), root)?;
    run_identity(&initial_run, &initial, id)?;
    let mut attempt = initial_run["run_attempt"]
        .as_u64()
        .filter(|a| *a > 0)
        .ok_or("Missing original Release attempt")?;
    let early_candidate = if !resume && operator.allows_early_admission() {
        Some(early_integration_candidate(&initial, root)?)
    } else {
        None
    };
    // Only an explicit resume of an already failed immutable R retries it.
    // A new source failure is reported with its original evidence intact.
    if resume
        && !initial.closed
        && initial_run.get("status").and_then(Value::as_str) == Some("completed")
        && initial_run.get("conclusion").and_then(Value::as_str) != Some("success")
    {
        pr_budget::check(budget)?;
        operator.verify()?;
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
        let transition = Instant::now();
        attempt = await_resumed_attempt(
            &initial_run,
            &initial,
            id,
            || {
                pr_budget::check(budget)?;
                operator.verify()?;
                github::api(repository, &format!("actions/runs/{id}"), root)
            },
            || transition.elapsed(),
            |duration| {
                if let Some(budget) = budget {
                    budget.sleep(duration)
                } else {
                    std::thread::sleep(duration);
                    Ok(())
                }
            },
        )?;
    }
    let default = Budget::new(Duration::from_secs(28_800));
    let budget = budget.unwrap_or(&default);
    let mut owned_queue: Option<queue::QueueOwner> = None;
    let result = (|| {
        loop {
            budget.check()?;
            let (run, jobs) = observe_release(&initial, id, attempt, root)?;
            operator.verify()?;
            let source = source(repository, number, head, tag, true, root)?;
            let tagged = github::tag_target(tag, root)?.is_some();
            if tagged {
                delivery::published_identity(&source, id, root)?;
            }
            if source.closed
                && (!tagged || run.get("conclusion").and_then(Value::as_str) != Some("success"))
            {
                return Err("A source receipt was closed before successful publication.".into());
            }
            if run.get("status").and_then(Value::as_str) == Some("completed") {
                if tagged && run.get("conclusion").and_then(Value::as_str) == Some("success") {
                    let release = github::api(repository, &format!("releases/tags/{tag}"), root)?;
                    if release.get("draft").and_then(Value::as_bool) != Some(false) {
                        return Err(
                            "Release workflow succeeded without a public GitHub Release.".into(),
                        );
                    }
                    println!(
                        "Pinned release published: {}. Close the unmerged source receipt after external verification.",
                        pr_contract::field(&release, "/html_url")?
                    );
                    return Ok(());
                }
                let failure = format!(
                    "Pinned R={id} failed; preserve H and its artifacts. Resume after a verified infrastructure repair, or abandon this cut for a source repair."
                );
                return Err(failure);
            }
            if !tagged && owned_queue.is_none() {
                if let Some(candidate) = &early_candidate {
                    let authority = authenticate_queue(
                        candidate,
                        &source,
                        operator,
                        budget,
                        (id, attempt),
                        root,
                    )?;
                    // Shared original authority checks all effective required PR
                    // contexts, exact source heads and configured Actions app IDs.
                    if pr_promote::checks_pass(candidate, root)? {
                        let mut verify = |candidate: &pr_contract::Candidate| {
                            authenticate_queue(
                                candidate,
                                &source,
                                operator,
                                budget,
                                (id, attempt),
                                root,
                            )
                        };
                        let mut execute = github::output;
                        let mut retain = retain_queue;
                        let mut adapter = queue::Adapter {
                            root,
                            execute: &mut execute,
                            retain: &mut retain,
                            batch: None,
                        };
                        queue::enqueue(
                            candidate,
                            &authority,
                            &mut verify,
                            &mut adapter,
                            &mut owned_queue,
                        )?;
                    }
                }
            }
            if !tagged
                && pr_contract::ready_job(&jobs)?
                && pr_promote::checks_pass(&source.candidate, root)?
            {
                if early_candidate.is_none() {
                    let integration = source.integration.to_string();
                    let endpoint = format!("repos/{repository}/pulls/{integration}/merge");
                    observe_release(&source, id, attempt, root)?;
                    budget.check()?;
                    if let Err(error) = github::output(
                        "gh",
                        &[
                            "api",
                            "--method",
                            "PUT",
                            &endpoint,
                            "-f",
                            "merge_method=squash",
                        ],
                        root,
                    ) {
                        println!("Integration admission is still waiting: {error}");
                    }
                }
                if let Some(receipt) = delivery::integration(&source, None, root)? {
                    budget.check()?;
                    delivery::promote(&source, id, &receipt, operator, budget, root)?;
                }
            }
            println!(
                "Pinned source PR #{number}, H={head}, R={id}: waiting for qualification and artifact gates. The tag follows their successful join and version integration."
            );
            budget.sleep(Duration::from_secs(20))?;
        }
    })();
    // Every refused watch path retains the primary failure. Never leave an
    // owned pending entry to a guessed automatic GitHub removal.
    if let (Err(failure), Some(candidate), Some(owner)) = (&result, &early_candidate, &owned_queue)
    {
        let refused = |error| {
            format!(
                "{failure}\nOwned queue withdrawal refused; preserve its complete evidence: {error}"
            )
        };
        let compensation = queue::Compensation::new().map_err(&refused)?;
        let actor = env::var("GITHUB_ACTOR").map_err(|error| refused(error.to_string()))?;
        if env::var("GITHUB_TRIGGERING_ACTOR").as_deref() != Ok(actor.as_str()) {
            return Err(refused("Original operator actor changed".into()));
        }
        let context = Rollback {
            pinned: &initial,
            candidate,
            owner,
            operator,
            actor: &actor,
            root,
        };
        return finish_owned_watch(
            failure,
            &context,
            |program, args, root| compensation.output(program, args, root, &mut retain_queue),
            |repository, endpoints, root| {
                compensation.batch(repository, endpoints, root, &mut retain_queue)
            },
        );
    }
    result
}

#[cfg(test)]
#[path = "pr_pin_watch_tests.rs"]
mod tests;
