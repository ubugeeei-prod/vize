//! Unchanged generated source receipt bodies.
use super::super::super::pr_github as github;
use super::super::Source;
use std::{fs, path::Path};

pub(super) fn source_body(
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

pub(super) fn set_body(source: &Source, bump: &str, root: &Path) -> Result<(), String> {
    let body = source_body(
        &source.candidate.tag,
        bump,
        &source.base_version,
        &source.cut,
        &source.candidate.head,
        Some(source.integration),
    ) + &super::super::stable::markers(source.stable.as_ref());
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
