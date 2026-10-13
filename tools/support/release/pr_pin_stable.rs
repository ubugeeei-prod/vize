//! An explicit reviewed stable cut; it never substitutes moving main's product graph.
use super::super::{pr_contract, pr_github as github};
use super::{Source, first_parent, marker, metadata};
use serde_json::Value;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
#[path = "pr_pin_stable_archive.rs"]
mod archive;
#[path = "pr_pin_stable_files.rs"]
mod files;
#[path = "pr_pin_stable_plan.rs"]
mod plan;
pub(super) use archive::{capture, from_archive};

pub(super) const PLAN_PATH: &str = "tools/support/release/stable_patch_plan.json";

#[derive(Clone, Debug)]
pub(super) struct Stable {
    pub policy: String,
    pub blob: String,
    pub sha256: String,
}

fn digest(bytes: &[u8]) -> Result<String, String> {
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("Stable manifest exceeds the complete 2 MiB plan bound".into());
    }
    let mut child = Command::new("python3")
        .args([
            "-c",
            "import hashlib,json,sys\ndef unique(pairs):\n d={}\n for k,v in pairs:\n  if k in d: raise ValueError(\"duplicate manifest field\")\n  d[k]=v\n return d\nb=sys.stdin.buffer.read()\njson.loads(b,object_pairs_hook=unique)\nprint(hashlib.sha256(b).hexdigest())",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    child
        .stdin
        .take()
        .ok_or("Missing digest input")?
        .write_all(bytes)
        .map_err(|e| e.to_string())?;
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("Cannot authenticate the complete stable plan digest".into());
    }
    let value = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    let value = value.trim();
    if value.len() != 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("Invalid stable plan SHA256".into());
    }
    Ok(value.into())
}

pub(super) fn load(
    policy: &str,
    cut: &str,
    repository: &str,
    root: &Path,
) -> Result<(Stable, Value), String> {
    super::retire_guard::no_replacements(root)?;
    pr_contract::sha(policy)?;
    pr_contract::sha(cut)?;
    let (context, value) = read_plan(policy, root)?;
    plan::verify(&value, &context, cut, repository, root)?;
    Ok((context, value))
}

fn read_plan(policy: &str, root: &Path) -> Result<(Stable, Value), String> {
    pr_contract::sha(policy)?;
    let blob = manifest_blob(policy, root)?;
    let bytes = metadata::bytes(&["--no-replace-objects", "cat-file", "blob", &blob], root)?;
    let context = Stable {
        policy: policy.into(),
        blob,
        sha256: digest(&bytes)?,
    };
    let value = serde_json::from_slice(&bytes).map_err(|e| format!("Invalid stable plan: {e}"))?;
    Ok((context, value))
}

fn manifest_blob(policy: &str, root: &Path) -> Result<String, String> {
    let raw = metadata::bytes(
        &[
            "--no-replace-objects",
            "ls-tree",
            "-z",
            policy,
            "--",
            PLAN_PATH,
        ],
        root,
    )?;
    let record = raw
        .strip_suffix(&[0])
        .ok_or("Missing regular stable manifest record")?;
    let text = std::str::from_utf8(record).map_err(|e| e.to_string())?;
    let (header, path) = text
        .split_once('\t')
        .ok_or("Invalid stable manifest record")?;
    let fields: Vec<_> = header.split(' ').collect();
    if fields.len() != 3 || fields[0] != "100644" || fields[1] != "blob" || path != PLAN_PATH {
        return Err(
            "Stable manifest must be exactly one regular 100644 blob at its literal path".into(),
        );
    }
    pr_contract::sha(fields[2])?;
    Ok(fields[2].into())
}

fn protocol_paths() -> &'static [&'static str] {
    &[
        "tools/commands/release/pr.rs",
        "tools/support/release/pr_pin.rs",
        "tools/support/release/pr_pin_stable.rs",
        "tools/support/release/pr_pin_stable_plan.rs",
        "tools/support/release/pr_pin_stable_files.rs",
        "tools/support/release/pr_pin_stable_archive.rs",
        "tools/support/release/pr_pin_start.rs",
        "tools/support/release/pr_pin_source_body.rs",
        "tools/support/release/pr_pin_candidate.rs",
        "tools/support/release/pr_pin_delivery.rs",
        "tools/support/release/pr_pin_integration.rs",
        "tools/support/release/pr_pin_lock.rs",
        "tools/support/release/pr_pin_tags.rs",
        "tools/support/release/pr_pin_retire_guard.rs",
        "tools/support/release/pr_pin_retire_source.rs",
        "tools/support/release/pr_pin_retire_evidence.rs",
    ]
}

pub(super) fn supports_cut(context: &Stable, cut: &str, root: &Path) -> Result<(), String> {
    // The frozen source must contain this protocol's actual compiled callers;
    // an old tag's workflow bytecode cannot inherit a new operator flag.
    for path in protocol_paths() {
        let observed = github::git(&["--no-replace-objects", "ls-tree", cut, "--", path], root)?;
        let expected = github::git(
            &[
                "--no-replace-objects",
                "ls-tree",
                &context.policy,
                "--",
                path,
            ],
            root,
        )?;
        if expected.is_empty() || observed != expected {
            return Err(format!(
                "Frozen stable C lacks the reviewed compiled protocol body/mode: {path}"
            ));
        }
    }
    Ok(())
}

