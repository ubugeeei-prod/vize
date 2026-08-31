#![allow(dead_code)]

use crate::common;
use base64::Engine;
use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha512};
use std::{
    collections::BTreeMap,
    env, fs,
    path::Path,
    process::{Command, Stdio},
};

pub const BOOTSTRAP_PACKAGE_PATH: &str = "npm/framework/nuxt-lint-config";
pub const BOOTSTRAP_PACKAGE_NAME: &str = "@vizejs/nuxt-lint-config";
pub const BOOTSTRAP_ARTIFACT_NAME: &str = "release-package-nuxt-lint-config";
pub const REQUIRED_SUCCESSFUL_RELEASE_JOBS: &[&str] = &[
    "Build release npm packages",
    "Smoke release npm package installs",
    "release-preflight / Verify release safety contract",
];
pub const REQUIRED_FAILED_RELEASE_JOBS: &[&str] = &["Release @vizejs/nuxt-lint-config to npm"];
pub const REQUIRED_SKIPPED_RELEASE_JOBS: &[&str] =
    &["Release @vizejs/nuxt to npm", "Create GitHub Release"];

const REGISTRY_ORIGIN: &str = "https://registry.npmjs.org";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapRequest {
    pub artifact_name: String,
    pub package_name: String,
    pub package_path: String,
    pub release_run_id: String,
    pub tag_name: String,
    pub workflow_sha: String,
}

#[derive(Debug, Serialize)]
pub struct Handoff {
    #[serde(rename = "schemaVersion")]
    schema_version: u8,
    package: HandoffPackage,
    source: HandoffSource,
    #[serde(rename = "handoffArtifact")]
    handoff_artifact: String,
    tarball: HandoffTarball,
    publish: HandoffPublish,
}

