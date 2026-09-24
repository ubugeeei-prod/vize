//! Registry source resolution.
//!
//! Order: `--registry` paths, then the project's installed
//! `node_modules/@vizejs/<pkg>/registry/registry.json`, then (for an explicit
//! version, or when the package is not installed) `npm pack` + `tar -xzf`.

use std::path::{Path, PathBuf};
use std::process::Command;

use vize_s0::{String, cstr};

use super::error::{LibError, LibResult};
use super::registry::LoadedRegistry;

/// Source families `vize lib` knows how to fetch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegistryKind {
    Ui,
    Composable,
}

impl RegistryKind {
    pub const ALL: [Self; 2] = [Self::Ui, Self::Composable];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ui => "ui",
            Self::Composable => "composable",
        }
    }

    pub fn package_name(self) -> &'static str {
        match self {
            Self::Ui => "@vizejs/ui",
            Self::Composable => "@vizejs/composable",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "ui" => Some(Self::Ui),
            "composable" => Some(Self::Composable),
            _ => None,
        }
    }
}

/// Environment override for the npm executable (mirrors, tests).
pub const NPM_ENV: &str = "VIZE_LIB_NPM";

/// Loads registries lazily and memoizes them per kind and version.
pub struct Resolver {
    project_root: PathBuf,
    explicit: Vec<LoadedRegistry>,
    offline: bool,
    npm: std::ffi::OsString,
    loaded: Vec<LoadedRegistry>,
}

impl Resolver {
    /// Build a resolver; `explicit` paths may name `registry.json`, a registry
    /// directory, or a package directory.
    pub fn new(project_root: &Path, explicit: &[PathBuf], offline: bool) -> LibResult<Self> {
        let mut registries: Vec<LoadedRegistry> = Vec::new();
        for path in explicit {
            let registry = LoadedRegistry::load(&manifest_path_for(path)?, "--registry")?;
            if registries
                .iter()
                .any(|seen| seen.manifest.kind == registry.manifest.kind)
            {
                return Err(LibError::new(cstr!(
                    "more than one --registry provides {} items",
                    registry.manifest.kind
                )));
            }
            registries.push(registry);
        }
        Ok(Self {
            project_root: project_root.to_path_buf(),
            explicit: registries,
            offline,
            npm: npm_program(),
            loaded: Vec::new(),
        })
    }

    /// Override the npm executable used for `npm pack`.
    pub fn set_npm_program(&mut self, program: std::ffi::OsString) {
        self.npm = program;
    }

    /// Whether registries were passed explicitly (auto-discovery is then off).
    pub fn has_explicit(&self) -> bool {
        !self.explicit.is_empty()
    }

    /// Registry for `kind`, optionally pinned to an exact package version.
    pub fn registry(
        &mut self,
        kind: RegistryKind,
        version: Option<&str>,
    ) -> LibResult<&LoadedRegistry> {
        let matches = |registry: &LoadedRegistry| {
            registry.manifest.kind == kind.as_str()
                && version.is_none_or(|version| registry.manifest.package.version == version)
        };
        if self.has_explicit() {
            let registry = self
                .explicit
                .iter()
                .find(|registry| registry.manifest.kind == kind.as_str())
                .ok_or_else(|| {
                    LibError::new(cstr!("no --registry provides {} items", kind.as_str()))
                })?;
            if !matches(registry) {
                return Err(LibError::new(cstr!(
                    "--registry provides {}, not version {}",
                    registry.package_label(),
                    version.unwrap_or("?")
                )));
            }
            return Ok(registry);
        }
        if let Some(index) = self.loaded.iter().position(matches) {
            return self
                .loaded
                .get(index)
                .ok_or_else(|| LibError::new("registry cache out of range"));
        }
        let registry = match self.installed(kind)? {
            Some(installed) if matches(&installed) => installed,
            installed if self.offline => {
                return Err(LibError::new(match installed {
                    Some(found) => cstr!(
                        "installed {} does not match {}@{} and --offline forbids npm pack",
                        found.package_label(),
                        kind.package_name(),
                        version.unwrap_or("latest")
                    ),
                    None => cstr!(
                        "{} is not installed and --offline forbids npm pack",
                        kind.package_name()
                    ),
                }));
            }
            _ => pack_registry(&self.npm, kind, version)?,
        };
        self.loaded.push(registry);
        self.loaded
            .last()
            .ok_or_else(|| LibError::new("registry cache is empty"))
    }

    /// Registry for `kind` if one is available without touching the network.
    pub fn local_registry(&mut self, kind: RegistryKind) -> Option<&LoadedRegistry> {
        if self.has_explicit() || self.offline {
            return self.registry(kind, None).ok();
        }
        let previous = self.offline;
        self.offline = true;
        let found = self.registry(kind, None).is_ok();
        self.offline = previous;
        if found {
            self.registry(kind, None).ok()
        } else {
            None
        }
    }

    fn installed(&self, kind: RegistryKind) -> LibResult<Option<LoadedRegistry>> {
        let relative: PathBuf = ["node_modules"]
            .into_iter()
            .chain(kind.package_name().split('/'))
            .chain(["registry", "registry.json"])
            .collect();
        for directory in self.project_root.ancestors() {
            let candidate = directory.join(&relative);
            if candidate.is_file() {
                return LoadedRegistry::load(&candidate, "node_modules").map(Some);
            }
        }
        Ok(None)
    }
}

/// Map a `--registry` argument to its `registry.json`.
pub fn manifest_path_for(path: &Path) -> LibResult<PathBuf> {
    if path.is_file() {
        return Ok(path.to_path_buf());
    }
    for candidate in [
        path.join("registry.json"),
        path.join("registry").join("registry.json"),
    ] {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(LibError::new(cstr!(
        "no registry.json at {} (expected a file, a registry directory, or a package directory)",
        path.display()
    )))
}

fn npm_program() -> std::ffi::OsString {
    std::env::var_os(NPM_ENV).unwrap_or_else(|| {
        if cfg!(windows) {
            "npm.cmd".into()
        } else {
            "npm".into()
        }
    })
}

/// `npm pack <pkg>@<version>` into a temporary directory and extract it.
fn pack_registry(
    npm: &std::ffi::OsStr,
    kind: RegistryKind,
    version: Option<&str>,
) -> LibResult<LoadedRegistry> {
    let spec: String = match version {
        Some(version) => cstr!("{}@{version}", kind.package_name()),
        None => cstr!("{}@latest", kind.package_name()),
    };
    let temporary = tempfile::Builder::new()
        .prefix("vize-lib-")
        .tempdir()
        .map_err(|error| LibError::new(cstr!("failed to create a temporary directory: {error}")))?;
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

fn run(command: &mut Command) -> Result<(), String> {
    let output = command.output().map_err(|error| cstr!("{error}"))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(cstr!("{} {}", output.status, stderr.trim()))
}
