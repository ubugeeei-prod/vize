use super::super::{
    pr_budget::{self, Budget},
    pr_github as github, pr_start,
};
use super::{Source, marker, metadata};
use serde_json::Value;
use std::{fs, path::Path};

pub(super) fn open_pr(
    repository: &str,
    branch: &str,
    title: &str,
    body: &str,
    draft: bool,
    root: &Path,
) -> Result<u64, String> {
    let path = root.join(".git-release-pinned-pr.md");
    fs::write(&path, body).map_err(|e| e.to_string())?;
    let mut args = vec![
        "pr",
        "create",
        "--repo",
        repository,
        "--base",
        "main",
        "--head",
        branch,
        "--title",
        title,
        "--body-file",
        path.to_str().ok_or("Invalid PR body path")?,
    ];
    if draft {
        args.push("--draft");
    }
    let created = github::output("gh", &args, root);
    let _ = fs::remove_file(path);
    let url = created.map_err(|e|format!("{e}\nPreserved branch {branch}; inspect its existing PR before retrying. No branch or remote tag was deleted."))?;
    println!("{url}");
    url.rsplit('/')
        .next()
        .and_then(|n| n.parse().ok())
        .ok_or("Invalid created PR URL".into())
}

pub(super) fn integration_pr(
    source: &Source,
    bump: &str,
    budget: Option<&Budget>,
    root: &Path,
) -> Result<u64, String> {
    pr_budget::check(budget)?;
    let candidate = &source.candidate;
    let branch = format!("release-integration/{}", candidate.tag);
    let found = github::output(
        "gh",
        &[
            "pr",
            "list",
            "--repo",
            &candidate.repository,
            "--head",
            &branch,
            "--state",
            "all",
            "--json",
            "number,body,state",
        ],
        root,
    )?;
    let prs: Value = serde_json::from_str(&found).map_err(|e| e.to_string())?;
    let prs = prs.as_array().ok_or("Missing integration PR list")?;
    if let [pr] = prs.as_slice() {
        // A closed receipt whose branch is gone is an abandoned cut of this
        // version. The next cut may reuse the integration name.
        let abandoned = pr["state"].as_str() == Some("CLOSED")
            && github::git(
                &[
                    "ls-remote",
                    "--heads",
                    "origin",
                    &format!("refs/heads/{branch}"),
                ],
                root,
            )?
            .is_empty();
        if !abandoned {
            let body = pr["body"]
                .as_str()
                .ok_or("Missing existing integration body")?;
            if marker(body, "vize-release-pin-head")? != candidate.head
                || marker(body, "vize-release-pin-source")? != candidate.number.to_string()
            {
                return Err(
                    "Existing integration branch belongs to a different cut; preserve it for review."
                        .into(),
                );
            }
            return pr["number"]
                .as_u64()
                .ok_or("Missing integration number".into());
        }
    }
    if !prs.is_empty() {
        return Err("Ambiguous integration PRs; refusing to replace any branch.".into());
    }
    if !github::git(
        &[
            "ls-remote",
            "--heads",
            "origin",
            &format!("refs/heads/{branch}"),
        ],
        root,
    )?
    .is_empty()
    {
        return Err(
            "An orphan integration branch exists; inspect it instead of replacing it.".into(),
        );
    }
    let main = github::fetch_main(root)?;
    super::stable::main_anchor(source, &main, root)?;
    pr_budget::check(budget)?;
    let path = pr_start::pinned_integration_worktree(&main, root)?;
    let result = (|| {
        if github::version(&path)? != source.base_version {
            return Err("Another version owns main; no new integration is allowed.".into());
        }
        pr_budget::check(budget)?;
        if source.stable.is_some() {
            pr_start::prepare("patch", &path)?;
            if format!("v{}", github::version(&path)?) != candidate.tag {
                return Err("Main metadata integration produced a different stable PATCH".into());
            }
        } else {
            super::retire::prepare(bump, &candidate.tag, budget, &path)?;
        }
        pr_budget::check(budget)?;
        let head = github::git(&["rev-parse", "HEAD"], &path)?;
        metadata::verify_delta(
            &main,
            &head,
            &source.base_version,
            candidate.tag.trim_start_matches('v'),
            &path,
        )?;
        pr_budget::check(budget)?;
        github::git(
            &["push", "origin", &format!("HEAD:refs/heads/{branch}")],
            &path,
        )?;
        let body = super::body::generated(
            candidate.number,
            &source.cut,
            &candidate.head,
            &candidate.tag,
        );
        open_pr(
            &candidate.repository,
            &branch,
            &format!(
                "chore(release): integrate {} version metadata",
                candidate.tag
            ),
            &body,
            false,
            &path,
        )
    })();
    match result {
        Ok(number) => {
            github::git(
                &[
                    "worktree",
                    "remove",
                    path.to_str().ok_or("Invalid integration path")?,
                ],
                root,
            )?;
            Ok(number)
        }
        Err(error) => Err(format!(
            "{error}\nPreserved generated integration worktree: {}",
            path.display()
        )),
    }
}