#[derive(Debug, Serialize)]
struct HandoffPackage {
    name: String,
    version: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HandoffSource {
    artifact_name: String,
    release_run_id: String,
    tag_name: String,
    tag_sha: String,
}

#[derive(Debug, Serialize)]
struct HandoffTarball {
    file: String,
    integrity: String,
    sha512: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HandoffPublish {
    authentication: String,
    command: String,
    provenance: String,
    trust_command: String,
}

pub fn validate_bootstrap_request(
    tag_name: &str,
    package_path: &str,
    release_run_id: &str,
    workflow_ref: &str,
    workflow_sha: &str,
) -> Result<BootstrapRequest, String> {
    if workflow_ref != "refs/heads/main" {
        return Err(format!(
            "npm bootstrap must be dispatched from main, got {}",
            empty_label(workflow_ref)
        ));
    }
    if package_path != BOOTSTRAP_PACKAGE_PATH {
        return Err(format!(
            "Package path is not approved for npm bootstrap: {}",
            if package_path.is_empty() {
                "(empty)"
            } else {
                package_path
            }
        ));
    }
    if !release_tag_pattern().is_match(tag_name) {
        return Err(format!(
            "Release tag must be a strict v-prefixed SemVer, got {}",
            empty_label(tag_name)
        ));
    }
    if !positive_safe_integer(release_run_id) {
        return Err(format!(
            "Release run ID must be a positive safe integer, got {}",
            empty_label(release_run_id)
        ));
    }
    if !is_full_sha(workflow_sha) {
        return Err(format!(
            "GITHUB_SHA must be a full commit SHA, got {}",
            empty_label(workflow_sha)
        ));
    }
    Ok(BootstrapRequest {
        artifact_name: BOOTSTRAP_ARTIFACT_NAME.to_string(),
        package_name: BOOTSTRAP_PACKAGE_NAME.to_string(),
        package_path: package_path.to_string(),
        release_run_id: release_run_id.to_string(),
        tag_name: tag_name.to_string(),
        workflow_sha: workflow_sha.to_string(),
    })
}

pub fn validate_bootstrap_manifest(
    tag_name: &str,
    tag_sha: &str,
    package_path: &str,
    package_name: &str,
    cargo_toml: &str,
    package_manifest: &str,
) -> Result<String, String> {
    let version = assert_release_metadata(
        tag_name,
        tag_sha,
        cargo_toml,
        &[(
            format!("{package_path}/package.json"),
            package_manifest.to_string(),
        )],
    )?;
    let package_json = parse_json(package_manifest, &format!("{package_path}/package.json"))?;
    if package_json.get("name").and_then(Value::as_str) != Some(package_name) {
        return Err(format!(
            "{package_path}/package.json names {}, expected {package_name}",
            package_json
                .get("name")
                .map(Value::to_string)
                .unwrap_or_else(|| "undefined".to_string())
        ));
    }
    if package_json
        .get("publishConfig")
        .and_then(|config| config.get("access"))
        .and_then(Value::as_str)
        != Some("public")
    {
        return Err(format!(
            "{package_name} must declare publishConfig.access as public"
        ));
    }
    Ok(version)
}

pub fn validate_release_commit(
    tag_sha: &str,
    workflow_sha: &str,
    main_sha: &str,
    is_on_first_parent: bool,
) -> Result<(), String> {
    if !is_full_sha(tag_sha) || !is_full_sha(workflow_sha) || !is_full_sha(main_sha) {
        return Err("The release tag and origin/main must resolve to full commit SHAs".to_string());
    }
    if tag_sha != workflow_sha {
        return Err(format!(
            "Release tag commit {tag_sha} must exactly match repository dispatch SHA {workflow_sha}"
        ));
    }
    if !is_on_first_parent {
        return Err(format!(
            "Release commit {tag_sha} is not on the first-parent history of current origin/main {main_sha}"
        ));
    }
    Ok(())
}

pub fn validate_release_run(
    run: &Value,
    release_run_id: &str,
    repository: &str,
    tag_name: &str,
    tag_sha: &str,
) -> Result<(), String> {
    let expected = BTreeMap::from([
        ("conclusion", "failure".to_string()),
        ("event", "push".to_string()),
        ("head_branch", tag_name.to_string()),
        ("head_sha", tag_sha.to_string()),
        ("id", release_run_id.to_string()),
        ("name", "Release".to_string()),
        ("path", ".github/workflows/release.yml".to_string()),
        ("repository", repository.to_string()),
        ("status", "completed".to_string()),
    ]);
    let actual = BTreeMap::from([
        ("conclusion", value_string(run.get("conclusion"))),
        ("event", value_string(run.get("event"))),
        ("head_branch", value_string(run.get("head_branch"))),
        ("head_sha", value_string(run.get("head_sha"))),
        ("id", value_string(run.get("id"))),
        ("name", value_string(run.get("name"))),
        ("path", value_string(run.get("path"))),
        (
            "repository",
            value_string(
                run.get("head_repository")
                    .and_then(|repo| repo.get("full_name")),
            ),
        ),
        ("status", value_string(run.get("status"))),
    ]);
    let mismatches = expected
        .iter()
        .filter_map(|(key, expected)| {
            let actual = actual.get(key).cloned().unwrap_or_default();
            if &actual == expected {
                None
            } else {
                Some(format!("{key}={actual}"))
            }
        })
        .collect::<Vec<_>>();
    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Release run {release_run_id} does not match the failed exact-tag release contract: {}",
            mismatches.join(", ")
        ))
    }
}

pub fn validate_release_jobs(jobs: &[Value]) -> Result<(), String> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for job in jobs {
        let name = value_string(job.get("name"));
        *counts.entry(name.clone()).or_default() += 1;
        if job.get("status").and_then(Value::as_str) != Some("completed") {
            return Err(format!(
                "Every Release job must be terminal; {name} is {}",
                value_string(job.get("status"))
            ));
        }
    }
    let duplicates = counts
        .iter()
        .filter(|(_, count)| **count != 1)
        .map(|(name, count)| format!("{name}={count}"))
        .collect::<Vec<_>>();
    if !duplicates.is_empty() {
        return Err(format!(
            "Release job names must be unique: {}",
            duplicates.join(", ")
        ));
    }

    for name in REQUIRED_SUCCESSFUL_RELEASE_JOBS {
        validate_exact_job(jobs, name, "success")?;
    }
    for name in REQUIRED_FAILED_RELEASE_JOBS {
        validate_exact_job(jobs, name, "failure")?;
    }
    for name in REQUIRED_SKIPPED_RELEASE_JOBS {
        validate_exact_job(jobs, name, "skipped")?;
    }

    let allowed_non_success = REQUIRED_FAILED_RELEASE_JOBS
        .iter()
        .map(|name| (*name, "failure"))
        .chain(
            REQUIRED_SKIPPED_RELEASE_JOBS
                .iter()
                .map(|name| (*name, "skipped")),
        )
        .collect::<BTreeMap<_, _>>();
    for job in jobs {
        let name = value_string(job.get("name"));
        let expected = allowed_non_success
            .get(name.as_str())
            .copied()
            .unwrap_or("success");
        let conclusion = value_string(job.get("conclusion"));
        if conclusion != expected {
            return Err(format!(
                "Unexpected Release job conclusion: {name}={conclusion}, expected {expected}"
            ));
        }
    }
    Ok(())
}

