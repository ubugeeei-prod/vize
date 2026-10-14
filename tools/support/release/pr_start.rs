use super::{pr_contract, pr_github as github, pr_watch};
use serde_json::Value;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

pub fn start(bump: &str, root: &Path) -> Result<(), String> {
    if !["patch", "minor", "major", "alpha", "beta", "rc", "release"].contains(&bump) {
        return Err("Unknown release bump".into());
    }
    let repository = github::repository(root)?;
    let login = github::output("gh", &["api", "user", "--jq", ".login"], root)?;
    pr_contract::maintainer(&github::author(&repository, &login, root)?)?;
    let base = github::fetch_main(root)?;
    let work = worktree(&base, root)?;
    let result = prepare_and_open(bump, &repository, &work);
    finish(result, &work, root)
}

pub(super) fn worktree(revision: &str, root: &Path) -> Result<PathBuf, String> {
    worktree_with_graph(revision, root, true, "vize-release")
}

pub(super) fn pinned_worktree(revision: &str, root: &Path) -> Result<PathBuf, String> {
    // Installed providers have no durable source receipt. Qualify the exact
    // requested frozen graph instead of borrowing root's node_modules.
    worktree_with_graph(revision, root, false, "vize-release")
}

pub(super) fn pinned_integration_worktree(revision: &str, root: &Path) -> Result<PathBuf, String> {
    worktree_with_graph(revision, root, false, "vize-release-integration")
}

fn worktree_with_graph(
    revision: &str,
    root: &Path,
    reuse: bool,
    prefix: &str,
) -> Result<PathBuf, String> {
    let path = env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        process::id(),
        revision.get(..12).unwrap_or("resume")
    ));
    github::git(
        &[
            "worktree",
            "add",
            "--detach",
            path.to_str().ok_or("Invalid worktree path")?,
            revision,
        ],
        root,
    )?;
    if reuse && root.join("node_modules").is_dir() {
        // The preparation checks read the workspace's installed JS tools.
        // Build and publication still run in clean Actions checkouts.
        #[cfg(unix)]
        std::os::unix::fs::symlink(root.join("node_modules"), path.join("node_modules"))
            .map_err(|e| e.to_string())?;
        #[cfg(windows)]
        github::run("vp", &["install", "--frozen-lockfile"], &path)?;
    } else {
        github::run("vp", &["install", "--frozen-lockfile"], &path)?;
    }
    println!("Release worktree: {}", path.display());
    Ok(path)
}

pub fn prepare(bump: &str, work: &Path) -> Result<(), String> {
    prepare_with_target(bump, None, None, work)
}

pub(super) fn prepare_pinned_minor(
    target: &str,
    budget: Option<&super::pr_budget::Budget>,
    work: &Path,
) -> Result<(), String> {
    prepare_with_target("minor", Some(target), budget, work)
}

fn prepare_with_target(
    bump: &str,
    target: Option<&str>,
    budget: Option<&super::pr_budget::Budget>,
    work: &Path,
) -> Result<(), String> {
    let mut command = release_preparation(work)?;
    command.args([bump, "-y", "--prepare-only"]);
    if let Some(target) = target {
        command.args(["--pinned-target", target]);
        // The frozen C may fall behind main while it is prepared; the local
        // guard accepts exactly this cut while it stays reachable from main.
        let cut = github::git(&["rev-parse", "HEAD"], work)?;
        command.env("VIZE_RELEASE_PINNED_CUT", cut);
    }
    super::pr_budget::child_command(&mut command, budget)?;
    let status = command
        .current_dir(work)
        .stdin(std::process::Stdio::null())
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Release preparation failed ({status})"))
    }
}

/// The release preparation entry point of the TypeScript release command.
pub const RELEASE_COMMAND: &str = "tools/support/release/release/main.ts";

/// Runs the prepared source's own TypeScript release command. Sources cut
/// before the TypeScript port (the MoonBit release package) can no longer be
/// prepared: MoonBit was retired from the repository tooling.
pub fn release_preparation(work: &Path) -> Result<process::Command, String> {
    if !work.join(RELEASE_COMMAND).is_file() {
        return Err(format!(
            "{RELEASE_COMMAND} is missing; this source predates the TypeScript release port and cannot be re-released"
        ));
    }
    let mut command = process::Command::new("node");
    command.arg(RELEASE_COMMAND);
    Ok(command)
}

