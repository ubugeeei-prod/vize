//! Cooperative operator leases and the durable pin-marker receipt.
use super::super::{pr_budget, pr_contract, pr_github as github};
use super::{Source, metadata};
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(test)]
#[path = "pr_pin_lock_tests.rs"]
mod tests;

pub(crate) struct Operator {
    root: PathBuf,
    directory: PathBuf,
    remote_ref: String,
    remote_head: String,
    admission: Option<Admission>,
}

struct Admission {
    run: u64,
    attempt: u64,
    source: String,
    deadline: u64,
}

fn validate_operator_run(
    observed: &serde_json::Value,
    run: u64,
    attempt: u64,
    repository: &str,
    cut: &str,
    actor: &str,
) -> Result<(), String> {
    if observed["id"].as_u64() != Some(run)
        || observed["run_attempt"].as_u64() != Some(attempt)
        || observed["head_sha"] != cut
        || observed["head_branch"] != "main"
        || observed["event"] != "workflow_dispatch"
        || observed["path"] != ".github/workflows/release-operator.yml"
        || observed["status"] != "in_progress"
        || observed.get("conclusion") != Some(&serde_json::Value::Null)
        || observed
            .pointer("/head_repository/full_name")
            .and_then(serde_json::Value::as_str)
            != Some(repository)
        || observed
            .pointer("/actor/login")
            .and_then(serde_json::Value::as_str)
            != Some(actor)
        || observed
            .pointer("/triggering_actor/login")
            .and_then(serde_json::Value::as_str)
            != Some(actor)
    {
        return Err("The official operator run/source/attempt/actor changed".into());
    }
    Ok(())
}

fn positive(value: &str) -> Result<u64, String> {
    let number = value
        .parse::<u64>()
        .map_err(|_| "Invalid operator identity")?;
    if number == 0 || number.to_string() != value {
        return Err("Canonical positive operator identity required".into());
    }
    Ok(number)
}

fn hosted_admission(
    cut: &str,
    budget: Option<&pr_budget::Budget>,
    root: &Path,
) -> Result<Option<Admission>, String> {
    if env::var_os("GITHUB_ACTIONS").is_none() {
        return Ok(None);
    }
    if env::var("GITHUB_ACTIONS").as_deref() != Ok("true") {
        return Err("Invalid hosted operator environment".into());
    }
    let required = |name| env::var(name).map_err(|_| format!("Missing official {name}"));
    let run = positive(&required("GITHUB_RUN_ID")?)?;
    let attempt = positive(&required("GITHUB_RUN_ATTEMPT")?)?;
    let repository = github::repository(root)?;
    if required("GITHUB_REPOSITORY")? != repository
        || required("GITHUB_SHA")? != cut
        || required("GITHUB_REF")? != "refs/heads/main"
        || required("GITHUB_EVENT_NAME")? != "workflow_dispatch"
    {
        return Err("Fresh early integration requires the exact official source/main event".into());
    }
    let observed = github::api(&repository, &format!("actions/runs/{run}"), root)?;
    let actor = required("GITHUB_ACTOR")?;
    if actor != required("GITHUB_TRIGGERING_ACTOR")? {
        return Err("The original and current operator actors differ".into());
    }
    validate_operator_run(&observed, run, attempt, &repository, cut, &actor)?;
    pr_contract::maintainer(&github::author(&repository, &actor, root)?)?;
    let (_, deadline) = budget
        .ok_or("Early protected integration requires the existing absolute operator budget")?
        .child_allowance()?;
    if deadline > 9_007_199_254_740_991 {
        return Err(
            "Operator deadline cannot be represented exactly by the read-only verifier".into(),
        );
    }
    Ok(Some(Admission {
        run,
        attempt,
        source: cut.into(),
        deadline,
    }))
}

/// Only an official fresh minor cut can authorize pending-Q protected checks.
/// Local commands, stable patches, legacy cuts and resumes retain serial admission.
pub(crate) fn acquire_for_start(
    tag: &str,
    head: &str,
    cut: &str,
    fresh_minor: bool,
    budget: Option<&pr_budget::Budget>,
    root: &Path,
) -> Result<Operator, String> {
    let admission = if fresh_minor {
        hosted_admission(cut, budget, root)?
    } else {
        None
    };
    acquire_with_admission(tag, head, admission, root)
}