pub fn validate_release_artifact(
    artifacts: &[Value],
    artifact_name: &str,
    release_run_id: &str,
    tag_name: &str,
    tag_sha: &str,
) -> Result<(), String> {
    let matches = artifacts
        .iter()
        .filter(|artifact| artifact.get("name").and_then(Value::as_str) == Some(artifact_name))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(format!(
            "Release run must contain exactly one {artifact_name} artifact, found {}",
            matches.len()
        ));
    }
    let artifact = matches[0];
    if artifact.get("expired").and_then(Value::as_bool) == Some(true) {
        return Err(format!("Release artifact {artifact_name} has expired"));
    }
    let source = artifact.get("workflow_run");
    if value_string(source.and_then(|source| source.get("id"))) != release_run_id
        || source
            .and_then(|source| source.get("head_branch"))
            .and_then(Value::as_str)
            != Some(tag_name)
        || source
            .and_then(|source| source.get("head_sha"))
            .and_then(Value::as_str)
            != Some(tag_sha)
    {
        return Err(format!(
            "Release artifact {artifact_name} is not bound to {tag_name} ({tag_sha})"
        ));
    }
    Ok(())
}

pub fn verify_release_run_evidence(
    api_url: &str,
    repository: &str,
    token: &str,
    release_run_id: &str,
    tag_name: &str,
    tag_sha: &str,
    artifact_name: &str,
) -> Result<(), String> {
    if repository != "ubugeeei-prod/vize" || token.is_empty() {
        return Err(
            "GITHUB_REPOSITORY must be ubugeeei-prod/vize and GITHUB_TOKEN is required".to_string(),
        );
    }
    let run = github_api_json(
        api_url,
        repository,
        token,
        &format!("actions/runs/{release_run_id}"),
    )?;
    validate_release_run(&run, release_run_id, repository, tag_name, tag_sha)?;
    let jobs = github_api_pages(
        api_url,
        repository,
        token,
        &format!("actions/runs/{release_run_id}/jobs"),
        Some("jobs"),
    )?;
    validate_release_jobs(&jobs)?;
    let artifacts = github_api_pages(
        api_url,
        repository,
        token,
        &format!("actions/runs/{release_run_id}/artifacts"),
        Some("artifacts"),
    )?;
    validate_release_artifact(&artifacts, artifact_name, release_run_id, tag_name, tag_sha)
}

pub fn validate_registry_response(package_name: &str, status: u16) -> Result<(), String> {
    match status {
        404 => Ok(()),
        200 => Err(format!(
            "{package_name} already exists on npm; bootstrap is only for first publish"
        )),
        status => Err(format!(
            "npm registry returned HTTP {status} while checking {package_name}"
        )),
    }
}

pub fn assert_package_is_unpublished(package_name: &str) -> Result<(), String> {
    let url = format!("{REGISTRY_ORIGIN}/{}", url_encode(package_name));
    let output = Command::new("curl")
        .args([
            "--silent",
            "--show-error",
            "--location",
            "--max-time",
            "30",
            "--output",
            "/dev/null",
            "--write-out",
            "%{http_code}",
            "--header",
            "accept: application/vnd.npm.install-v1+json",
            "--header",
            "user-agent: vize-npm-bootstrap (https://github.com/ubugeeei-prod/vize)",
        ])
        .arg(url)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to query npm registry: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "npm registry query failed\n{}{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        )
        .trim()
        .to_string());
    }
    let status = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u16>()
        .map_err(|error| format!("npm registry returned unreadable status: {error}"))?;
    validate_registry_response(package_name, status)
}

