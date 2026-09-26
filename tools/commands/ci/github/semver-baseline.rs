#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! semver = "1"
//! serde_json = "1"
//! sha2 = "0.10"
//! toml_edit = "0.25"
//!
//! [package]
//! edition = "2024"
//! ```

// First publication under a new package name still checks the old published API.
// These immutable archives retain their Rust source, versions, and dependencies.
// Only the package and library names are adapted for baseline package selection.
use semver::Version;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::Path,
    process::{Command, ExitCode},
};
use toml_edit::{DocumentMut, value};

#[path = "../../../support/release/semver_git_baseline.rs"]
mod git_baseline;

const OLD_VERSION: &str = "0.428.1";
const RENAMES: &[(&str, &str, &str)] = &[
    (
        "vize_l1",
        "vize_s1",
        "f28076ad1b5a7adbe4439a57e158a99584dcd4e0d980e0f8b70c8650ce5e243b",
    ),
    (
        "vize_l2",
        "vize_s2",
        "5fc5c0a5d6f0118ab070c76963a8be9e17e14ff5895f727d8fe0b9c428e3c18a",
    ),
    (
        "vize_l1_to_l2",
        "vize_s1_to_s2",
        "92e2f8378bb827ae67f6471e08a6ac8aef40e6dfacdab5044449b1637031d241",
    ),
    (
        "vize_l2_to_l3",
        "vize_s2_to_s3",
        "02aa69e3f234592083045ad076cf33f681c84505ccc571da452f6bcaf4227f45",
    ),
];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("SemVer baseline: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let [package, destination, base @ ..] = args.as_slice() else {
        return Err("Usage: rust-script tools/commands/ci/github/semver-baseline.rs <package> <empty-directory> [actual-base-revision]".into());
    };
    if base.len() > 1 {
        return Err("expected at most one actual base revision".into());
    }
    let Some((_, old_name, checksum)) = RENAMES.iter().find(|(name, _, _)| name == package) else {
        return Ok(()); // Existing names keep cargo-semver-checks' registry baseline.
    };
    let current_manifest =
        fs::read_to_string(format!("crates/{package}/Cargo.toml")).map_err(|e| e.to_string())?;
    let current: DocumentMut = current_manifest
        .parse()
        .map_err(|e: toml_edit::TomlError| e.to_string())?;
    if current["package"]["name"].as_str() != Some(package) {
        return Err("current package identity mismatch".into());
    }
    let workspace: DocumentMut = fs::read_to_string("Cargo.toml")
        .map_err(|e| e.to_string())?
        .parse()
        .map_err(|e: toml_edit::TomlError| e.to_string())?;
    let version = current["package"]["version"]
        .as_str()
        .or_else(|| workspace["workspace"]["package"]["version"].as_str())
        .ok_or("current package version is missing")?;
    let version = Version::parse(version).map_err(|e| e.to_string())?;
    let destination = Path::new(destination);
    if !destination.is_dir()
        || fs::read_dir(destination)
            .map_err(|e| e.to_string())?
            .next()
            .is_some()
    {
        return Err("baseline destination must be an empty directory".into());
    }
    if let Some(base) = base.first() {
        if let Some(root) = git_baseline::prepare(package, old_name, destination, base)? {
            println!("{root}");
        }
        return Ok(());
    }
    let index_path = destination.join("index.jsonl");
    let status = download(
        &format!("https://index.crates.io/vi/ze/{package}"),
        &index_path,
    )?;
    if status == 200
        && has_registry_baseline(
            &fs::read_to_string(&index_path).map_err(|e| e.to_string())?,
            package,
            &version,
        )?
    {
        return Ok(());
    }
    if status != 200 && status != 404 {
        return Err(format!("registry index returned HTTP {status}"));
    }
    if version < Version::parse(OLD_VERSION).unwrap() {
        return Err(format!(
            "current version predates the immutable {OLD_VERSION} baseline"
        ));
    }
    let archive = destination.join("baseline.crate");
    let status = download(
        &format!("https://static.crates.io/crates/{old_name}/{old_name}-{OLD_VERSION}.crate"),
        &archive,
    )?;
    if status != 200 {
        return Err(format!("old published archive returned HTTP {status}"));
    }
    verify_checksum(&fs::read(&archive).map_err(|e| e.to_string())?, checksum)?;
    let result = Command::new("tar")
        .args(["-xzf"])
        .arg(&archive)
        .arg("-C")
        .arg(destination)
        .status()
        .map_err(|e| e.to_string())?;
    if !result.success() {
        return Err("cannot extract immutable published archive".into());
    }
    let root = destination.join(format!("{old_name}-{OLD_VERSION}"));
    let manifest_path = root.join("Cargo.toml");
    let source = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    fs::write(&manifest_path, adapt_manifest(&source, old_name, package)?)
        .map_err(|e| e.to_string())?;
    eprintln!(
        "SemVer baseline: {old_name}@{OLD_VERSION}, SHA256 {checksum}; package/lib name -> {package}, source and version unchanged"
    );
    println!("{}", root.display());
    Ok(())
}

