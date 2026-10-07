use super::super::{
    pr_contract::{self, Candidate},
    pr_github as github, pr_start,
};
use super::integration::{integration_pr, open_pr};
use super::{Source, delivery, marker, metadata, run_identity, source, watch};
use serde_json::Value;
use std::{fs, path::Path};

fn authorize(root: &Path) -> Result<String, String> {
    let repository = github::repository(root)?;
    let login = github::output("gh", &["api", "user", "--jq", ".login"], root)?;
    pr_contract::maintainer(&github::author(&repository, &login, root)?)?;
    Ok(repository)
}

fn supports_pin(head: &str, root: &Path) -> Result<(), String> {
    github::git(
        &[
            "cat-file",
            "-e",
            &format!("{head}:tools/support/release/pr_pin.rs"),
        ],
        root,
    )?;
    let workflow = String::from_utf8(metadata::bytes(
        &["show", &format!("{head}:.github/workflows/release.yml")],
        root,
    )?)
    .map_err(|e| e.to_string())?;
    if !workflow.contains("pinned:") || !workflow.contains("validate-pinned") {
        return Err("The frozen H lacks the reviewed pinned workflow; publish the source fix first and prepare a new H.".into());
    }
    Ok(())
}

pub fn start(bump: &str, root: &Path) -> Result<(), String> {
    if !["patch", "minor", "major", "alpha", "beta", "rc", "release"].contains(&bump) {
        return Err("Unknown release bump".into());
    }
    let repository = authorize(root)?;
    let cut = github::fetch_main(root)?;
    let work = pr_start::worktree(&cut, root)?;
    let result = (|| {
        let base_version = github::version(&work)?;
        pr_start::prepare(bump, &work)?;
        let head = github::git(&["rev-parse", "HEAD"], &work)?;
        let tag = format!("v{}", github::version(&work)?);
        supports_pin(&head, &work)?;
        let branch = format!("release/{tag}");
        super::dispatch::version_owner(&repository, &tag, &work)?;
        let operator = super::lock::acquire(&tag, &head, &work)?;
        if !github::git(
            &[
                "ls-remote",
                "--heads",
                "origin",
                &format!("refs/heads/{branch}"),
            ],
            &work,
        )?
        .is_empty()
        {
            return Err("Release branch already exists; resume its PR with --pin.".into());
        }
        github::git(
            &["push", "origin", &format!("HEAD:refs/heads/{branch}")],
            &work,
        )?;
        let body = source_body(&tag, bump, &base_version, &cut, &head, None);
        let number = open_pr(
            &repository,
            &branch,
            &format!("chore(release): {tag} immutable source cut"),
            &body,
            true,
            &work,
        )?;
        let mut source = Source {
            candidate: Candidate {
                number,
                repository: repository.clone(),
                author: String::new(),
                branch,
                head: head.clone(),
                base: cut.clone(),
                tag: tag.clone(),
                merged: false,
            },
            cut,
            base_version,
            integration: 0,
            closed: false,
        };
        source.integration = integration_pr(&source, bump, &work)?;
        super::lock::install_pin(&source, &operator, &work)?;
        set_body(&source, bump, &work)?;
        watch(&repository, number, &head, &tag, false, &operator, &work)
    })();
    pr_start::finish(result, &work, root)
}

pub fn resume(number: u64, root: &Path) -> Result<(), String> {
    let repository = authorize(root)?;
    let pr = github::api(&repository, &format!("pulls/{number}"), root)?;
    let head = pr_contract::field(&pr, "/head/sha")?.to_string();
    let branch = pr_contract::field(&pr, "/head/ref")?.to_string();
    let tag = branch
        .strip_prefix("release/")
        .ok_or("Not a release source PR")?
        .to_string();
    super::dispatch::version_owner(&repository, &tag, root)?;
    github::git(&["fetch", "--no-tags", "origin", &head], root)?;
    // Old workflow bytecode cannot acquire new promotion semantics through a
    // CLI flag. A cut must already contain the reviewed opt-in implementation.
    supports_pin(&head, root)?;
    let work = pr_start::worktree(&head, root)?;
    let result = (|| {
        let body = pr.get("body").and_then(Value::as_str).unwrap_or("");
        let complete = body
            .lines()
            .any(|line| line.starts_with("<!-- vize-release-pin:"))
            && marker(body, "vize-release-integration").is_ok();
        if complete {
            source(&repository, number, &head, &tag, true, &work)?;
        } else {
            authenticate_partial(&pr, &repository, &head, &tag, &work)?;
        }
        let operator = super::lock::acquire(&tag, &head, &work)?;
        if !complete {
            if github::tag_target(&tag, &work)?.is_some() {
                return Err(
                    "A published legacy tag cannot be retrofitted into a pinned release.".into(),
                );
            }
            let bump = marker(body, "vize-release-bump")?;
            let base_version = marker(body, "vize-release-base-version")?;
            let cut = github::parent(&head, &work)?;
            metadata::verify_delta(
                &cut,
                &head,
                &base_version,
                tag.trim_start_matches('v'),
                &work,
            )?;
            let mut source = Source {
                candidate: Candidate {
                    number,
                    repository: repository.clone(),
                    author: String::new(),
                    branch: branch.clone(),
                    head: head.clone(),
                    base: cut.clone(),
                    tag: tag.clone(),
                    merged: false,
                },
                cut,
                base_version,
                integration: 0,
                closed: false,
            };
            source.integration = integration_pr(&source, &bump, &work)?;
            super::lock::install_pin(&source, &operator, &work)?;
            set_body(&source, &bump, &work)?;
        }
        watch(&repository, number, &head, &tag, true, &operator, &work)
    })();
    pr_start::finish(result, &work, root)
}

