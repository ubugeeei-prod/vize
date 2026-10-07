use super::super::{pr_contract, pr_github as github};
use super::delivery::Receipt;
use super::{Source, metadata};
use std::{path::Path, process::Command};

pub(super) fn annotation(source: &Source, id: u64, receipt: &Receipt) -> String {
    format!(
        "Release {}\n\nRelease-Mode: pinned\nRelease-PR: #{}\nValidated-run: {id}\nValidated-head: {}\nValidated-base: {}\nIntegration-PR: #{}\nIntegration-head: {}\nIntegration-merge: {}\nIntegration-parent: {}\nProtected-candidate: {}",
        source.candidate.tag,
        source.candidate.number,
        source.candidate.head,
        source.cut,
        source.integration,
        receipt.head,
        receipt.merge,
        receipt.parent,
        receipt.gate
    )
}

pub(super) fn tag_receipt(source: &Source, root: &Path) -> Result<(u64, Receipt), String> {
    let candidate = &source.candidate;
    if github::tag_target(&candidate.tag, root)?.as_deref() != Some(&candidate.head) {
        return Err("The remote immutable tag does not peel to H.".into());
    }
    github::git(
        &[
            "fetch",
            "--no-tags",
            "origin",
            &format!("refs/tags/{}", candidate.tag),
        ],
        root,
    )?;
    parse_tag(source, "FETCH_HEAD", root)
}

fn parse_tag(source: &Source, revision: &str, root: &Path) -> Result<(u64, Receipt), String> {
    let candidate = &source.candidate;
    if github::git(&["cat-file", "-t", revision], root)? != "tag" {
        return Err("Pinned releases require an annotated immutable tag receipt.".into());
    }
    let raw = String::from_utf8(metadata::bytes(&["cat-file", "-p", revision], root)?)
        .map_err(|e| e.to_string())?;
    let (headers, message) = raw
        .split_once("\n\n")
        .ok_or("Missing annotated tag receipt")?;
    if !headers
        .lines()
        .any(|line| line == format!("object {}", candidate.head))
        || !headers.lines().any(|line| line == "type commit")
        || !headers
            .lines()
            .any(|line| line == format!("tag {}", candidate.tag))
    {
        return Err("Fetched annotated tag object/name does not bind the immutable H.".into());
    }
    let body = message.trim_end();
    let field = |key: &str| -> Result<String, String> {
        let prefix = format!("{key}: ");
        let found: Vec<_> = body
            .lines()
            .filter_map(|line| line.strip_prefix(&prefix))
            .collect();
        match found.as_slice() {
            [value] if !value.is_empty() => Ok((*value).into()),
            _ => Err(format!("Missing/ambiguous tag field {key}")),
        }
    };
    let id = field("Validated-run")?
        .parse::<u64>()
        .map_err(|_| "Invalid annotated release run")?;
    if id == 0 {
        return Err("Invalid annotated release run".into());
    }
    let receipt = Receipt {
        head: field("Integration-head")?,
        merge: field("Integration-merge")?,
        parent: field("Integration-parent")?,
        gate: field("Protected-candidate")?,
    };
    for sha in [
        &receipt.head,
        &receipt.merge,
        &receipt.parent,
        &receipt.gate,
    ] {
        pr_contract::sha(sha)?;
    }
    if body != annotation(source, id, &receipt) {
        return Err(
            "The annotated tag belongs to a different cut/run/integration protocol.".into(),
        );
    }
    Ok((id, receipt))
}

pub(super) fn ensure_local_tag(
    source: &Source,
    id: u64,
    receipt: &Receipt,
    root: &Path,
) -> Result<(), String> {
    let tag_ref = format!("refs/tags/{}", source.candidate.tag);
    let found = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", &tag_ref])
        .current_dir(root)
        .output()
        .map_err(|e| e.to_string())?;
    if !found.status.success() {
        if found.status.code() != Some(1) {
            return Err(format!(
                "Cannot inspect local tag: {}",
                String::from_utf8_lossy(&found.stderr)
            ));
        }
        github::git(
            &[
                "tag",
                "-a",
                &source.candidate.tag,
                &source.candidate.head,
                "-m",
                &annotation(source, id, receipt),
            ],
            root,
        )?;
    }
    let (recorded_id, recorded) = parse_tag(source, &tag_ref, root)?;
    if recorded_id != id
        || annotation(source, recorded_id, &recorded) != annotation(source, id, receipt)
    {
        return Err("An existing local tag belongs to another cut/run/integration; preserve it without overwrite.".into());
    }
    Ok(())
}

pub(super) fn push_tag(root: &Path, tag_ref: &str) -> Result<String, String> {
    github::git(&["push", "origin", tag_ref], root)
}