fn remote(ref_name: &str, root: &Path) -> Result<Option<String>, String> {
    let value = github::git(&["ls-remote", "--refs", "origin", ref_name], root)?;
    Ok(value.split_whitespace().next().map(str::to_string))
}

pub(crate) fn reject_pinned(tag: &str, root: &Path) -> Result<(), String> {
    pr_contract::tag(tag)?;
    if remote(&format!("refs/heads/release-pin/{tag}"), root)?.is_some() {
        return Err("An immutable pin owns this release; legacy source refresh/promotion is forbidden. Resume with --pin.".into());
    }
    Ok(())
}

fn commit(head: &str, message: &str, root: &Path) -> Result<String, String> {
    let tree = github::git(&["rev-parse", &format!("{head}^{{tree}}")], root)?;
    github::git(&["commit-tree", &tree, "-p", head, "-m", message], root)
}

pub(crate) fn acquire(tag: &str, head: &str, root: &Path) -> Result<Operator, String> {
    acquire_with_admission(tag, head, None, root)
}

fn acquire_with_admission(
    tag: &str,
    head: &str,
    admission: Option<Admission>,
    root: &Path,
) -> Result<Operator, String> {
    pr_contract::tag(tag)?;
    pr_contract::sha(head)?;
    let common = PathBuf::from(github::git(
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        root,
    )?);
    let directory = common.join(format!("vize-release-operator-{tag}.lock"));
    fs::create_dir(&directory).map_err(|e|format!("Another release operator owns {}: {e}. Inspect its owner receipt; never steal a live or ambiguous lease.",directory.display()))?;
    let mut attempted = false;
    let mut rejected = false;
    let result = (|| {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let message = match &admission {
            None => format!(
                "Vize release operator\n\nTag: {tag}\nSource-head: {head}\nPID: {}\nNonce: {nonce}",
                std::process::id()
            ),
            Some(admission) => format!(
                "Vize release operator v2\n\nTag: {tag}\nSource-head: {head}\nOperator-run: {}\nOperator-attempt: {}\nOperator-source: {}\nOriginal-deadline-unix-millis: {}\nPID: {}\nNonce: {nonce}",
                admission.run,
                admission.attempt,
                admission.source,
                admission.deadline,
                std::process::id()
            ),
        };
        fs::write(directory.join("owner.txt"), &message).map_err(|e| e.to_string())?;
        let remote_head = commit(head, &message, root)?;
        fs::write(directory.join("remote-head.txt"), &remote_head).map_err(|e| e.to_string())?;
        let remote_ref = format!("refs/heads/release-operator/{tag}");
        attempted = true;
        let pushed = github::git(
            &[
                "push",
                &format!("--force-with-lease={remote_ref}:"),
                "origin",
                &format!("{remote_head}:{remote_ref}"),
            ],
            root,
        );
        if let Err(error) = pushed {
            let observed = remote(&remote_ref, root)?;
            if observed.as_deref() != Some(&remote_head) {
                rejected = observed.is_some();
                return Err(format!(
                    "{error}\nAnother/unknown remote release operator holds {remote_ref}; no source/tag/main was changed."
                ));
            }
        }
        Ok(Operator {
            root: root.into(),
            directory: directory.clone(),
            remote_ref,
            remote_head,
            admission,
        })
    })();
    if result.is_err() && (!attempted || rejected) {
        let _ = fs::remove_dir_all(&directory);
    }
    result
}

impl Operator {
    pub(crate) fn allows_early_admission(&self) -> bool {
        self.admission.is_some()
    }

