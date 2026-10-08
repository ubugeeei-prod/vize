//! Cooperative operator leases and the durable pin-marker receipt.
use super::super::{pr_contract, pr_github as github};
use super::{Source, metadata};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) struct Operator {
    root: PathBuf,
    directory: PathBuf,
    remote_ref: String,
    remote_head: String,
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
        let message = format!(
            "Vize release operator\n\nTag: {tag}\nSource-head: {head}\nPID: {}\nNonce: {nonce}",
            std::process::id()
        );
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
        })
    })();
    if result.is_err() && (!attempted || rejected) {
        let _ = fs::remove_dir_all(&directory);
    }
    result
}

impl Operator {
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
    )
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