pub(super) fn authenticate_partial(
    pr: &Value,
    repository: &str,
    head: &str,
    tag: &str,
    root: &Path,
) -> Result<(), String> {
    pr_contract::candidate_fields(
        pr,
        &github::author(repository, pr_contract::field(pr, "/user/login")?, root)?,
        repository,
        head,
        tag,
        false,
    )?;
    let remote = github::git(
        &[
            "ls-remote",
            "--heads",
            "origin",
            &format!("refs/heads/release/{tag}"),
        ],
        root,
    )?;
    if remote.split_whitespace().next() != Some(head) {
        return Err("The partial source receipt and remote H disagree; no integration/marker can be created.".into());
    }
    let cut = github::parent(head, root)?;
    let body = pr
        .get("body")
        .and_then(Value::as_str)
        .ok_or("Missing partial source body")?;
    if body
        .lines()
        .any(|line| line.starts_with("<!-- vize-release-pin:"))
        && (marker(body, "vize-release-pin")? != "immutable-v1"
            || marker(body, "vize-release-pin-head")? != head
            || marker(body, "vize-release-pin-cut")? != cut)
    {
        return Err("The interrupted pin fields changed; preserve the source receipt.".into());
    }
    let base = marker(body, "vize-release-base-version")?;
    metadata::verify_delta(&cut, head, &base, tag.trim_start_matches('v'), root)
}

fn source_body(
    tag: &str,
    bump: &str,
    base_version: &str,
    cut: &str,
    head: &str,
    integration: Option<u64>,
) -> String {
    format!(
        "## Release {tag}\n\nImmutable source receipt C={cut}, H={head}. Full exact-H checks, release preflight, and all build artifacts qualify this cut. Ordinary main merges continue.\n\nThe separate version integration PR enters the protected queue under maintainer control. The official --pin command verifies actual signed delivery, identical registry closure, and every generated version byte, then creates only an annotated tag at H. This source PR stays draft and is never merged; close it only after successful external publication verification.\n\nResume: `vp run release --resume <PR> --pin`\n\n<!-- vize-release-bump: {bump} -->\n<!-- vize-release-base-version: {base_version} -->\n<!-- vize-release-pin: immutable-v1 -->\n<!-- vize-release-pin-cut: {cut} -->\n<!-- vize-release-pin-head: {head} -->\n{}",
        integration
            .map(|id| format!("<!-- vize-release-integration: {id} -->\n"))
            .unwrap_or_default()
    )
}

fn set_body(source: &Source, bump: &str, root: &Path) -> Result<(), String> {
    let body = source_body(
        &source.candidate.tag,
        bump,
        &source.base_version,
        &source.cut,
        &source.candidate.head,
        Some(source.integration),
    );
    let path = root.join(".git-release-pinned-body.md");
    fs::write(&path, body).map_err(|e| e.to_string())?;
    let result = github::output(
        "gh",
        &[
            "pr",
            "edit",
            &source.candidate.number.to_string(),
            "--repo",
            &source.candidate.repository,
            "--body-file",
            path.to_str().ok_or("Invalid receipt path")?,
        ],
        root,
    );
    let _ = fs::remove_file(path);
    result.map(|_| ())
}

pub(super) fn find_or_dispatch(source: &Source, root: &Path) -> Result<u64, String> {
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
    github::output(
        "gh",
        &[
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
        ],
        root,
    )?;
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_secs(2));
        if let Some(id) = find()? {
            return Ok(id);
        }
    }
    Err(
        "No identifiable pinned R appeared; preserve H and inspect dispatch before retrying."
            .into(),
    )
}