fn prepare_and_open(bump: &str, repository: &str, work: &Path) -> Result<(), String> {
    let base_version = github::version(work)?;
    prepare(bump, work)?;
    let tag = format!("v{}", github::version(work)?);
    let operator =
        super::pr_pin::lock::acquire(&tag, &github::git(&["rev-parse", "HEAD"], work)?, work)?;
    let branch = format!("release/{tag}");
    if !github::git(
        &[
            "ls-remote",
            "--heads",
            "origin",
            &format!("refs/heads/{branch}"),
        ],
        work,
    )?
    .is_empty()
    {
        return Err(format!(
            "{branch} already exists. Find its PR with gh pr list --repo {repository} --head {branch} --state all, then use vp run release --resume <PR>. If creation previously failed and no PR exists, inspect and remove that orphan branch before retrying."
        ));
    }
    github::git(
        &["push", "origin", &format!("HEAD:refs/heads/{branch}")],
        work,
    )?;
    let body = work.join(".git-release-pr-body.md");
    fs::write(&body, format!("## Release {tag}\n\nPrepare aligned workspace and package versions from current main.\n\nThe release workflow builds and verifies all artifacts before a tag exists. The release command refreshes this PR when main advances, then atomically fast-forwards main and creates the tag. Publication consumes the already-validated artifacts.\n\nThis PR remains draft while validation runs. Do not convert it to ready or merge it in the GitHub UI. The release command promotes main and the tag together; GitHub then records an indirect merge.\n\n- Required author role: maintain or admin\n- Failed validation leaves this PR open and creates no tag\n- Resume: `vp run release --resume <PR number>`\n\n<!-- vize-release-bump: {bump} -->\n<!-- vize-release-base-version: {base_version} -->\n")).map_err(|e| e.to_string())?;
    let created = github::output(
        "gh",
        &[
            "pr",
            "create",
            "--draft",
            "--repo",
            repository,
            "--base",
            "main",
            "--head",
            &branch,
            "--title",
            &format!("chore(release): {tag}"),
            "--body-file",
            body.to_str().ok_or("Invalid body path")?,
        ],
        work,
    );
    let _ = fs::remove_file(&body);
    let url = created.map_err(|error| format!(
        "{error}\nThe pushed branch {branch} is preserved: a failed response does not prove GitHub did not create the PR. Check gh pr list --repo {repository} --head {branch} --state all and resume that PR. If no PR exists, inspect and delete the orphan with git push origin --delete {branch}, then retry."
    ))?;
    println!("{url}");
    let number = url
        .rsplit('/')
        .next()
        .and_then(|v| v.parse::<u64>().ok())
        .ok_or("Could not read the created PR number")?;
    pr_watch::release(
        repository,
        number,
        &tag,
        bump,
        &base_version,
        false,
        &operator,
        work,
    )
}

pub fn resume(number: u64, root: &Path) -> Result<(), String> {
    let repository = github::repository(root)?;
    let login = github::output("gh", &["api", "user", "--jq", ".login"], root)?;
    pr_contract::maintainer(&github::author(&repository, &login, root)?)?;
    let pr = github::protect_release_pr(&repository, number, root)?;
    let head = pr_contract::field(&pr, "/head/sha")?;
    let branch = pr_contract::field(&pr, "/head/ref")?;
    let tag = branch.strip_prefix("release/").ok_or("Not a release PR")?;
    super::pr_pin::lock::reject_pinned(tag, root)?;
    let author = pr_contract::field(&pr, "/user/login")?;
    pr_contract::candidate(
        &pr,
        &github::author(&repository, author, root)?,
        &repository,
        head,
        tag,
        true,
    )?;
    let body = pr.get("body").and_then(Value::as_str).unwrap_or("");
    let bump = marker(body, "vize-release-bump")?;
    let base_version = marker(body, "vize-release-base-version")?;
    github::git(
        &[
            "fetch",
            "--no-tags",
            "origin",
            &format!("refs/pull/{number}/head"),
        ],
        root,
    )?;
    let work = worktree(head, root)?;
    let operator = super::pr_pin::lock::acquire(tag, head, root)?;
    finish(
        pr_watch::release(
            &repository,
            number,
            tag,
            &bump,
            &base_version,
            true,
            &operator,
            &work,
        ),
        &work,
        root,
    )
}

pub(super) fn marker(body: &str, key: &str) -> Result<String, String> {
    let prefix = format!("<!-- {key}: ");
    body.lines()
        .find_map(|line| {
            line.strip_prefix(&prefix)
                .and_then(|v| v.strip_suffix(" -->"))
        })
        .map(str::to_string)
        .ok_or_else(|| format!("Release PR is missing {key}"))
}

pub(super) fn finish(result: Result<(), String>, work: &Path, root: &Path) -> Result<(), String> {
    match result {
        Ok(()) => {
            github::git(
                &[
                    "worktree",
                    "remove",
                    work.to_str().ok_or("Invalid worktree path")?,
                ],
                root,
            )?;
            Ok(())
        }
        Err(error) => Err(format!(
            "{error}\nPreserved release worktree: {}",
            work.display()
        )),
    }
}

/// The opt-in protocol keeps the source cut independent of the moving main.
pub fn start_pinned(bump: &str, root: &Path) -> Result<(), String> {
    super::pr_pin::start(bump, root)
}

pub fn resume_pinned(number: u64, root: &Path) -> Result<(), String> {
    super::pr_pin::resume(number, root)
}

#[cfg(test)]
mod preparation_tests {
    use std::fs;

    #[test]
    fn release_preparation_runs_the_prepared_sources_own_release_command() {
        let work = std::env::temp_dir().join(format!("vize-preparation-{}", std::process::id()));
        let _ = fs::remove_dir_all(&work);
        fs::create_dir_all(&work).unwrap();
        assert!(super::release_preparation(&work).is_err());
        let command = work.join(super::RELEASE_COMMAND);
        fs::create_dir_all(command.parent().unwrap()).unwrap();
        fs::write(&command, "").unwrap();
        let current = super::release_preparation(&work).unwrap();
        assert_eq!(current.get_program(), "node");
        assert_eq!(
            current.get_args().collect::<Vec<_>>(),
            [super::RELEASE_COMMAND]
        );
        fs::remove_dir_all(&work).unwrap();
    }
}
