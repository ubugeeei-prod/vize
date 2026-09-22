use super::pr_contract::{self, Candidate};
use serde_json::Value;
use std::{
    env, fs,
    path::Path,
    process::{Command, Stdio},
};

pub fn output(program: &str, args: &[&str], cwd: &Path) -> Result<String, String> {
    let result = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("Could not start {program}: {e}"))?;
    if !result.status.success() {
        return Err(format!(
            "{program} {} failed:\n{}{}",
            args.join(" "),
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&result.stdout).trim().to_string())
}

pub fn run(program: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .status()
        .map_err(|e| format!("Could not start {program}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} {} failed ({status})", args.join(" ")))
    }
}

pub fn git(args: &[&str], root: &Path) -> Result<String, String> {
    output("git", args, root)
}

pub fn api(repository: &str, resource: &str, root: &Path) -> Result<Value, String> {
    let path = format!("repos/{repository}/{resource}");
    let text = output("gh", &["api", &path], root)?;
    serde_json::from_str(&text).map_err(|e| format!("Invalid GitHub response: {e}"))
}

pub fn repository(root: &Path) -> Result<String, String> {
    let name = if let Ok(name) = env::var("GITHUB_REPOSITORY") {
        name
    } else {
        output(
            "gh",
            &[
                "repo",
                "view",
                "--json",
                "nameWithOwner",
                "--jq",
                ".nameWithOwner",
            ],
            root,
        )?
    };
    let parts: Vec<_> = name.split('/').collect();
    if parts.len() != 2
        || parts.iter().any(|p| {
            p.is_empty()
                || !p
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        })
    {
        return Err("Invalid GitHub repository name".into());
    }
    Ok(name)
}

pub fn author(repository: &str, login: &str, root: &Path) -> Result<Value, String> {
    if login.is_empty()
        || !login
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("Invalid release author login".into());
    }
    api(
        repository,
        &format!("collaborators/{login}/permission"),
        root,
    )
}

pub fn candidate(
    repository: &str,
    number: u64,
    head: &str,
    tag: &str,
    merged: bool,
    root: &Path,
) -> Result<Candidate, String> {
    let pr = api(repository, &format!("pulls/{number}"), root)?;
    let login = pr_contract::field(&pr, "/user/login")?;
    let permission = author(repository, login, root)?;
    pr_contract::candidate(&pr, &permission, repository, head, tag, merged)
}

pub fn fetch_main(root: &Path) -> Result<String, String> {
    git(
        &[
            "fetch",
            "--no-tags",
            "origin",
            "+refs/heads/main:refs/remotes/origin/main",
        ],
        root,
    )?;
    git(&["rev-parse", "refs/remotes/origin/main"], root)
}

pub fn parent(head: &str, root: &Path) -> Result<String, String> {
    let line = git(&["rev-list", "--parents", "-n", "1", head], root)?;
    let parts: Vec<_> = line.split_whitespace().collect();
    if parts.len() != 2 || parts[0] != head {
        return Err("A release candidate must be one commit on current main.".into());
    }
    Ok(parts[1].into())
}

pub fn version(root: &Path) -> Result<String, String> {
    let cargo = fs::read_to_string(root.join("Cargo.toml")).map_err(|e| e.to_string())?;
    version_text(&cargo)
}

pub fn version_text(cargo: &str) -> Result<String, String> {
    let mut workspace = false;
    for line in cargo.lines().map(str::trim) {
        if line.starts_with('[') {
            workspace = line == "[workspace.package]";
        }
        if workspace
            && let Some(value) = line
                .strip_prefix("version = \"")
                .and_then(|s| s.strip_suffix('"'))
        {
            pr_contract::tag(&format!("v{value}"))?;
            return Ok(value.into());
        }
    }
    Err("Missing workspace version".into())
}

pub fn tag_target(tag: &str, root: &Path) -> Result<Option<String>, String> {
    let plain = format!("refs/tags/{tag}");
    let peeled = format!("{plain}^{{}}");
    let refs = git(&["ls-remote", "--tags", "origin", &plain, &peeled], root)?;
    let rows: Vec<_> = refs.lines().filter_map(|l| l.split_once('\t')).collect();
    Ok(rows
        .iter()
        .find(|(_, r)| *r == peeled)
        .or_else(|| rows.iter().find(|(_, r)| *r == plain))
        .map(|(sha, _)| sha.to_string()))
}

pub fn jobs(repository: &str, run_id: u64, root: &Path) -> Result<Vec<Value>, String> {
    let mut result = Vec::new();
    for page in 1..=100 {
        let response = api(
            repository,
            &format!("actions/runs/{run_id}/jobs?filter=latest&per_page=100&page={page}"),
            root,
        )?;
        let jobs = response
            .get("jobs")
            .and_then(Value::as_array)
            .ok_or("Missing workflow jobs")?;
        result.extend(jobs.iter().cloned());
        if jobs.len() < 100 {
            return Ok(result);
        }
    }
    Err("Workflow job inventory exceeded its pagination limit".into())
}

pub fn clean(root: &Path) -> Result<(), String> {
    if git(&["status", "--porcelain"], root)?.is_empty() {
        Ok(())
    } else {
        Err("Release worktree has uncommitted changes; preserving them.".into())
    }
}
