use super::*;

pub(super) fn verify_release_target() -> Result<ReleaseTarget, String> {
    let candidate = env::var("RELEASE_PR_NUMBER").is_ok_and(|value| !value.is_empty());
    let tag = env::var(if candidate {
        "RELEASE_TAG_NAME"
    } else {
        "GITHUB_REF_NAME"
    })
    .unwrap_or_default();
    let sha = env::var("GITHUB_SHA").unwrap_or_default();
    if !candidate && env::var("GITHUB_REF_TYPE").unwrap_or_default() != "tag" {
        return Err(format!(
            "Release preflight requires a tag event, got {}",
            env::var("GITHUB_REF_TYPE").unwrap_or_else(|_| "unknown".to_string())
        ));
    }
    let root = repo_root()?;
    let version = assert_release_metadata(
        &tag,
        &sha,
        &common::read_text(root.join("Cargo.toml"))?,
        &read_package_manifests(&root)?,
    )?;
    verify_git_release_target(&tag, &sha, &version)?;
    let base_sha = release_parent_sha(&sha)?;
    let version_only = release_changes_version_metadata_only(&base_sha, &sha);
    Ok(ReleaseTarget {
        tag,
        sha,
        version,
        base_sha,
        version_only,
    })
}

pub(super) fn read_package_manifests(root: &Path) -> Result<Vec<PackageManifest>, String> {
    let files = run_git(
        &[
            "ls-files",
            "-z",
            "--",
            RELEASE_PACKAGE_ROOTS[0],
            RELEASE_PACKAGE_ROOTS[1],
        ],
        &[0],
        root,
    )?
    .stdout;
    let mut manifests = Vec::new();
    for relative in files
        .split('\0')
        .filter(|path| path.ends_with("/package.json"))
    {
        let content = common::read_text(root.join(relative))?;
        let package_json: Value = serde_json::from_str(&content).map_err(|error| {
            format!("Failed to parse tracked package manifest {relative}: {error}")
        })?;
        if package_json.get("private").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        manifests.push(PackageManifest {
            path: relative.to_string(),
            content,
        });
    }
    manifests.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(manifests)
}

pub(super) fn assert_release_metadata(
    tag: &str,
    sha: &str,
    cargo_toml: &str,
    package_manifests: &[PackageManifest],
) -> Result<String, String> {
    let sha_re = Regex::new(r"^[0-9a-f]{40}$").unwrap();
    if !sha_re.is_match(sha) {
        return Err(format!("Release SHA must be a full commit SHA, got {sha}"));
    }
    parse_release_version(tag)?;
    let version = workspace_version_from_cargo_toml(cargo_toml)?;
    if tag != format!("v{version}") {
        return Err(format!(
            "Release tag {tag} does not match workspace version {version}"
        ));
    }
    let mut mismatches = Vec::new();
    for manifest in package_manifests {
        let package_json: Value = serde_json::from_str(&manifest.content).map_err(|error| {
            format!(
                "Failed to parse release package manifest {}: {error}",
                manifest.path
            )
        })?;
        if package_json.get("private").and_then(Value::as_bool) == Some(true) {
            mismatches.push(format!("{} is private", manifest.path));
            continue;
        }
        if package_json.get("version").and_then(Value::as_str) != Some(version.as_str()) {
            mismatches.push(format!(
                "{}={}",
                manifest.path,
                package_json
                    .get("version")
                    .map_or_else(|| "null".to_string(), value_to_string)
            ));
        }
    }
    if !mismatches.is_empty() {
        return Err(format!(
            "Release package versions must all equal {version}:\n{}",
            mismatches
                .iter()
                .map(|value| format!("- {value}"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    Ok(version)
}

pub(super) fn workspace_version_from_cargo_toml(content: &str) -> Result<String, String> {
    let mut in_workspace_package = false;
    let version_re = Regex::new(r#"^version\s*=\s*"([^"]+)"$"#).unwrap();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_workspace_package = trimmed == "[workspace.package]";
            continue;
        }
        if !in_workspace_package {
            continue;
        }
        if let Some(captures) = version_re.captures(trimmed) {
            return Ok(captures[1].to_string());
        }
    }
    Err("Cargo.toml is missing [workspace.package].version".to_string())
}

pub(super) fn verify_git_release_target(tag: &str, sha: &str, version: &str) -> Result<(), String> {
    let root = repo_root()?;
    if let Ok(number) = env::var("RELEASE_PR_NUMBER")
        && !number.is_empty()
    {
        let status = Command::new("rust-script")
            .args([
                "tools/commands/release/pr.rs",
                "validate",
                &number,
                sha,
                tag,
            ])
            .current_dir(&root)
            .status()
            .map_err(|error| error.to_string())?;
        return if status.success() {
            Ok(())
        } else {
            Err("Release PR validation failed".into())
        };
    }
    let head = run_git(&["rev-parse", "HEAD"], &[0], &root)?
        .stdout
        .trim()
        .to_string();
    if head != sha {
        return Err(format!(
            "Checked out HEAD {head} does not match release event SHA {sha}"
        ));
    }
    let main_sha = run_git(&["rev-parse", "refs/remotes/origin/main"], &[0], &root)?
        .stdout
        .trim()
        .to_string();
    let main_first_parent_history = run_git(
        &["rev-list", "--first-parent", "refs/remotes/origin/main"],
        &[0],
        &root,
    )?
    .stdout;
    assert_release_commit_is_on_main_first_parent(
        sha,
        &main_sha,
        main_first_parent_history
            .lines()
            .any(|line| line.trim() == sha),
    )?;
    let main_cargo = run_git(
        &["show", "refs/remotes/origin/main:Cargo.toml"],
        &[0],
        &root,
    )?
    .stdout;
    let main_version = workspace_version_from_cargo_toml(&main_cargo)?;
    assert_release_version_still_owns_main(tag, sha, &main_sha, version, &main_version)?;
    let remote = run_git(
        &[
            "ls-remote",
            "--exit-code",
            "--tags",
            "origin",
            &format!("refs/tags/{tag}"),
            &format!("refs/tags/{tag}^{{}}"),
        ],
        &[0],
        &root,
    )?
    .stdout;
    let target = remote_tag_commit(&remote, tag);
    if target.as_deref() != Some(sha) {
        let target_description = target.unwrap_or_else(|| "nothing".to_string());
        return Err(format!(
            "Remote tag {tag} points to {target_description}, expected {sha}"
        ));
    }
    Ok(())
}

pub(super) fn release_dispatch_ref(tag: &str) -> Result<String, String> {
    if env::var("RELEASE_PR_NUMBER").is_ok_and(|value| !value.is_empty()) {
        let branch = env::var("GITHUB_REF_NAME").unwrap_or_default();
        if branch != format!("release/{tag}") {
            return Err("Release dispatch must use its candidate branch".into());
        }
        Ok(branch)
    } else {
        Ok(tag.into())
    }
}