    /// Bounded rollback reads identity only; never renew D or qualify artifacts.
    pub(super) fn compensation_authority(
        &self,
        source: &Source,
        actor: &str,
        adapter: &mut super::queue::Adapter<'_>,
    ) -> Result<super::queue::Authority, String> {
        let admission = self
            .admission
            .as_ref()
            .ok_or("No original compensation authority")?;
        if admission.source != source.cut
            || actor.is_empty()
            || !actor
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err("Original operator source changed".into());
        }
        let repository = &source.candidate.repository;
        let h = &source.candidate.head;
        let lease_ref = adapter.read(
            "rollback-lease-ref",
            repository,
            &format!("git/ref/heads/release-operator/{}", source.candidate.tag),
        )?;
        if lease_ref["ref"] != self.remote_ref
            || lease_ref["object"]["type"] != "commit"
            || lease_ref["object"]["sha"] != self.remote_head
        {
            return Err("Original operator lease changed; compensation forbidden".into());
        }
        let run = adapter.read(
            "rollback-operator",
            repository,
            &format!("actions/runs/{}", admission.run),
        )?;
        validate_operator_run(
            &run,
            admission.run,
            admission.attempt,
            repository,
            &source.cut,
            actor,
        )?;
        pr_contract::maintainer(&adapter.read(
            "rollback-operator-permission",
            repository,
            &format!("collaborators/{actor}/permission"),
        )?)?;
        let head = adapter.commit(repository, h, &source.cut, None)?;
        let tree = pr_contract::field(&head, "/tree/sha")?;
        let owned =
            fs::read_to_string(self.directory.join("owner.txt")).map_err(|e| e.to_string())?;
        let tag = &source.candidate.tag;
        let Admission {
            run,
            attempt,
            source: cut,
            deadline,
        } = admission;
        let prefix = format!(
            "Vize release operator v2\n\nTag: {tag}\nSource-head: {h}\nOperator-run: {run}\nOperator-attempt: {attempt}\nOperator-source: {cut}\nOriginal-deadline-unix-millis: {deadline}\nPID: "
        );
        if !owned.starts_with(&prefix) {
            return Err("Original lease deadline changed".into());
        }
        adapter.commit(repository, &self.remote_head, h, Some((tree, &owned)))?;
        let pin_ref = adapter.read(
            "rollback-pin-ref",
            repository,
            &format!("git/ref/heads/release-pin/{}", source.candidate.tag),
        )?;
        if pin_ref["ref"] != format!("refs/heads/release-pin/{}", source.candidate.tag)
            || pin_ref["object"]["type"] != "commit"
        {
            return Err("Original pin ref changed".into());
        }
        let pin_sha = pr_contract::field(&pin_ref, "/object/sha")?;
        pr_contract::sha(pin_sha)?;
        let expected = pin_message(source);
        adapter.commit(repository, pin_sha, h, Some((tree, &expected)))?;
        Ok(super::queue::Authority {
            lease: self.remote_head.clone(),
            operator_run: admission.run,
            operator_attempt: admission.attempt,
        })
    }

    #[cfg(test)]
    pub(super) fn compensation_fixture(
        source: &Source,
        deadline: u64,
        root: &Path,
    ) -> Result<Self, String> {
        acquire_with_admission(
            &source.candidate.tag,
            &source.candidate.head,
            Some(Admission {
                run: 7,
                attempt: 2,
                source: source.cut.clone(),
                deadline,
            }),
            root,
        )
    }

    pub(super) fn queue_authority(
        &self,
        cut: &str,
        root: &Path,
    ) -> Result<super::queue::Authority, String> {
        self.verify()?;
        let admission = self
            .admission
            .as_ref()
            .ok_or("No original early-queue authority")?;
        if admission.source != cut {
            return Err("The original operator source changed".into());
        }
        let repository = github::repository(root)?;
        let observed = github::api(
            &repository,
            &format!("actions/runs/{}", admission.run),
            root,
        )?;
        let actor = env::var("GITHUB_ACTOR").map_err(|_| "Missing original operator actor")?;
        if env::var("GITHUB_TRIGGERING_ACTOR").as_deref() != Ok(actor.as_str()) {
            return Err("Original operator actor changed".into());
        }
        validate_operator_run(
            &observed,
            admission.run,
            admission.attempt,
            &repository,
            cut,
            &actor,
        )?;
        pr_contract::maintainer(&github::author(&repository, &actor, root)?)?;
        Ok(super::queue::Authority {
            lease: self.remote_head.clone(),
            operator_run: admission.run,
            operator_attempt: admission.attempt,
        })
    }

    pub(crate) fn verify(&self) -> Result<(), String> {
        if remote(&self.remote_ref, &self.root)?.as_deref() != Some(&self.remote_head) {
            return Err(
                "The active release operator lease changed; promotion is forbidden.".into(),
            );
        }
        Ok(())
    }
}

