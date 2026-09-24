//! External tooling used to obtain registries: `npm pack`, `npm view`,
//! `tar`, and `curl`. `vize lib` has no HTTP client of its own.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;

use vize_s0::{String, cstr};

use super::error::{LibError, LibResult};
use super::registry::{LoadedRegistry, RemoteFiles};

/// Environment override for the npm executable (mirrors, tests).
pub const NPM_ENV: &str = "VIZE_LIB_NPM";
/// Environment override for the curl executable (tests).
pub const CURL_ENV: &str = "VIZE_LIB_CURL";

/// Executables used for fetching, overridable for tests and mirrors.
#[derive(Debug, Clone)]
pub struct Tools {
    pub npm: OsString,
    pub curl: OsString,
}

impl Default for Tools {
    fn default() -> Self {
        let npm = std::env::var_os(NPM_ENV).unwrap_or_else(|| {
            if cfg!(windows) {
                "npm.cmd".into()
            } else {
                "npm".into()
            }
        });
        let curl = std::env::var_os(CURL_ENV).unwrap_or_else(|| "curl".into());
        Self { npm, curl }
    }
}

fn run(command: &mut Command) -> Result<std::process::Output, String> {
    let output = command.output().map_err(|error| cstr!("{error}"))?;
    if output.status.success() {
        return Ok(output);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(cstr!("{} {}", output.status, stderr.trim()))
}

fn temporary_dir() -> LibResult<tempfile::TempDir> {
    tempfile::Builder::new()
        .prefix("vize-lib-")
        .tempdir()
        .map_err(|error| LibError::new(cstr!("failed to create a temporary directory: {error}")))
}

/// `node_modules/<package>/registry/registry.json`, searched upward from `root`.
pub fn installed_manifest(root: &Path, package: &str) -> Option<PathBuf> {
    let relative: PathBuf = ["node_modules"]
        .into_iter()
        .chain(package.split('/'))
        .chain(["registry", "registry.json"])
        .collect();
    root.ancestors()
        .map(|directory| directory.join(&relative))
        .find(|candidate| candidate.is_file())
}

/// `npm pack <package>@<version|latest>` into a temporary directory and extract it.
pub fn pack_registry(
    npm: &OsStr,
    package: &str,
    version: Option<&str>,
) -> LibResult<LoadedRegistry> {
    let spec: String = cstr!("{package}@{}", version.unwrap_or("latest"));
    let temporary = temporary_dir()?;
    let destination = temporary.path();
    run(Command::new(npm)
        .arg("pack")
        .arg(spec.as_str())
        .arg("--pack-destination")
        .arg(destination)
        .arg("--silent"))
    .map_err(|error| LibError::new(cstr!("npm pack {spec} failed: {error}")))?;
    let tarball = std::fs::read_dir(destination)
        .map_err(|error| LibError::io("read", destination, &error))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|extension| extension == "tgz"))
        .ok_or_else(|| LibError::new(cstr!("npm pack {spec} produced no tarball")))?;
    run(Command::new("tar")
        .arg("-xzf")
        .arg(&tarball)
        .arg("-C")
        .arg(destination))
    .map_err(|error| LibError::new(cstr!("tar -xzf {} failed: {error}", tarball.display())))?;
    let manifest_path = destination
        .join("package")
        .join("registry")
        .join("registry.json");
    if !manifest_path.is_file() {
        return Err(LibError::new(cstr!(
            "{spec} does not ship registry/registry.json (published before vize lib support?)"
        )));
    }
    let mut registry = LoadedRegistry::load(&manifest_path, cstr!("npm pack {spec}"))?;
    registry._temporary = Some(temporary);
    Ok(registry)
}

/// Latest published version of an npm package (`npm view <package> version`).
pub fn latest_version(npm: &OsStr, package: &str) -> LibResult<String> {
    let output = run(Command::new(npm).arg("view").arg(package).arg("version"))
        .map_err(|error| LibError::new(cstr!("npm view {package} version failed: {error}")))?;
    let version = String::from_utf8_lossy(&output.stdout);
    let version = version.trim();
    if version.is_empty() {
        return Err(LibError::new(cstr!(
            "npm view {package} returned no version"
        )));
    }
    Ok(version.into())
}

/// Download one `https://` URL to `target` with curl (HTTPS only, fail on HTTP errors).
pub fn download(curl: &OsStr, url: &str, target: &Path) -> LibResult<()> {
    if !url.starts_with("https://") {
        return Err(LibError::new(cstr!(
            "refusing to download non-https URL {url}"
        )));
    }
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|error| LibError::io("create", parent, &error))?;
    }
    run(Command::new(curl)
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
        ])
        .arg("--output")
        .arg(target)
        .arg(url))
    .map(drop)
    .map_err(|error| LibError::new(cstr!("curl {url} failed: {error}")))
}

/// Fetch an `https://…/registry.json`; its files are downloaded on demand.
pub fn url_registry(curl: &OsStr, url: &str) -> LibResult<LoadedRegistry> {
    let Some((base_url, _)) = url.rsplit_once('/') else {
        return Err(LibError::new(cstr!("invalid registry URL {url}")));
    };
    let temporary = temporary_dir()?;
    let manifest_path = temporary.path().join("registry.json");
    download(curl, url, &manifest_path)?;
    let mut registry = LoadedRegistry::load(&manifest_path, url)?;
    registry._temporary = Some(temporary);
    registry.remote = Some(RemoteFiles {
        base_url: base_url.into(),
        curl: curl.to_owned(),
    });
    Ok(registry)
}
