//! Bounded immutable metadata/failure-log custody; original artifact bodies remain in Actions.
use super::super::pr_github as github;
use super::metadata;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

pub(super) const MAX_BYTES: usize = 64 * 1024 * 1024;
pub(super) const MAX_LOG_BYTES: usize = 8 * 1024 * 1024;

pub(super) fn check_size(metadata_bytes: usize, log_bytes: usize) -> Result<(), String> {
    if log_bytes > MAX_LOG_BYTES {
        return Err("Failure logs exceed bounded 8 MiB aggregate".into());
    }
    if metadata_bytes
        .checked_add(log_bytes)
        .is_none_or(|total| total > MAX_BYTES)
    {
        return Err(format!(
            "Complete retirement archive exceeds 64 MiB: metadata={metadata_bytes} bytes; logs={log_bytes} bytes; limit={MAX_BYTES} bytes"
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "pr_pin_retire_size_tests.rs"]
mod size_tests;

pub(super) fn remote(reference: &str, root: &Path) -> Result<Option<String>, String> {
    let raw = github::git(&["ls-remote", "--refs", "origin", reference], root)?;
    let rows: Vec<_> = raw.lines().collect();
    match rows.as_slice() {
        [] => Ok(None),
        [row] => {
            let (sha, observed) = row.split_once('\t').ok_or("Invalid retirement ref row")?;
            super::super::pr_contract::sha(sha)?;
            if observed != reference {
                return Err("Retirement ref identity changed".into());
            }
            Ok(Some(sha.into()))
        }
        _ => Err("Ambiguous retirement ref inventory".into()),
    }
}

pub(super) fn reference(tag: &str) -> String {
    format!("refs/heads/release-retired/{tag}")
}

pub(super) fn identity(receipt: &Value) -> Result<&Value, String> {
    if receipt["schema"] != "vize-unpublished-retirement-v1" {
        return Err("Unknown immutable retirement schema".into());
    }
    let identity = receipt
        .get("identity")
        .filter(|v| v.is_object())
        .ok_or("Missing retirement identity")?;
    for key in ["head", "cut", "pin", "integrationHead"] {
        super::super::pr_contract::sha(identity[key].as_str().ok_or("Missing retirement SHA")?)?;
    }
    super::super::pr_contract::tag(identity["tag"].as_str().ok_or("Missing retirement tag")?)?;
    for key in [
        "sourcePr",
        "integrationPr",
        "releaseRun",
        "operatorRun",
        "releaseAttempt",
        "operatorAttempt",
    ] {
        if identity[key].as_u64().is_none_or(|n| n == 0) {
            return Err("Missing positive retirement identity".into());
        }
    }
    Ok(identity)
}

pub(super) fn read(tag: &str, root: &Path) -> Result<Option<(String, Value)>, String> {
    let reference = reference(tag);
    let Some(head) = remote(&reference, root)? else {
        return Ok(None);
    };
    github::git(&["fetch", "--no-tags", "origin", &reference], root)?;
    if github::git(&["rev-parse", "FETCH_HEAD"], root)? != head {
        return Err("Retirement archive changed while fetching".into());
    }
    let bytes = github::git(
        &["cat-file", "-s", &format!("{head}:retirement.json")],
        root,
    )?
    .parse::<usize>()
    .map_err(|_| "Invalid retirement receipt blob size")?;
    if bytes == 0 || bytes > MAX_BYTES {
        return Err("Retirement receipt exceeds bounded archive".into());
    }
    let receipt: Value = serde_json::from_str(&metadata::text(&head, "retirement.json", root)?)
        .map_err(|e| e.to_string())?;
    let fields = identity(&receipt)?;
    if fields["tag"].as_str() != Some(tag) {
        return Err("Retirement archive tag mismatch".into());
    }
    let parents = github::git(&["rev-list", "--parents", "-n", "1", &head], root)?;
    let expected = format!(
        "{head} {} {}",
        fields["pin"].as_str().unwrap(),
        fields["integrationHead"].as_str().unwrap()
    );
    if parents != expected {
        return Err("Retirement archive does not retain exact pin/H/C and M parents".into());
    }
    let rows = github::git(&["ls-tree", "-r", "--long", &head], root)?;
    let mut total = 0usize;
    let mut observed = BTreeMap::new();
    for row in rows.lines() {
        let (entry, path) = row.split_once('\t').ok_or("Invalid archive tree row")?;
        let fields: Vec<_> = entry.split_whitespace().collect();
        if fields.len() != 4
            || fields[0] != "100644"
            || fields[1] != "blob"
            || !(path == "retirement.json"
                || (path.starts_with("failure/") && path.ends_with(".log")))
        {
            return Err("Archive contains an unexpected path/type/mode".into());
        }
        let size = fields[3]
            .parse::<usize>()
            .map_err(|_| "Invalid archive blob size")?;
        total = total.checked_add(size).ok_or("Archive size overflow")?;
        if total > MAX_BYTES {
            return Err("Retirement metadata archive exceeds 64 MiB".into());
        }
        if path != "retirement.json" {
            observed.insert(path.to_string(), json!({"oid":fields[2],"bytes":size}));
        }
    }
    if receipt["failureLogObjects"] != json!(observed) {
        return Err("Retirement failure log objects and immutable ledger differ".into());
    }
    super::retire_ledger::validate(&receipt, root)?;
    if remote(&reference, root)?.as_deref() != Some(&head) {
        return Err("Retirement archive changed after reading".into());
    }
    Ok(Some((head, receipt)))
}

fn blob(bytes: &[u8], root: &Path) -> Result<String, String> {
    let mut child = Command::new("git")
        .args(["hash-object", "-w", "--stdin"])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    child
        .stdin
        .take()
        .ok_or("Missing Git blob stdin")?
        .write_all(bytes)
        .map_err(|e| e.to_string())?;
    let result = child.wait_with_output().map_err(|e| e.to_string())?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).into());
    }
    let sha = String::from_utf8(result.stdout)
        .map_err(|e| e.to_string())?
        .trim()
        .to_string();
    super::super::pr_contract::sha(&sha)?;
    Ok(sha)
}

