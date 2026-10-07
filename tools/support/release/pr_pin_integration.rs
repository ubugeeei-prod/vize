use super::super::{pr_github as github, pr_start};
use super::{Source, first_parent, marker, metadata};
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

pub(super) fn integration_pr(source: &Source, bump: &str, root: &Path) -> Result<u64, String> {
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
            "number,body",
        ],
        root,
    )?;
    let prs: Value = serde_json::from_str(&found).map_err(|e| e.to_string())?;
    let prs = prs.as_array().ok_or("Missing integration PR list")?;
    if let [pr] = prs.as_slice() {
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
    first_parent(&source.cut, &main, root)?;
    let path = std::env::temp_dir().join(format!(
        "vize-release-integration-{}-{}",
        std::process::id(),
        &main[..12]
    ));
    github::git(
        &[
            "worktree",
            "add",
            "--detach",
            path.to_str().ok_or("Invalid integration path")?,
            &main,
        ],
        root,
    )?;
    #[cfg(unix)]
    if root.join("node_modules").is_dir() {
        std::os::unix::fs::symlink(root.join("node_modules"), path.join("node_modules"))
            .map_err(|e| e.to_string())?;
    }
    let result = (|| {
        if github::version(&path)? != source.base_version {
            return Err("Another version owns main; no new integration is allowed.".into());
        }
        pr_start::prepare(bump, &path)?;
        let head = github::git(&["rev-parse", "HEAD"], &path)?;
        metadata::verify_delta(
            &main,
            &head,
            &source.base_version,
            candidate.tag.trim_start_matches('v'),
            &path,
        )?;
        github::git(
            &["push", "origin", &format!("HEAD:refs/heads/{branch}")],
            &path,
        )?;
        let body = format!(
            "Integrate only generated {} version metadata into moving main. Source PR #{} remains an immutable draft receipt at H={}. The maintainer owns protected queue admission; this command never auto-merges this PR or writes main. Admit this integration only after immutable H/R qualification has succeeded: all required H checks, all five full H gates, every release build, and full preflight must be terminal green. Actual delivery is checked against its final first parent, not the initial base.\n\n<!-- vize-release-pin-source: {} -->\n<!-- vize-release-pin-cut: {} -->\n<!-- vize-release-pin-head: {} -->\n<!-- vize-release-pin-tag: {} -->\n",
            candidate.tag,
            candidate.number,
            candidate.head,
            candidate.number,
            source.cut,
            candidate.head,
            candidate.tag
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
