use super::super::{
    pr_budget,
    pr_contract::{self, Candidate},
    pr_github as github, pr_start,
};
use super::integration::{integration_pr, open_pr};
use super::{Source, marker, metadata, source, watch};
use serde_json::Value;
use std::{env, fs, path::Path};

pub(super) fn expected_cut(expected: Option<&str>, cut: &str) -> Result<(), String> {
    if let Some(expected) = expected {
        pr_contract::sha(expected)?;
        if cut != expected {
            return Err("The expected release cut differs from current main; redispatch from current main before preparation or source mutation.".into());
        }
    }
    Ok(())
}

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
    let budget = pr_budget::configured()?;
    if !["patch", "minor", "major", "alpha", "beta", "rc", "release"].contains(&bump) {
        return Err("Unknown release bump".into());
    }
    let repository = authorize(root)?;
    let cut = github::fetch_main(root)?;
    let expected = match env::var("VIZE_RELEASE_EXPECTED_CUT_SHA") {
        Ok(value) => Some(value),
        Err(env::VarError::NotPresent) => None,
        Err(_) => return Err("VIZE_RELEASE_EXPECTED_CUT_SHA is not valid Unicode.".into()),
    };
    expected_cut(expected.as_deref(), &cut)?;
    let work = pr_start::pinned_worktree(&cut, root)?;
    let result = (|| {
        let base_version = github::version(&work)?;
        pr_budget::check(budget.as_ref())?;
        if bump == "minor" {
            let target =
                super::retire::next_minor(&repository, &base_version, budget.as_ref(), &work)?;
            pr_budget::check(budget.as_ref())?;
            super::retire::prepare(bump, &format!("v{target}"), budget.as_ref(), &work)?;
        } else {
            pr_start::prepare(bump, &work)?;
        }
        let head = github::git(&["rev-parse", "HEAD"], &work)?;
        let tag = format!("v{}", github::version(&work)?);
        supports_pin(&head, &work)?;
        let branch = format!("release/{tag}");
        super::dispatch::version_owner(&repository, &tag, &work)?;
        pr_budget::check(budget.as_ref())?;
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
        pr_budget::check(budget.as_ref())?;
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
        source.integration = integration_pr(&source, bump, budget.as_ref(), &work)?;
        pr_budget::check(budget.as_ref())?;
        super::lock::install_pin(&source, &operator, &work)?;
        set_body(&source, bump, &work)?;
        watch(
            &repository,
            number,
            &head,
            &tag,
            false,
            &operator,
            budget.as_ref(),
            &work,
        )
    })();
    pr_start::finish(result, &work, root)
}

pub fn resume(number: u64, root: &Path) -> Result<(), String> {
    let budget = pr_budget::configured()?;
    let repository = authorize(root)?;
    let pr = github::api(&repository, &format!("pulls/{number}"), root)?;
    let head = pr_contract::field(&pr, "/head/sha")?.to_string();
    let branch = pr_contract::field(&pr, "/head/ref")?.to_string();
    let tag = branch
        .strip_prefix("release/")
        .ok_or("Not a release source PR")?
        .to_string();
    super::retire::reject_resume(&tag, root)?;
    super::dispatch::version_owner(&repository, &tag, root)?;
    github::git(&["fetch", "--no-tags", "origin", &head], root)?;
    // Old workflow bytecode cannot acquire new promotion semantics through a
    // CLI flag. A cut must already contain the reviewed opt-in implementation.
    supports_pin(&head, root)?;
    let work = pr_start::pinned_worktree(&head, root)?;
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
        pr_budget::check(budget.as_ref())?;
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
            source.integration = integration_pr(&source, &bump, budget.as_ref(), &work)?;
            pr_budget::check(budget.as_ref())?;
            super::lock::install_pin(&source, &operator, &work)?;
            set_body(&source, &bump, &work)?;
        }
        watch(
            &repository,
            number,
            &head,
            &tag,
            true,
            &operator,
            budget.as_ref(),
            &work,
        )
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