pub fn validate_downloaded_artifact(
    package_manifest: &str,
    expected_name: &str,
    expected_version: &str,
) -> Result<(), String> {
    let package_json =
        parse_json(package_manifest, "downloaded package.json").map_err(|error| {
            error.replace(
                "Invalid release evidence JSON",
                "Downloaded Release artifact has an invalid package.json",
            )
        })?;
    if package_json.get("name").and_then(Value::as_str) != Some(expected_name)
        || package_json.get("version").and_then(Value::as_str) != Some(expected_version)
    {
        return Err(format!(
            "Downloaded Release artifact is {}@{}, expected {expected_name}@{expected_version}",
            package_json
                .get("name")
                .map(Value::to_string)
                .unwrap_or_else(|| "undefined".to_string()),
            package_json
                .get("version")
                .map(Value::to_string)
                .unwrap_or_else(|| "undefined".to_string())
        ));
    }
    Ok(())
}

pub fn cli_handoff_names(package_name: &str, version: &str) -> Result<(String, String), String> {
    let package = Regex::new(r"^@([a-z0-9][a-z0-9._-]*)/([a-z0-9][a-z0-9._-]*)$").unwrap();
    let Some(captures) = package.captures(package_name) else {
        return Err(format!(
            "CLI handoff requires a lowercase scoped npm package, got {package_name}"
        ));
    };
    require_single_line("package version", version, &strict_version_pattern())?;
    let slug = format!("{}-{}", &captures[1], &captures[2]);
    Ok((
        format!("npm-cli-first-publish-{slug}-{version}"),
        format!("{slug}-{version}.tgz"),
    ))
}

pub fn create_cli_publish_handoff(
    package_path: &Path,
    output_path: &Path,
    expected_name: &str,
    expected_version: &str,
    source_artifact_name: &str,
    release_run_id: &str,
    release_tag_name: &str,
    release_tag_sha: &str,
    npm_bin: &str,
) -> Result<Handoff, String> {
    require_single_line(
        "Release run ID",
        release_run_id,
        &Regex::new(r"^[1-9]\d*$").unwrap(),
    )?;
    require_single_line(
        "Release tag SHA",
        release_tag_sha,
        &Regex::new(r"^[0-9a-f]{40}$").unwrap(),
    )?;
    require_single_line(
        "Release artifact name",
        source_artifact_name,
        &Regex::new(r"^release-package-[a-z0-9-]+$").unwrap(),
    )?;
    if release_tag_name != format!("v{expected_version}") {
        return Err(format!(
            "Release tag {} must match package version {expected_version}",
            empty_label(release_tag_name)
        ));
    }

    let package_manifest = fs::read_to_string(package_path.join("package.json"))
        .map_err(|error| format!("cannot read package.json: {error}"))?;
    validate_downloaded_artifact(&package_manifest, expected_name, expected_version)?;
    let package_json = parse_json(&package_manifest, "package.json")?;
    if package_json
        .get("publishConfig")
        .and_then(|config| config.get("access"))
        .and_then(Value::as_str)
        != Some("public")
    {
        return Err(format!(
            "{expected_name} must declare publishConfig.access as public"
        ));
    }

    let (artifact_name, tarball_name) = cli_handoff_names(expected_name, expected_version)?;
    fs::create_dir(output_path)
        .map_err(|error| format!("cannot create {}: {error}", output_path.display()))?;
    let temp_root = tempfile::Builder::new()
        .prefix("vize-npm-cli-handoff-")
        .tempdir()
        .map_err(|error| error.to_string())?;
    let first_destination = temp_root.path().join("first");
    let second_destination = temp_root.path().join("second");
    fs::create_dir(&first_destination).map_err(|error| error.to_string())?;
    fs::create_dir(&second_destination).map_err(|error| error.to_string())?;
    let first = pack_once(
        npm_bin,
        package_path,
        &first_destination,
        expected_name,
        expected_version,
        &tarball_name,
    )?;
    let second = pack_once(
        npm_bin,
        package_path,
        &second_destination,
        expected_name,
        expected_version,
        &tarball_name,
    )?;
    if first.contents != second.contents || first.integrity != second.integrity {
        return Err(format!(
            "npm pack did not reproduce {tarball_name} byte-for-byte"
        ));
    }

    fs::write(output_path.join(&tarball_name), &first.contents)
        .map_err(|error| format!("cannot write handoff tarball: {error}"))?;
    fs::write(
        output_path.join("SHA512SUMS"),
        format!("{}  {tarball_name}\n", first.sha512),
    )
    .map_err(|error| format!("cannot write SHA512SUMS: {error}"))?;

    let handoff = Handoff {
        schema_version: 1,
        package: HandoffPackage {
            name: expected_name.to_string(),
            version: expected_version.to_string(),
        },
        source: HandoffSource {
            artifact_name: source_artifact_name.to_string(),
            release_run_id: release_run_id.to_string(),
            tag_name: release_tag_name.to_string(),
            tag_sha: release_tag_sha.to_string(),
        },
        handoff_artifact: artifact_name,
        tarball: HandoffTarball {
            file: tarball_name,
            integrity: first.integrity,
            sha512: first.sha512,
        },
        publish: HandoffPublish {
            authentication: "interactive npm CLI owner session with 2FA".to_string(),
            command: format!("npm publish ./{} --access public", first.tarball_name),
            provenance: "none: local CLI first publish is outside GitHub Actions OIDC".to_string(),
            trust_command: format!(
                "npm trust github {expected_name} --file release.yml --repo ubugeeei-prod/vize --env npm --allow-publish --yes"
            ),
        },
    };
    common::write_json_pretty(output_path.join("npm-publish-handoff.json"), &handoff)?;
    Ok(handoff)
}

