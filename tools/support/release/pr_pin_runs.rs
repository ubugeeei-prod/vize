//! Find or dispatch only the exact immutable source run.
use super::super::{
    pr_budget::{self, Budget},
    pr_ci, pr_github as github,
};
use super::{Source, delivery, lock, run_identity};
use serde_json::Value;
use std::path::Path;

pub(super) fn find_or_dispatch(
    source: &Source,
    operator: &lock::Operator,
    budget: Option<&Budget>,
    root: &Path,
) -> Result<u64, String> {
    pr_budget::check(budget)?;
    let candidate = &source.candidate;
    if github::tag_target(&candidate.tag, root)?.is_some() {
        let (id, _) = delivery::tag_receipt(source, root)?;
        delivery::published_identity(source, id, root)?;
        return Ok(id);
    }
    if source.closed {
        return Err("A closed source receipt cannot dispatch publication.".into());
    }
    let find = || -> Result<Option<u64>, String> {
        let response = github::api(
            &candidate.repository,
            &format!(
                "actions/workflows/release.yml/runs?event=workflow_dispatch&head_sha={}&per_page=100",
                candidate.head
            ),
            root,
        )?;
        let runs = response
            .get("workflow_runs")
            .and_then(Value::as_array)
            .ok_or("Missing pinned release runs")?;
        Ok(runs
            .iter()
            .filter_map(|run| {
                run.get("id")
                    .and_then(Value::as_u64)
                    .filter(|id| run_identity(run, source, *id).is_ok())
            })
            .max())
    };
    if let Some(id) = find()? {
        return Ok(id);
    }
    pr_ci::corsa_first_publication(
        &candidate.repository,
        candidate.number,
        &candidate.branch,
        &candidate.head,
        &source.cut,
        &candidate.tag,
        budget,
        root,
    )?;
    // Start Q separately; R builds overlap its read-only qualification join.
    // Existing immutable R resumes above retain their original evidence.
    operator.verify()?;
    let qualification_deadline = pr_ci::start_check(
        &candidate.repository,
        &candidate.branch,
        &candidate.head,
        &source.cut,
        budget,
        root,
    )?;
    pr_budget::check(budget)?;
    operator.verify()?;
    let current = super::source(
        &candidate.repository,
        candidate.number,
        &candidate.head,
        &candidate.tag,
        false,
        root,
    )?;
    if current.cut != source.cut
        || current.base_version != source.base_version
        || current.integration != source.integration
    {
        return Err("The immutable source receipt changed during Check qualification.".into());
    }
    if github::tag_target(&candidate.tag, root)?.is_some() {
        return Err("The release tag appeared during Check qualification; inspect the retained source and Release before resuming.".into());
    }
    // A concurrent observer may have found the same R while Q was starting.
    // Never create another publication run after the authenticated handoff.
    if let Some(id) = find()? {
        return Ok(id);
    }
    pr_budget::check(budget)?;
    operator.verify()?;
    let mut arguments = [
        "workflow",
        "run",
        "release.yml",
        "--repo",
        &candidate.repository,
        "--ref",
        &candidate.branch,
        "--field",
        &format!("tag_name={}", candidate.tag),
        "--field",
        &format!("release_pr={}", candidate.number),
        "--field",
        &format!("expected_sha={}", candidate.head),
        "--field",
        "pinned=true",
    ]
    .map(str::to_owned)
    .to_vec();
    if let Some(deadline) = qualification_deadline {
        arguments.extend([
            "--field".into(),
            format!("qualification_deadline={deadline}"),
        ]);
    }
    github::output(
        "gh",
        &arguments.iter().map(String::as_str).collect::<Vec<_>>(),
        root,
    )?;
    for _ in 0..30 {
        if let Some(budget) = budget {
            budget.sleep(std::time::Duration::from_secs(2))?;
        } else {
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
        if let Some(id) = find()? {
            return Ok(id);
        }
    }
    Err(
        "No identifiable pinned R appeared; preserve H and inspect dispatch before retrying."
            .into(),
    )
}
