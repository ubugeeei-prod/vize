//! Validate immutable dependency preparation before a divergence comparison.

use regex::Regex;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

use vize_tool_support::common;

pub(super) fn read_and_validate_dependency_preparation(
    root: &Path,
    report_dir: &Path,
    project: &Value,
    summary: &Value,
) -> Result<Value, String> {
    let project_id = project_string(project, "id")?;
    let path = report_dir.join(format!("{project_id}-typecheck-dependencies.json"));
    let raw = common::read_text(&path).map_err(|_| {
        format!("Missing typecheck dependency preparation evidence for {project_id}")
    })?;
    let artifact = serde_json::from_str::<Value>(&raw).map_err(|error| {
        format!("Invalid typecheck dependency preparation JSON for {project_id}: {error}")
    })?;
    require_exact_keys(
        &artifact,
        &[
            "baselinePrepare",
            "evidence",
            "install",
            "lockfile",
            "packageManager",
            "project",
            "qualificationLockfile",
            "revision",
            "schema",
            "version",
        ],
        &format!("Typecheck dependency preparation keys are invalid for {project_id}"),
    )?;
    if artifact.get("schema").and_then(Value::as_str)
        != Some("vize.fixtureTypecheckDependencyInstall")
        || artifact.get("version").and_then(Value::as_u64) != Some(3)
        || artifact.get("project").and_then(Value::as_str) != Some(project_id.as_str())
        || artifact.get("revision") != project.get("revision")
        || artifact.pointer("/evidence/commitSha") != summary.pointer("/evidence/commitSha")
    {
        return Err(format!(
            "Typecheck dependency preparation identity is invalid for {project_id}"
        ));
    }
    let performance = project
        .get("typecheckPerformance")
        .ok_or_else(|| format!("{project_id} has no typecheckPerformance"))?;
    if artifact.pointer("/packageManager/name") != performance.get("packageManager")
        || artifact.pointer("/packageManager/version") != performance.get("packageManagerVersion")
    {
        return Err(format!(
            "Typecheck dependency package manager evidence is invalid for {project_id}"
        ));
    }
    let fixture_root = root.join(project_string(project, "fixturePath")?);
    let lockfile_path = artifact
        .pointer("/lockfile/path")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            format!("Typecheck dependency lockfile evidence is invalid for {project_id}")
        })?;
    if Some(lockfile_path) != performance.get("lockfile").and_then(Value::as_str) {
        return Err(format!(
            "Typecheck dependency lockfile evidence is invalid for {project_id}"
        ));
    }
    let lockfile = fs::read(fixture_root.join(lockfile_path)).map_err(|error| {
        format!("Typecheck dependency lockfile evidence is invalid for {project_id}: {error}")
    })?;
    if artifact
        .pointer("/lockfile/sizeBytes")
        .and_then(Value::as_u64)
        != Some(lockfile.len() as u64)
        || artifact.pointer("/lockfile/sha256").and_then(Value::as_str)
            != Some(sha256_bytes(&lockfile).as_str())
    {
        return Err(format!(
            "Typecheck dependency lockfile evidence is invalid for {project_id}"
        ));
    }
    let qualified_lockfile = qualify_lockfile_bytes(performance, &lockfile)?;
    let qualification = artifact
        .get("qualificationLockfile")
        .unwrap_or(&Value::Null);
    if qualified_lockfile == lockfile {
        if !qualification.is_null() {
            return Err(format!(
                "Unexpected typecheck qualification lockfile for {project_id}"
            ));
        }
    } else {
        require_exact_keys(
            qualification,
            &["mode", "sha256", "sizeBytes"],
            &format!("Typecheck qualification lockfile evidence is invalid for {project_id}"),
        )?;
        if qualification.get("mode").and_then(Value::as_str)
            != Some("pnpm-injected-workspace-packages")
            || qualification.get("sizeBytes").and_then(Value::as_u64)
                != Some(qualified_lockfile.len() as u64)
            || qualification.get("sha256").and_then(Value::as_str)
                != Some(sha256_bytes(&qualified_lockfile).as_str())
        {
            return Err(format!(
                "Typecheck qualification lockfile evidence is invalid for {project_id}"
            ));
        }
    }
    Ok(json!({
        "schema": "vize.fixtureTypecheckPreparationEvidence",
        "version": 1,
        "payloadSha256": sha256(raw.as_bytes()),
        "packageManager": artifact["packageManager"],
        "lockfile": artifact["lockfile"],
        "install": {
            "command": artifact.pointer("/install/command").cloned().unwrap_or(Value::Null),
            "exitCode": artifact.pointer("/install/exitCode").cloned().unwrap_or(Value::Null),
            "stdoutSha256": artifact.pointer("/install/stdoutSha256").cloned().unwrap_or(Value::Null),
            "stderrSha256": artifact.pointer("/install/stderrSha256").cloned().unwrap_or(Value::Null),
        },
        "baselinePrepare": artifact.get("baselinePrepare").and_then(|value| {
            if value.is_null() {
                Some(Value::Null)
            } else {
                Some(json!({
                    "command": value.get("command").cloned().unwrap_or(Value::Null),
                    "exitCode": value.get("exitCode").cloned().unwrap_or(Value::Null),
                    "stdoutSha256": value.get("stdoutSha256").cloned().unwrap_or(Value::Null),
                    "stderrSha256": value.get("stderrSha256").cloned().unwrap_or(Value::Null),
                }))
            }
        }).unwrap_or(Value::Null),
    }))
}