pub fn format_cli_handoff_summary(handoff: &Handoff) -> String {
    format!(
        r#"## npm CLI first-publish handoff

- Package: `{}@{}`
- Handoff artifact: `{}`
- Tarball: `{}`
- SHA-512: `{}`
- Source: `{}` / Release run `{}` / `{}`

Download the handoff artifact, verify `SHA512SUMS`, authenticate with the npm CLI as an owner with 2FA, then run:

```bash
{}
```

This workflow did not request an OIDC token and did not publish. The one-time local CLI publish does not carry GitHub Actions OIDC provenance. Immediately configure `release.yml` as the trusted publisher with:

```bash
{}
```

"#,
        handoff.package.name,
        handoff.package.version,
        handoff.handoff_artifact,
        handoff.tarball.file,
        handoff.tarball.sha512,
        handoff.source.tag_name,
        handoff.source.release_run_id,
        handoff.source.artifact_name,
        handoff.publish.command,
        handoff.publish.trust_command
    )
}

pub fn handoff_artifact_name(handoff: &Handoff) -> &str {
    &handoff.handoff_artifact
}

pub fn handoff_tarball_file(handoff: &Handoff) -> &str {
    &handoff.tarball.file
}

pub fn handoff_sha512(handoff: &Handoff) -> &str {
    &handoff.tarball.sha512
}

struct PackResult {
    contents: Vec<u8>,
    integrity: String,
    sha512: String,
    tarball_name: String,
}

fn pack_once(
    npm_bin: &str,
    package_path: &Path,
    destination: &Path,
    expected_name: &str,
    expected_version: &str,
    tarball_name: &str,
) -> Result<PackResult, String> {
    let metadata = run_npm_pack(npm_bin, package_path, destination)?;
    if metadata.get("name").and_then(Value::as_str) != Some(expected_name)
        || metadata.get("version").and_then(Value::as_str) != Some(expected_version)
        || metadata.get("filename").and_then(Value::as_str) != Some(tarball_name)
    {
        return Err(format!(
            "npm pack produced {}@{} in {}, expected {expected_name}@{expected_version} in {tarball_name}",
            value_string(metadata.get("name")),
            value_string(metadata.get("version")),
            value_string(metadata.get("filename"))
        ));
    }
    let tarball_path = destination.join(tarball_name);
    let meta = fs::symlink_metadata(&tarball_path)
        .map_err(|error| format!("cannot stat {}: {error}", tarball_path.display()))?;
    if !meta.is_file() || meta.file_type().is_symlink() {
        return Err(format!(
            "npm pack output must be a regular file: {}",
            tarball_path.display()
        ));
    }
    let contents = fs::read(&tarball_path)
        .map_err(|error| format!("cannot read {}: {error}", tarball_path.display()))?;
    let digest = Sha512::digest(&contents);
    let sha512_hex = format!("{digest:x}");
    let integrity = format!(
        "sha512-{}",
        base64::engine::general_purpose::STANDARD.encode(digest)
    );
    if metadata.get("integrity").and_then(Value::as_str) != Some(&integrity) {
        return Err(format!("npm pack integrity mismatch for {tarball_name}"));
    }
    Ok(PackResult {
        contents,
        integrity,
        sha512: sha512_hex,
        tarball_name: tarball_name.to_string(),
    })
}