impl Drop for Operator {
    fn drop(&mut self) {
        let released = github::git(
            &[
                "push",
                &format!(
                    "--force-with-lease={}:{}",
                    self.remote_ref, self.remote_head
                ),
                "origin",
                &format!(":{}", self.remote_ref),
            ],
            &self.root,
        );
        if released.is_ok() {
            let _ = fs::remove_dir_all(&self.directory);
        } else {
            eprintln!(
                "Release operator lease cleanup is ambiguous; preserved {} and {} for explicit ownership recovery.",
                self.directory.display(),
                self.remote_ref
            );
        }
    }
}

fn pin_message(source: &Source) -> String {
    format!(
        "Vize immutable release cut\n\nSource-PR: #{}\nSource-head: {}\nSource-cut: {}\nTag: {}\nBase-version: {}\nIntegration-PR: #{}",
        source.candidate.number,
        source.candidate.head,
        source.cut,
        source.candidate.tag,
        source.base_version,
        source.integration
    ) + &super::stable::annotation(source.stable.as_ref())
}

pub(super) fn verify_pin(source: &Source, root: &Path) -> Result<(), String> {
    verify_source_head(source, root)?;
    let pin_ref = format!("refs/heads/release-pin/{}", source.candidate.tag);
    if remote(&pin_ref, root)?.is_none() {
        return Err("The durable immutable pin marker is absent.".into());
    }
    github::git(&["fetch", "--no-tags", "origin", &pin_ref], root)?;
    let pin = github::git(&["rev-parse", "FETCH_HEAD"], root)?;
    if github::parent(&pin, root)? != source.candidate.head
        || github::git(&["rev-parse", &format!("{pin}^{{tree}}")], root)?
            != github::git(
                &["rev-parse", &format!("{}^{{tree}}", source.candidate.head)],
                root,
            )?
    {
        return Err(
            "The durable pin marker does not bind the immutable source tree/commit.".into(),
        );
    }
    let raw = String::from_utf8(metadata::bytes(&["cat-file", "-p", &pin], root)?)
        .map_err(|e| e.to_string())?;
    if raw.split_once("\n\n").map(|(_, body)| body.trim_end()) != Some(pin_message(source).as_str())
    {
        return Err("The durable pin marker/source receipt fields disagree.".into());
    }
    Ok(())
}

fn verify_source_head(source: &Source, root: &Path) -> Result<(), String> {
    let source_ref = format!("refs/heads/{}", source.candidate.branch);
    if remote(&source_ref, root)?.as_deref() != Some(&source.candidate.head) {
        return Err("The remote source changed during pin installation; preserve the marker and stop without rewriting source or retiring legacy evidence.".into());
    }
    Ok(())
}

pub(super) fn install_pin(source: &Source, operator: &Operator, root: &Path) -> Result<(), String> {
    operator.verify()?;
    verify_source_head(source, root)?;
    let candidate = &source.candidate;
    let pin_ref = format!("refs/heads/release-pin/{}", candidate.tag);
    if remote(&pin_ref, root)?.is_some() {
        return verify_pin(source, root);
    }
    let pin = commit(&candidate.head, &pin_message(source), root)?;
    let source_ref = format!("refs/heads/{}", candidate.branch);
    // The exact lease rejects a changed advertised source without rewinding it.
    // Git can omit an unchanged H→H update; this is not a source CAS. Fresh
    // pre/post custody checks and the cooperative operator lease are required.
    // A raced marker is preserved, but cannot authorize subsequent mutation.
    let pushed = github::git(
        &[
            "push",
            "--atomic",
            &format!("--force-with-lease={source_ref}:{}", candidate.head),
            &format!("--force-with-lease={pin_ref}:"),
            "origin",
            &format!("{}:{source_ref}", candidate.head),
            &format!("{pin}:{pin_ref}"),
        ],
        root,
    );
    if let Err(error) = pushed {
        if remote(&pin_ref, root)?.is_none() {
            return Err(error);
        }
    }
    verify_pin(source, root)
}