fn qualify_lockfile_bytes(performance: &Value, source: &[u8]) -> Result<Vec<u8>, String> {
    if performance.get("packageManager").and_then(Value::as_str) != Some("pnpm")
        || performance.get("workspacePackages").and_then(Value::as_str) != Some("injected")
    {
        return Ok(source.to_vec());
    }
    let source =
        std::str::from_utf8(source).map_err(|_| "pnpm lockfile must be valid UTF-8".to_string())?;
    let settings_pattern = Regex::new(r"(?m)^settings:\r?\n(?:[ \t][^\r\n]*(?:\r?\n|$))*")
        .map_err(|error| error.to_string())?;
    let settings_match = settings_pattern
        .find(source)
        .ok_or_else(|| "pnpm lockfile must contain a top-level settings block".to_string())?;
    let settings = settings_match.as_str();
    let configured_pattern = Regex::new(r"(?m)^[ \t]+injectWorkspacePackages: ([^\r\n]+)\r?$")
        .map_err(|error| error.to_string())?;
    let configured = configured_pattern.find_iter(settings).collect::<Vec<_>>();
    let valid_setting = configured.is_empty()
        || matches!(
            configured[0].as_str().trim_end_matches('\r'),
            "  injectWorkspacePackages: true" | "  injectWorkspacePackages: false"
        );
    if configured.len() > 1 || !valid_setting {
        return Err("pnpm lockfile has an invalid injectWorkspacePackages setting".to_string());
    }
    let qualified_settings = if let Some(setting) = configured.first() {
        if setting.as_str().trim_end_matches('\r') == "  injectWorkspacePackages: false" {
            let false_pattern = Regex::new(r"(?m)^  injectWorkspacePackages: false\r?$")
                .map_err(|error| error.to_string())?;
            false_pattern
                .replace(settings, "  injectWorkspacePackages: true")
                .into_owned()
        } else {
            settings.to_string()
        }
    } else {
        let eol = if settings.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        format!("{settings}  injectWorkspacePackages: true{eol}")
    };
    let mut qualified = String::with_capacity(
        source.len() + qualified_settings.len().saturating_sub(settings.len()),
    );
    qualified.push_str(&source[..settings_match.start()]);
    qualified.push_str(&qualified_settings);
    qualified.push_str(&source[settings_match.end()..]);
    Ok(qualified.into_bytes())
}

fn project_string(project: &Value, key: &str) -> Result<String, String> {
    project
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("project {key} must be a string"))
}

fn require_exact_keys(value: &Value, expected: &[&str], message: &str) -> Result<(), String> {
    let object = value.as_object().ok_or_else(|| message.to_string())?;
    let mut actual = object.keys().map(String::as_str).collect::<Vec<_>>();
    actual.sort();
    let mut expected = expected.to_vec();
    expected.sort();
    if actual != expected {
        return Err(message.to_string());
    }
    Ok(())
}

fn sha256(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

fn sha256_bytes(value: &[u8]) -> String {
    sha256(value)
}