fn run_npm_pack(npm_bin: &str, package_path: &Path, destination: &Path) -> Result<Value, String> {
    let output = Command::new(npm_bin)
        .args(["pack", "--ignore-scripts", "--json", "--pack-destination"])
        .arg(destination)
        .current_dir(package_path)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run npm pack: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "npm pack failed with exit {}\n{}{}",
            output.status.code().unwrap_or(1),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .trim()
        .to_string());
    }
    let entries = parse_json(&String::from_utf8_lossy(&output.stdout), "npm pack")?;
    let entries = entries
        .as_array()
        .ok_or_else(|| format!("npm pack must describe exactly one tarball, got {entries}"))?;
    if entries.len() != 1 {
        return Err(format!(
            "npm pack must describe exactly one tarball, got {}",
            Value::Array(entries.clone())
        ));
    }
    Ok(entries[0].clone())
}

pub fn assert_release_metadata(
    tag: &str,
    sha: &str,
    cargo_toml: &str,
    package_manifests: &[(String, String)],
) -> Result<String, String> {
    if !is_full_sha(sha) {
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
    for (path, content) in package_manifests {
        let package_json = parse_json(content, path)?;
        if package_json.get("private").and_then(Value::as_bool) == Some(true) {
            mismatches.push(format!("{path} is private"));
            continue;
        }
        if package_json.get("version").and_then(Value::as_str) != Some(&version) {
            mismatches.push(format!(
                "{path}={}",
                package_json
                    .get("version")
                    .map(Value::to_string)
                    .unwrap_or_else(|| "undefined".to_string())
            ));
        }
    }
    if mismatches.is_empty() {
        Ok(version)
    } else {
        Err(format!(
            "Release package versions must all equal {version}:\n{}",
            mismatches
                .iter()
                .map(|value| format!("- {value}"))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

pub fn workspace_version_from_cargo_toml(content: &str) -> Result<String, String> {
    let mut in_workspace_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_workspace_package = trimmed == "[workspace.package]";
            continue;
        }
        if !in_workspace_package {
            continue;
        }
        if let Some(version) = trimmed
            .strip_prefix("version")
            .and_then(|rest| rest.trim_start().strip_prefix('='))
            .map(str::trim)
            .and_then(|value| value.strip_prefix('"'))
            .and_then(|value| value.strip_suffix('"'))
        {
            return Ok(version.to_string());
        }
    }
    Err("Cargo.toml is missing [workspace.package].version".to_string())
}

fn validate_exact_job(jobs: &[Value], name: &str, conclusion: &str) -> Result<(), String> {
    let matches = jobs
        .iter()
        .filter(|job| job.get("name").and_then(Value::as_str) == Some(name))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(format!(
            "Release run must contain exactly one {name} job, found {}",
            matches.len()
        ));
    }
    let job = matches[0];
    if job.get("status").and_then(Value::as_str) != Some("completed")
        || job.get("conclusion").and_then(Value::as_str) != Some(conclusion)
    {
        return Err(format!(
            "{name} must be completed/{conclusion}, got {}/{}",
            value_string(job.get("status")),
            value_string(job.get("conclusion"))
        ));
    }
    Ok(())
}

fn github_api_json(
    api_url: &str,
    repository: &str,
    token: &str,
    resource: &str,
) -> Result<Value, String> {
    let body = github_api_get(api_url, repository, token, resource)?;
    parse_json(&body, resource)
}

fn github_api_pages(
    api_url: &str,
    repository: &str,
    token: &str,
    resource: &str,
    collection: Option<&str>,
) -> Result<Vec<Value>, String> {
    let mut values = Vec::new();
    for page in 1.. {
        let separator = if resource.contains('?') { '&' } else { '?' };
        let body = github_api_get(
            api_url,
            repository,
            token,
            &format!("{resource}{separator}per_page=100&page={page}"),
        )?;
        let payload = parse_json(&body, resource)?;
        let page_values = if let Some(collection) = collection {
            payload
                .get(collection)
                .and_then(Value::as_array)
                .cloned()
                .ok_or_else(|| format!("GitHub API {resource} did not return {collection}"))?
        } else {
            payload
                .as_array()
                .cloned()
                .ok_or_else(|| format!("GitHub API {resource} did not return an array"))?
        };
        let count = page_values.len();
        values.extend(page_values);
        if count < 100 {
            return Ok(values);
        }
    }
    unreachable!()
}

fn github_api_get(
    api_url: &str,
    repository: &str,
    token: &str,
    resource: &str,
) -> Result<String, String> {
    let url = format!(
        "{}/repos/{}/{}",
        api_url.trim_end_matches('/'),
        repository,
        resource
    );
    let output = Command::new("curl")
        .args([
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--max-time",
            "30",
            "--header",
            "accept: application/vnd.github+json",
            "--header",
            &format!("authorization: Bearer {token}"),
            "--header",
            "content-type: application/json",
            "--header",
            "x-github-api-version: 2022-11-28",
        ])
        .arg(url)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to query GitHub API: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "GitHub API GET /repos/{repository}/{resource} failed\n{}{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        )
        .trim()
        .to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn parse_release_version(tag: &str) -> Result<(), String> {
    if release_tag_pattern().is_match(tag) {
        Ok(())
    } else {
        Err(format!(
            "Release tag must be a strict v-prefixed SemVer, got {}",
            empty_label(tag)
        ))
    }
}

fn release_tag_pattern() -> Regex {
    Regex::new(r"^v(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-(?:(?:0|[1-9]\d*)|(?:\d*[A-Za-z-][0-9A-Za-z-]*))(?:\.(?:(?:0|[1-9]\d*)|(?:\d*[A-Za-z-][0-9A-Za-z-]*)))*)?$").unwrap()
}

fn strict_version_pattern() -> Regex {
    Regex::new(r"^(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-(?:(?:0|[1-9]\d*)|(?:\d*[A-Za-z-][0-9A-Za-z-]*))(?:\.(?:(?:0|[1-9]\d*)|(?:\d*[A-Za-z-][0-9A-Za-z-]*)))*)?$").unwrap()
}

fn require_single_line<'a>(
    label: &str,
    value: &'a str,
    pattern: &Regex,
) -> Result<&'a str, String> {
    if pattern.is_match(value) {
        Ok(value)
    } else {
        Err(format!("Invalid {label}: {}", empty_label(value)))
    }
}

fn is_full_sha(value: &str) -> bool {
    Regex::new(r"^[0-9a-f]{40}$").unwrap().is_match(value)
}

fn positive_safe_integer(value: &str) -> bool {
    let Ok(parsed) = value.parse::<u64>() else {
        return false;
    };
    parsed > 0 && parsed <= 9_007_199_254_740_991 && parsed.to_string() == value
}

fn empty_label(value: &str) -> &str {
    if value.is_empty() { "(empty)" } else { value }
}

fn value_string(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        Some(Value::Null) | None => String::new(),
        Some(value) => value.to_string(),
    }
}

