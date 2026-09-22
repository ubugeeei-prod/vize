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

fn worktree(revision: &str, root: &Path) -> Result<PathBuf, String> {
    let path = env::temp_dir().join(format!(
        "vize-release-{}-{}",
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
    if root.join("node_modules").is_dir() {
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
    github::run(
        "moon",
        &[
            "run",
            "--target",
            "native",
            "tools/moon/cmd/release",
            "--",
            bump,
            "-y",
            "--prepare-only",
        ],
        work,
    )
}

fn prepare_and_open(bump: &str, repository: &str, work: &Path) -> Result<(), String> {
    let base_version = github::version(work)?;
    prepare(bump, work)?;
    let tag = format!("v{}", github::version(work)?);
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
            "{branch} already exists; use vp run release --resume <PR>."
        ));
    }
    github::git(
        &["push", "origin", &format!("HEAD:refs/heads/{branch}")],
        work,
    )?;
    let body = work.join(".git-release-pr-body.md");
    fs::write(&body, format!("## Release {tag}\n\nPrepare aligned workspace and package versions from current main.\n\nThe release workflow builds and verifies all artifacts before a tag exists. The release command refreshes this PR when main advances, then atomically fast-forwards main and creates the tag. Publication consumes the already-validated artifacts.\n\n- Required author role: maintain or admin\n- Failed validation leaves this PR open and creates no tag\n- Resume: `vp run release --resume <PR number>`\n\n<!-- vize-release-bump: {bump} -->\n<!-- vize-release-base-version: {base_version} -->\n")).map_err(|e| e.to_string())?;
    let created = github::output(
        "gh",
        &[
            "pr",
            "create",
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
    let url = created?;
    println!("{url}");
    let number = url
        .rsplit('/')
        .next()
        .and_then(|v| v.parse::<u64>().ok())
        .ok_or("Could not read the created PR number")?;
    pr_watch::release(repository, number, &tag, bump, &base_version, false, work)
}

pub fn resume(number: u64, root: &Path) -> Result<(), String> {
    let repository = github::repository(root)?;
    let login = github::output("gh", &["api", "user", "--jq", ".login"], root)?;
    pr_contract::maintainer(&github::author(&repository, &login, root)?)?;
    let pr = github::api(&repository, &format!("pulls/{number}"), root)?;
    let head = pr_contract::field(&pr, "/head/sha")?;
    let branch = pr_contract::field(&pr, "/head/ref")?;
    let tag = branch.strip_prefix("release/").ok_or("Not a release PR")?;
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
    finish(
        pr_watch::release(&repository, number, tag, &bump, &base_version, true, &work),
        &work,
        root,
    )
}

fn marker(body: &str, key: &str) -> Result<String, String> {
    let prefix = format!("<!-- {key}: ");
    body.lines()
        .find_map(|line| {
            line.strip_prefix(&prefix)
                .and_then(|v| v.strip_suffix(" -->"))
        })
        .map(str::to_string)
        .ok_or_else(|| format!("Release PR is missing {key}"))
}

fn finish(result: Result<(), String>, work: &Path, root: &Path) -> Result<(), String> {
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