pub(super) fn from_body(
    body: &str,
    cut: &str,
    repository: &str,
    root: &Path,
) -> Result<Option<Stable>, String> {
    body_with(body, &|policy| {
        github::git(&["fetch", "--no-tags", "origin", policy], root)?;
        let (context, _) = load(policy, cut, repository, root)?;
        supports_cut(&context, cut, root)?;
        Ok(context)
    })
}

fn body_with(
    body: &str,
    load: &dyn Fn(&str) -> Result<Stable, String>,
) -> Result<Option<Stable>, String> {
    if !body
        .lines()
        .any(|line| line.starts_with("<!-- vize-release-stable-"))
    {
        return Ok(None);
    }
    let policy = marker(body, "vize-release-stable-policy")?;
    let blob = marker(body, "vize-release-stable-plan")?;
    let sha256 = marker(body, "vize-release-stable-sha256")?;
    let context = load(&policy)?;
    if context.blob != blob || context.sha256 != sha256 {
        return Err("The immutable stable manifest changed".into());
    }
    let expected = markers(Some(&context));
    let observed: String = body
        .lines()
        .filter(|l| l.starts_with("<!-- vize-release-stable-"))
        .map(|l| format!("{l}\n"))
        .collect();
    if observed != expected {
        return Err("Unknown, reordered or duplicate stable authority markers".into());
    }
    Ok(Some(context))
}

pub(super) fn resume_authority(
    body: &str,
    tag: &str,
    cut: &str,
    root: &Path,
    authenticate: &dyn Fn() -> Result<Option<Stable>, String>,
) -> Result<Option<Stable>, String> {
    let context = authenticate()?;
    if let Some(context) = &context {
        if marker(body, "vize-release-bump")? != "patch" || target(context, root)? != tag {
            return Err("Stable source receipt is not the reviewed exact PATCH target".into());
        }
    } else {
        // Removing every stable marker must not turn a detached C into a main cut.
        first_parent(cut, &github::fetch_main(root)?, root)?;
    }
    Ok(context)
}

pub(super) fn markers(context: Option<&Stable>) -> String {
    context.map(|c| format!("<!-- vize-release-stable-policy: {} -->\n<!-- vize-release-stable-plan: {} -->\n<!-- vize-release-stable-sha256: {} -->\n", c.policy, c.blob, c.sha256)).unwrap_or_default()
}

pub(super) fn annotation(context: Option<&Stable>) -> String {
    context
        .map(|c| {
            format!(
                "\nStable-policy: {}\nStable-plan: {}\nStable-plan-sha256: {}",
                c.policy, c.blob, c.sha256
            )
        })
        .unwrap_or_default()
}

pub(super) fn main_anchor(source: &Source, main: &str, root: &Path) -> Result<(), String> {
    first_parent(operator_cut(source), main, root)
}

pub(super) fn operator_cut(source: &Source) -> &str {
    source
        .stable
        .as_ref()
        .map_or(source.cut.as_str(), |c| c.policy.as_str())
}

pub(super) fn shipping_catalog(source: &Source, root: &Path) -> Result<Value, String> {
    // Version fields legitimately differ; authenticate the complete official
    // C->H transformation, then derive H's full catalog without main's graph.
    metadata::verify_delta(
        &source.cut,
        &source.candidate.head,
        &source.base_version,
        source.candidate.tag.trim_start_matches('v'),
        root,
    )?;
    metadata::catalog(
        &source.candidate.head,
        source.candidate.tag.trim_start_matches('v'),
        root,
    )
}

pub(super) fn target(context: &Stable, root: &Path) -> Result<String, String> {
    let bytes = metadata::bytes(
        &["--no-replace-objects", "cat-file", "blob", &context.blob],
        root,
    )?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    Ok(pr_contract::field(&value, "/target")?.into())
}

#[cfg(test)]
mod manifest_tests {
    use super::*;
    #[test]
    fn whole_manifest_digest_refuses_duplicate_keys_at_every_depth() {
        let good = br#"{"a":1,"nested":{"b":2}}"#;
        assert_eq!(
            digest(good).unwrap(),
            "d9c57aebd2e08855d90bf8541f89942c6b2d9705fe7686f3d94baaaaad4d4a7c"
        );
        for bad in [br#"{"a":1,"a":2}"#.as_slice(), br#"{"a":{"b":1,"b":2}}"#] {
            assert!(digest(bad).is_err());
        }
    }

    #[test]
    fn literal_manifest_tree_record_refuses_executable_and_symlink_modes() {
        let repo = super::super::tests::Repo::new();
        let regular = repo.commit(&[(PLAN_PATH, "{\"schema\":\"whole\"}\n")]);
        let blob = manifest_blob(&regular, &repo.work).unwrap();
        for mode in ["100755", "120000"] {
            github::git(
                &[
                    "update-index",
                    "--cacheinfo",
                    &format!("{mode},{blob},{PLAN_PATH}"),
                ],
                &repo.work,
            )
            .unwrap();
            github::git(
                &["-c", "core.hooksPath=/dev/null", "commit", "-m", mode],
                &repo.work,
            )
            .unwrap();
            let changed = github::git(&["rev-parse", "HEAD"], &repo.work).unwrap();
            let error = manifest_blob(&changed, &repo.work).unwrap_err();
            assert!(error.contains("100644"), "{mode}: {error}");
        }
    }
}