fn parse_json(text: &str, name: &str) -> Result<Value, String> {
    serde_json::from_str(text)
        .map_err(|error| format!("Invalid release evidence JSON {name}: {error}"))
}

fn url_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            byte => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

pub fn run_git(args: &[&str], accepted: &[i32]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;
    let status = output.status.code().unwrap_or(1);
    if !accepted.contains(&status) {
        return Err(format!(
            "git {} failed with exit {status}\n{}{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .trim()
        .to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn read_tagged_file(tag_sha: &str, file_path: &str) -> Result<String, String> {
    run_git(&["show", &format!("{tag_sha}:{file_path}")], &[0])
}

pub fn run_preflight(env: &BTreeMap<String, String>) -> Result<(), String> {
    let request = validate_bootstrap_request(
        env.get("RELEASE_TAG_NAME")
            .map(String::as_str)
            .unwrap_or(""),
        env.get("BOOTSTRAP_PACKAGE_PATH")
            .map(String::as_str)
            .unwrap_or(""),
        env.get("RELEASE_RUN_ID").map(String::as_str).unwrap_or(""),
        env.get("GITHUB_REF").map(String::as_str).unwrap_or(""),
        env.get("GITHUB_SHA").map(String::as_str).unwrap_or(""),
    )?;
    run_git(
        &[
            "fetch",
            "--quiet",
            "--no-tags",
            "origin",
            "+refs/heads/main:refs/remotes/origin/main",
        ],
        &[0],
    )?;
    run_git(
        &[
            "fetch",
            "--quiet",
            "--no-tags",
            "origin",
            &format!(
                "+refs/tags/{}:refs/tags/{}",
                request.tag_name, request.tag_name
            ),
        ],
        &[0],
    )?;
    let tag_sha = run_git(
        &[
            "rev-parse",
            &format!("refs/tags/{}^{{commit}}", request.tag_name),
        ],
        &[0],
    )?
    .trim()
    .to_string();
    let main_sha = run_git(&["rev-parse", "refs/remotes/origin/main"], &[0])?
        .trim()
        .to_string();
    let main_history = run_git(
        &["rev-list", "--first-parent", "refs/remotes/origin/main"],
        &[0],
    )?;
    validate_release_commit(
        &tag_sha,
        &request.workflow_sha,
        &main_sha,
        main_history.lines().any(|line| line == tag_sha),
    )?;
    let package_manifest =
        read_tagged_file(&tag_sha, &format!("{}/package.json", request.package_path))?;
    let version = validate_bootstrap_manifest(
        &request.tag_name,
        &tag_sha,
        &request.package_path,
        &request.package_name,
        &read_tagged_file(&tag_sha, "Cargo.toml")?,
        &package_manifest,
    )?;
    verify_release_run_evidence(
        env.get("GITHUB_API_URL")
            .map(String::as_str)
            .unwrap_or("https://api.github.com"),
        env.get("GITHUB_REPOSITORY")
            .map(String::as_str)
            .unwrap_or(""),
        env.get("GITHUB_TOKEN").map(String::as_str).unwrap_or(""),
        &request.release_run_id,
        &request.tag_name,
        &tag_sha,
        &request.artifact_name,
    )?;
    assert_package_is_unpublished(&request.package_name)?;
    let output = env
        .get("GITHUB_OUTPUT")
        .ok_or_else(|| "GITHUB_OUTPUT is required for npm bootstrap preflight".to_string())?;
    common::append_text(
        output,
        &format!(
            "artifact_name={}\npackage_name={}\npackage_path={}\nrelease_run_id={}\ntag_sha={tag_sha}\nversion={version}\n",
            request.artifact_name,
            request.package_name,
            request.package_path,
            request.release_run_id
        ),
    )?;
    println!(
        "npm bootstrap preflight passed for {}@{} from {} ({}).",
        request.package_name, version, request.tag_name, tag_sha
    );
    Ok(())
}

pub fn run_registry_recheck(env: &BTreeMap<String, String>) -> Result<(), String> {
    let request = validate_bootstrap_request(
        env.get("RELEASE_TAG_NAME")
            .map(String::as_str)
            .unwrap_or(""),
        env.get("BOOTSTRAP_PACKAGE_PATH")
            .map(String::as_str)
            .unwrap_or(""),
        env.get("RELEASE_RUN_ID").map(String::as_str).unwrap_or(""),
        env.get("GITHUB_REF").map(String::as_str).unwrap_or(""),
        env.get("GITHUB_SHA").map(String::as_str).unwrap_or(""),
    )?;
    assert_package_is_unpublished(&request.package_name)?;
    println!(
        "{} is still absent from npm immediately before first publish.",
        request.package_name
    );
    Ok(())
}

pub fn run_artifact_validation(env: &BTreeMap<String, String>) -> Result<(), String> {
    let artifact_path = env
        .get("BOOTSTRAP_ARTIFACT_PATH")
        .map(String::as_str)
        .unwrap_or("");
    if artifact_path != "bootstrap-package" {
        return Err(format!(
            "Bootstrap artifact path must be bootstrap-package, got {}",
            empty_label(artifact_path)
        ));
    }
    let expected_name = env
        .get("EXPECTED_PACKAGE_NAME")
        .map(String::as_str)
        .unwrap_or("");
    let expected_version = env
        .get("EXPECTED_PACKAGE_VERSION")
        .map(String::as_str)
        .unwrap_or("");
    validate_downloaded_artifact(
        &fs::read_to_string(Path::new(artifact_path).join("package.json"))
            .map_err(|error| format!("cannot read bootstrap package manifest: {error}"))?,
        expected_name,
        expected_version,
    )?;
    println!("Validated downloaded Release artifact {expected_name}@{expected_version}.");
    Ok(())
}

pub fn env_map() -> BTreeMap<String, String> {
    env::vars().collect()
}