/// Create a new archive only; a foreign or changed archive is never overwritten.
#[cfg(test)]
pub(super) fn install(
    receipt: Value,
    logs: &BTreeMap<String, Vec<u8>>,
    root: &Path,
) -> Result<String, String> {
    install_authorized(receipt, logs, root, &|| Ok(()))
}

pub(super) fn install_authorized(
    mut receipt: Value,
    logs: &BTreeMap<String, Vec<u8>>,
    root: &Path,
    authorize: &dyn Fn() -> Result<(), String>,
) -> Result<String, String> {
    let fields = identity(&receipt)?.clone();
    let tag = fields["tag"].as_str().unwrap();
    if let Some((head, prior)) = read(tag, root)? {
        if identity(&prior)? != &fields {
            return Err("Existing retirement archive belongs to another cut".into());
        }
        return Ok(head);
    }
    let mut objects = BTreeMap::new();
    let mut entries = Vec::new();
    let mut total = 0usize;
    for (path, bytes) in logs {
        let number = path
            .strip_prefix("failure/")
            .and_then(|v| v.strip_suffix(".log"));
        if number.is_none_or(|v| v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit())) {
            return Err("Unsafe retirement failure-log path".into());
        }
        total = total
            .checked_add(bytes.len())
            .ok_or("Archive size overflow")?;
        if total > MAX_LOG_BYTES {
            return Err("Failure logs exceed bounded 8 MiB aggregate".into());
        }
        let oid = blob(bytes, root)?;
        objects.insert(path.clone(), json!({"oid":oid,"bytes":bytes.len()}));
        entries.push(format!("100644 blob {oid}\t{path}\n"));
    }
    receipt["failureLogObjects"] = json!(objects);
    super::retire_ledger::validate(&receipt, root)?;
    let encoded = serde_json::to_vec_pretty(&receipt).map_err(|e| e.to_string())?;
    check_size(encoded.len(), total)?;
    let oid = blob(&encoded, root)?;
    entries.push(format!("100644 blob {oid}\tretirement.json\n"));
    entries.sort();
    // Paths are deliberately shallow; mktree builds the failure subtree explicitly.
    let failure_entries: String = entries
        .iter()
        .filter_map(|row| {
            row.split_once("\tfailure/")
                .map(|(entry, path)| format!("{entry}\t{path}"))
        })
        .collect();
    let failure_tree = tree(failure_entries.as_bytes(), root)?;
    let tree = tree(
        format!("040000 tree {failure_tree}\tfailure\n100644 blob {oid}\tretirement.json\n")
            .as_bytes(),
        root,
    )?;
    let message = format!(
        "Vize retired unpublished release {tag}\n\nSource-PR: #{}\nSource-head: {}\nRelease-run: {}\nOriginal objects and artifact metadata remain diagnostic evidence only.",
        fields["sourcePr"],
        fields["head"].as_str().unwrap(),
        fields["releaseRun"]
    );
    let head = github::git(
        &[
            "commit-tree",
            &tree,
            "-p",
            fields["pin"].as_str().unwrap(),
            "-p",
            fields["integrationHead"].as_str().unwrap(),
            "-m",
            &message,
        ],
        root,
    )?;
    let reference = reference(tag);
    authorize()?;
    let pushed = github::git(
        &[
            "push",
            "--atomic",
            &format!("--force-with-lease={reference}:"),
            "origin",
            &format!("{head}:{reference}"),
        ],
        root,
    );
    if let Err(error) = pushed {
        if remote(&reference, root)?.as_deref() != Some(&head) {
            return Err(error);
        }
    }
    let (observed, archived) = read(tag, root)?.ok_or("Archive disappeared after installation")?;
    if observed != head || archived != receipt {
        return Err("Archive installation custody mismatch".into());
    }
    Ok(head)
}

fn tree(bytes: &[u8], root: &Path) -> Result<String, String> {
    let mut child = Command::new("git")
        .arg("mktree")
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    child
        .stdin
        .take()
        .ok_or("Missing Git tree stdin")?
        .write_all(bytes)
        .map_err(|e| e.to_string())?;
    let result = child.wait_with_output().map_err(|e| e.to_string())?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).into());
    }
    Ok(String::from_utf8(result.stdout)
        .map_err(|e| e.to_string())?
        .trim()
        .into())
}
