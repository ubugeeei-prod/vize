use super::super::{
    pr_budget,
    pr_contract::{self, Candidate},
    pr_github as github, pr_start,
};
use super::integration::{PreparedNormalCut, integration_pr, open_pr};
use super::{Source, marker, metadata, source, watch};
#[path = "pr_pin_source_body.rs"]
mod body;
use body::{set_body, source_body};
use serde_json::Value;
use std::{env, path::Path};

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
    start_selected(bump, None, root)
}

pub fn start_stable(policy: &str, cut: &str, root: &Path) -> Result<(), String> {
    start_selected("patch", Some((policy, cut)), root)
}

fn start_selected(bump: &str, requested: Option<(&str, &str)>, root: &Path) -> Result<(), String> {
    let budget = pr_budget::configured()?;
    if !["patch", "minor", "major", "alpha", "beta", "rc", "release"].contains(&bump) {
        return Err("Unknown release bump".into());
    }
    let repository = authorize(root)?;
    let main = github::fetch_main(root)?;
    let expected = match env::var("VIZE_RELEASE_EXPECTED_CUT_SHA") {
        Ok(value) => Some(value),
        Err(env::VarError::NotPresent) => None,
        Err(_) => return Err("VIZE_RELEASE_EXPECTED_CUT_SHA is not valid Unicode.".into()),
    };
    expected_cut(expected.as_deref(), &main)?;
    let (cut, stable) = match requested {
        None => (main.clone(), None),
        Some((policy, cut)) => {
            if policy != main {
                return Err(
                    "Stable preparation policy is stale; redispatch from current main".into(),
                );
            }
            pr_contract::sha(cut)?;
            github::git(&["fetch", "--no-tags", "origin", cut], root)?;
            let (context, plan) = super::stable::load(policy, cut, &repository, root)?;
            supports_pin(cut, root)?;
            super::stable::supports_cut(&context, cut, root)?;
            let latest = github::api(&repository, "releases/latest", root)?;
            if pr_contract::field(&latest, "/tag_name")?
                != pr_contract::field(&plan, "/published/tag")?
            {
                return Err("A newer public version owns the stable line".into());
            }
            (cut.to_owned(), Some(context))
        }
    };
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
        if let Some(context) = &stable {
            if super::stable::target(context, &work)? != tag {
                return Err("Stable preparation produced a different PATCH version".into());
            }
            github::git(
                &[
                    "--no-replace-objects",
                    "cat-file",
                    "-e",
                    &format!("{head}:tools/support/release/pr_pin_stable.rs"),
                ],
                &work,
            )?;
        }
        let branch = format!("release/{tag}");
        super::dispatch::version_owner(&repository, &tag, &work)?;
        pr_budget::check(budget.as_ref())?;
        let operator = super::lock::acquire_for_start(
            &tag,
            &head,
            &cut,
            bump == "minor" && stable.is_none(),
            budget.as_ref(),
            &work,
        )?;
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
        let body = source_body(&tag, bump, &base_version, &cut, &head, None)
            + &super::stable::markers(stable.as_ref());
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
            stable: stable.clone(),
        };
        let prepared = PreparedNormalCut::capture(&source, bump, &work)?;
        source.integration =
            integration_pr(&source, bump, budget.as_ref(), prepared, &operator, &work)?;
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
        let (stable, operator) = resume_operator(&tag, &head, &work, &|| {
            let stable = if complete {
                source(&repository, number, &head, &tag, true, &work)?.stable
            } else {
                authenticate_partial(&pr, &repository, &head, &tag, &work)?
            };
            pr_budget::check(budget.as_ref())?;
            Ok(stable)
        })?;
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
                cut: cut.clone(),
                base_version,
                integration: 0,
                closed: false,
                stable,
            };
            source.integration =
                integration_pr(&source, &bump, budget.as_ref(), None, &operator, &work)?;
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

pub(super) fn resume_operator(
    tag: &str,
    head: &str,
    root: &Path,
    authenticate: &dyn Fn() -> Result<Option<super::stable::Stable>, String>,
) -> Result<(Option<super::stable::Stable>, super::lock::Operator), String> {
    let stable = authenticate()?;
    Ok((stable, super::lock::acquire(tag, head, root)?))
}

pub(super) fn authenticate_partial(
    pr: &Value,
    repository: &str,
    head: &str,
    tag: &str,
    root: &Path,
) -> Result<Option<super::stable::Stable>, String> {
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
    metadata::verify_delta(&cut, head, &base, tag.trim_start_matches('v'), root)?;
    super::stable::resume_authority(body, tag, &cut, root, &|| {
        super::stable::from_body(body, &cut, repository, root)
    })
}
