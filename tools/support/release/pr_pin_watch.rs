use super::super::{
    pr_budget::{self, Budget},
    pr_contract, pr_github as github, pr_promote,
};
use super::{delivery, dispatch, lock, run_identity, runs, source};
use serde_json::Value;
use std::{path::Path, time::Duration};

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
    let id = runs::find_or_dispatch(&initial, budget, root)?;
    let initial_run = github::api(repository, &format!("actions/runs/{id}"), root)?;
    run_identity(&initial_run, &initial, id)?;
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
            return Err(format!(
                "Pinned R={id} failed; preserve H and its artifacts. Resume after a verified infrastructure repair, or abandon this cut for a source repair."
            ));
        }
        if !tagged
            && pr_contract::ready_job(&github::jobs(repository, id, root)?)?
            && pr_promote::checks_pass(&source.candidate, root)?
        {
            if let Some(receipt) = delivery::integration(&source, None, root)? {
                budget.check()?;
                delivery::promote(&source, id, &receipt, operator, budget, root)?;
            }
        }
        println!(
            "Pinned source PR #{number}, H={head}, R={id}: qualification/publication continues. Root may admit integration only after all required H checks, five full H gates, every build, and full preflight are terminal green."
        );
        budget.sleep(Duration::from_secs(20))?;
    }
}
