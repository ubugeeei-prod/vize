use super::super::{
    pr_budget::{self, Budget},
    pr_contract, pr_github as github, pr_promote,
};
use super::{
    Source, body, candidate, delivery, dispatch, lock, marker, queue, run_identity, runs, source,
};
use serde_json::{Value, json};
use std::{env, fs, io::Write, path::Path, time::Duration};

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

fn authenticate_queue(
    expected: &pr_contract::Candidate,
    pinned: &Source,
    operator: &lock::Operator,
    budget: &Budget,
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
    budget.check()?;
    Ok(authority)
}

fn retain_queue(stage: &str, value: &Value) -> Result<(), String> {
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

pub(super) struct Rollback<'a> {
    pub pinned: &'a Source,
    pub candidate: &'a pr_contract::Candidate,
    pub owner: &'a queue::QueueOwner,
    pub operator: &'a lock::Operator,
    pub actor: &'a str,
    pub root: &'a Path,
}

fn authenticate_compensation(
    context: &Rollback<'_>,
    adapter: &mut queue::Adapter<'_>,
) -> Result<queue::Authority, String> {
    let pinned = context.pinned;
    let expected = context.candidate;
    if expected.repository != pinned.candidate.repository
        || expected.number != pinned.integration
        || expected.tag != pinned.candidate.tag
        || context.owner.head != expected.head
    {
        return Err("Original rollback source/integration identity changed".into());
    }
    let repository = &expected.repository;
    let mut read_pr =
        |number: u64, branch: &str, head: &str, draft: bool| -> Result<Value, String> {
            let pr = adapter.read("rollback-pr", repository, &format!("pulls/{number}"))?;
            let consumed = !draft && pr["state"] == "closed" && pr["merged"] == true;
            if pr["number"].as_u64() != Some(number)
                || pr["draft"] != draft
                || (!consumed && (pr["state"] != "open" || pr["merged"] != false))
                || pr["head"]["sha"] != head
                || pr["head"]["ref"] != branch
                || pr["base"]["ref"] != "main"
                || pr["head"]["repo"]["full_name"] != *repository
                || pr["base"]["repo"]["full_name"] != *repository
                || pr["user"]["type"] != "User"
            {
                return Err("Original rollback PR changed".into());
            }
            let user = pr_contract::field(&pr, "/user/login")?;
            if user.is_empty()
                || !user
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            {
                return Err("Invalid original PR author".into());
            }
            pr_contract::maintainer(&adapter.read(
                "rollback-pr-permission",
                repository,
                &format!("collaborators/{user}/permission"),
            )?)?;
            if !consumed {
                let reference = adapter.read(
                    "rollback-pr-ref",
                    repository,
                    &format!("git/ref/heads/{branch}"),
                )?;
                if reference["ref"] != format!("refs/heads/{branch}")
                    || reference["object"]["type"] != "commit"
                    || reference["object"]["sha"] != head
                {
                    return Err("Original rollback PR ref changed".into());
                }
            }
            Ok(pr)
        };
    let receipt = read_pr(
        pinned.candidate.number,
        &pinned.candidate.branch,
        &pinned.candidate.head,
        true,
    )?;
    let integration = read_pr(expected.number, &expected.branch, &expected.head, false)?;
    let raw = pr_contract::field(&receipt, "/body")?;
    for (key, value) in [
        ("vize-release-pin", "immutable-v1".into()),
        ("vize-release-bump", "minor".into()),
        ("vize-release-pin-head", pinned.candidate.head.clone()),
        ("vize-release-pin-cut", pinned.cut.clone()),
        ("vize-release-base-version", pinned.base_version.clone()),
        ("vize-release-integration", pinned.integration.to_string()),
    ] {
        if marker(raw, key)? != value {
            return Err("Original rollback source markers changed".into());
        }
    }
    let canonical = body::generated(
        pinned.candidate.number,
        &pinned.cut,
        &pinned.candidate.head,
        &pinned.candidate.tag,
    );
    body::qualify(
        pr_contract::field(&integration, "/body")?,
        &canonical,
        repository,
        expected.number,
    )?;
    if integration["merged"] == false
        && candidate::integration_fields(&integration, repository, expected.number, &expected.head)?
            != (
                pinned.candidate.number,
                pinned.candidate.head.clone(),
                pinned.cut.clone(),
                pinned.candidate.tag.clone(),
            )
    {
        return Err("Original rollback integration markers changed".into());
    }
    context
        .operator
        .compensation_authority(pinned, context.actor, adapter)
}

/// The actual watch error route, also exercised after a real lease acquisition.
/// Only bounded identity reads and withdrawal of its already-owned entry occur.
pub(super) fn finish_owned_watch(
    failure: &str,
    context: &Rollback<'_>,
    execute: impl Fn(&str, &[&str], &Path) -> Result<String, String>,
) -> Result<(), String> {
    let mut verify = |candidate: &pr_contract::Candidate| {
        if candidate != context.candidate {
            return Err("Rollback candidate changed".into());
        }
        let mut bounded = |program: &str, args: &[&str], root: &Path| execute(program, args, root);
        let mut retain = retain_queue;
        let mut adapter = queue::Adapter {
            root: context.root,
            execute: &mut bounded,
            retain: &mut retain,
        };
        authenticate_compensation(context, &mut adapter)
    };
    let mut bounded = |program: &str, args: &[&str], root: &Path| execute(program, args, root);
    let mut retain = retain_queue;
    let mut adapter = queue::Adapter {
        root: context.root,
        execute: &mut bounded,
        retain: &mut retain,
    };
    match queue::cleanup(context.candidate, context.owner, &mut verify, &mut adapter) {
        Ok(()) => Err(failure.to_owned()),
        Err(error) => Err(format!(
            "{failure}\nOwned queue withdrawal refused; preserve its complete evidence: {error}"
        )),
    }
}

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
        if let Some(budget) = budget {
            budget.sleep(Duration::from_secs(10))?;
        } else {
            std::thread::sleep(Duration::from_secs(10));
        }
    }
    let default = Budget::new(Duration::from_secs(28_800));
    let budget = budget.unwrap_or(&default);
    let mut owned_queue: Option<queue::QueueOwner> = None;
    let result = (|| {
        loop {
            budget.check()?;
            operator.verify()?;
            let source = source(repository, number, head, tag, true, root)?;
            let run = github::api(repository, &format!("actions/runs/{id}"), root)?;
            run_identity(&run, &source, id)?;
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
                    let authority = authenticate_queue(candidate, &source, operator, budget, root)?;
                    // Shared original authority checks all effective required PR
                    // contexts, exact source heads and configured Actions app IDs.
                    if pr_promote::checks_pass(candidate, root)? {
                        let mut verify = |candidate: &pr_contract::Candidate| {
                            authenticate_queue(candidate, &source, operator, budget, root)
                        };
                        let mut execute = github::output;
                        let mut retain = retain_queue;
                        let mut adapter = queue::Adapter {
                            root,
                            execute: &mut execute,
                            retain: &mut retain,
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
                && pr_contract::ready_job(&github::jobs(repository, id, root)?)?
                && pr_promote::checks_pass(&source.candidate, root)?
            {
                if early_candidate.is_none() {
                    let integration = source.integration.to_string();
                    let endpoint = format!("repos/{repository}/pulls/{integration}/merge");
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
        return finish_owned_watch(failure, &context, |program, args, root| {
            compensation.output(program, args, root, &mut retain_queue)
        });
    }
    result
}