fn download(url: &str, destination: &Path) -> Result<u16, String> {
    let output = Command::new("curl")
        .args([
            "--silent",
            "--show-error",
            "--location",
            "--fail-with-body",
            "--connect-timeout",
            "5",
            "--max-time",
            "15",
            "--retry",
            "2",
            "--retry-delay",
            "1",
            "--output",
        ])
        .arg(destination)
        .args(["--write-out", "%{http_code}", url])
        .output()
        .map_err(|e| e.to_string())?;
    let status = String::from_utf8_lossy(&output.stdout)
        .parse::<u16>()
        .map_err(|_| "invalid registry HTTP status")?;
    if !output.status.success() && !(output.status.code() == Some(22) && status == 404) {
        return Err(format!(
            "registry request failed (HTTP {status}): {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(status)
}

fn has_registry_baseline(index: &str, package: &str, current: &Version) -> Result<bool, String> {
    let mut found = false;
    let mut rows = 0;
    for line in index.lines() {
        let row: Value =
            serde_json::from_str(line).map_err(|e| format!("invalid registry index: {e}"))?;
        if row["name"].as_str() != Some(package) || !row["yanked"].is_boolean() {
            return Err("registry index package identity/schema mismatch".into());
        }
        let version = row["vers"].as_str().ok_or("registry version is missing")?;
        let version = Version::parse(version).map_err(|e| e.to_string())?;
        found |= version <= *current;
        rows += 1;
    }
    if rows == 0 {
        return Err("registry index is empty".into());
    }
    Ok(found)
}

fn verify_checksum(bytes: &[u8], expected: &str) -> Result<(), String> {
    if format!("{:x}", Sha256::digest(bytes)) != expected {
        return Err("immutable published archive checksum mismatch".into());
    }
    Ok(())
}

fn adapt_manifest(source: &str, old_name: &str, new_name: &str) -> Result<String, String> {
    let mut manifest: DocumentMut = source
        .parse()
        .map_err(|e: toml_edit::TomlError| e.to_string())?;
    if manifest["package"]["name"].as_str() != Some(old_name)
        || manifest["package"]["version"].as_str() != Some(OLD_VERSION)
        || manifest["lib"]["name"].as_str() != Some(old_name)
    {
        return Err("immutable baseline package/lib/version identity mismatch".into());
    }
    manifest["package"]["name"] = value(new_name);
    manifest["lib"]["name"] = value(new_name);
    Ok(manifest.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn published_identity_adapter_keeps_version_source_path_and_dependency_aliases() {
        let source = "[package]\nname = \"vize_s1_to_s2\"\nversion = \"0.428.1\"\n[lib]\nname = \"vize_s1_to_s2\"\npath = \"src/lib.rs\"\n[dependencies.vize_s0]\npackage = \"vize_carton\"\nversion = \"=0.428.1\"\n";
        let adapted = adapt_manifest(source, "vize_s1_to_s2", "vize_l1_to_l2").unwrap();
        assert_eq!(adapted.replace("vize_l1_to_l2", "vize_s1_to_s2"), source);
        assert!(
            adapt_manifest(
                &source.replace("0.428.1", "0.429.0"),
                "vize_s1_to_s2",
                "vize_l1_to_l2"
            )
            .is_err()
        );
        assert!(adapt_manifest(source, "other", "vize_l1_to_l2").is_err());
    }

    #[test]
    fn registry_schema_errors_do_not_trigger_old_archive_fallback() {
        let current = Version::parse("0.429.0").unwrap();
        assert!(has_registry_baseline("", "vize_l1", &current).is_err());
        assert!(has_registry_baseline("not-json", "vize_l1", &current).is_err());
        assert!(
            has_registry_baseline(
                r#"{"name":"wrong","vers":"0.429.0","yanked":false}"#,
                "vize_l1",
                &current
            )
            .is_err()
        );
        assert!(
            has_registry_baseline(
                r#"{"name":"vize_l1","vers":"0.429.0"}"#,
                "vize_l1",
                &current
            )
            .is_err()
        );
        assert!(
            has_registry_baseline(
                r#"{"name":"vize_l1","vers":"0.429.0","yanked":false}"#,
                "vize_l1",
                &current
            )
            .unwrap()
        );
        assert!(
            !has_registry_baseline(
                r#"{"name":"vize_l1","vers":"0.430.0","yanked":false}"#,
                "vize_l1",
                &current
            )
            .unwrap()
        );
    }

    #[test]
    fn corrupted_archive_cannot_be_used_as_baseline() {
        assert!(verify_checksum(b"changed API", RENAMES[2].2).is_err());
        assert!(
            verify_checksum(
                b"",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            )
            .is_ok()
        );
    }
}
