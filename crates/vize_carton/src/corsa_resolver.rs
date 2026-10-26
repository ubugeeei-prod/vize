//! Unified Corsa/native TypeScript executable discovery.
//!
//! Historically three independent resolvers existed (canon's LSP client, canon's
//! batch executor, and patina's type-aware linter session) and they could pick
//! different binaries for the same project. This module is the single source of
//! truth they all share.
//!
//! Resolution order:
//!
//! 1. Explicit configuration ([`CorsaResolveRequest::explicit_path`]).
//! 2. Environment variables, in precedence order: `CORSA_PATH`,
//!    `CORSA_EXECUTABLE`, `TSGO_PATH`, `TSGO_EXECUTABLE`, `TSRS_PATH`,
//!    `TSRS_EXECUTABLE`. The first one set wins; its value must point at an
//!    existing file.
//! 3. Project discovery: an ancestor walk from the selected project root (or
//!    the current directory when no root is supplied), following user-installed
//!    native TypeScript packages, declared aliases and linked platform packages.
//!    The nearest native package wins; local launchers remain a final fallback.
//!    Unlinked package-store entries and Vize's old bundled runtime are ignored.
//!
//! Discovery ends at those project roots. Global executables require an
//! explicit configuration or environment path; discovery does not run npm.
//!
//! Developer-checkout paths (`ref/typescript-go/...` and `../corsa-bind`) are
//! probed only in debug builds or when
//! `VIZE_CORSA_DEV_PATHS` is set to a non-empty value other than `0`, so
//! production binaries never scan repo-internal layouts.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

mod packages;
use packages::{find_node_modules_native, find_node_modules_wrapper, resolve_owned_launcher};

#[cfg(test)]
mod normalize_tests;

/// Environment variables honored by the resolver, in precedence order.
pub const CORSA_ENV_VARS: [&str; 6] = [
    "CORSA_PATH",
    "CORSA_EXECUTABLE",
    "TSGO_PATH",
    "TSGO_EXECUTABLE",
    "TSRS_PATH",
    "TSRS_EXECUTABLE",
];

/// Executable names probed by every discovery source, in precedence order.
pub const CORSA_EXECUTABLE_NAMES: [&str; 3] = ["corsa", "tsgo", "tsrs"];

/// Environment variable gating developer-checkout discovery paths.
pub const CORSA_DEV_PATHS_ENV: &str = "VIZE_CORSA_DEV_PATHS";

/// Input for [`resolve_corsa_executable`].
#[derive(Debug, Clone, Copy, Default)]
pub struct CorsaResolveRequest<'a> {
    /// Explicitly configured executable path (CLI flag or config file).
    /// Takes precedence over everything else and must exist.
    pub explicit_path: Option<&'a Path>,
    /// Project root used to anchor relative explicit/env paths and the
    /// discovery ancestor walk.
    pub project_root: Option<&'a Path>,
}

/// Failure modes of [`resolve_corsa_executable`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorsaResolveError {
    /// An explicitly requested path (config or environment variable) does not
    /// exist on disk.
    ExplicitNotFound {
        /// Where the path came from: `"configuration"` or an env var name.
        source: &'static str,
        /// The (relative-resolved) path that was probed.
        path: PathBuf,
    },
    /// No executable was found by any discovery source.
    NotFound,
    /// A runtime declared under this `node_modules` directory is installed
    /// but incomplete or mismatched. Discovery stops there, as Node would.
    InvalidRuntime {
        /// The `node_modules` directory that owns the declaration.
        directory: PathBuf,
    },
}

impl std::fmt::Display for CorsaResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExplicitNotFound { source, path } => write!(
                f,
                "Configured Corsa executable does not exist: {} (from {source})",
                path.display()
            ),
            Self::InvalidRuntime { directory } => write!(
                f,
                "The native TypeScript runtime declared under {} is incomplete or mismatched (platform package version, package.json, or lib/tsc). Reinstall it, or set typeChecker.corsaPath, CORSA_PATH, or TSGO_PATH to its executable.",
                directory.display()
            ),
            Self::NotFound => write!(
                f,
                "No native TypeScript/tsgo executable found. Install a native TypeScript runtime in the selected project, or set typeChecker.corsaPath, CORSA_PATH, or TSGO_PATH to its executable."
            ),
        }
    }
}

impl std::error::Error for CorsaResolveError {}

/// Resolve the Corsa executable for a project.
///
/// See the module docs for the full resolution order.
pub fn resolve_corsa_executable(
    request: CorsaResolveRequest<'_>,
) -> Result<PathBuf, CorsaResolveError> {
    resolve_with_env(request, |name| std::env::var_os(name))
}

/// Discover a Corsa executable by walking `start` and its ancestors, without
/// consulting env vars, `$HOME`, `npm root -g`, or `PATH`.
///
/// This is the discovery subset used to normalize wrapper paths; it is also
/// useful for tests that need a project-scoped lookup.
pub fn discover_corsa_in_ancestors(start: &Path) -> Option<PathBuf> {
    discover_in_walk(&[start.to_path_buf()], dev_paths_enabled())
}

/// Follow a selected `.bin` target and its own package's native dependency.
/// Never replace an explicit launcher with a different project provider.
pub fn normalize_corsa_path(path: &Path) -> PathBuf {
    normalize_corsa_path_with_discovery(path, resolve_owned_launcher)
}

fn normalize_corsa_path_with_discovery(
    path: &Path,
    discover: impl FnOnce(&Path) -> Option<PathBuf>,
) -> PathBuf {
    if wrapper_project_root(path).is_none() {
        return path.to_path_buf();
    }
    let target = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    discover(&target).unwrap_or(target)
}

/// The `@typescript/*-*` platform suffix for the current target.
pub fn platform_suffix() -> &'static str {
    if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "darwin-arm64"
        } else {
            "darwin-x64"
        }
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "aarch64") {
            "linux-arm64"
        } else {
            "linux-x64"
        }
    } else if cfg!(target_os = "windows") {
        if cfg!(target_arch = "aarch64") {
            "win32-arm64"
        } else {
            "win32-x64"
        }
    } else {
        ""
    }
}

fn resolve_with_env(
    request: CorsaResolveRequest<'_>,
    env: impl Fn(&str) -> Option<OsString>,
) -> Result<PathBuf, CorsaResolveError> {
    if let Some(path) = request.explicit_path {
        return resolve_required_path(path, "configuration", request.project_root);
    }

    for env_var in CORSA_ENV_VARS {
        let Some(value) = env(env_var) else {
            continue;
        };
        if value.is_empty() {
            continue;
        }
        return resolve_required_path(Path::new(&value), env_var, request.project_root);
    }

    discover_runtime(request.project_root)?.ok_or(CorsaResolveError::NotFound)
}

/// Resolve an explicitly requested path: anchor relative paths, require
/// existence, normalize wrappers, and canonicalize.
fn resolve_required_path(
    path: &Path,
    source: &'static str,
    project_root: Option<&Path>,
) -> Result<PathBuf, CorsaResolveError> {
    let resolved = resolve_relative_path(path, project_root);
    if !resolved.is_file() {
        return Err(CorsaResolveError::ExplicitNotFound {
            source,
            path: resolved,
        });
    }
    let normalized = normalize_corsa_path(&resolved);
    Ok(normalized.canonicalize().unwrap_or(normalized))
}

fn resolve_relative_path(path: &Path, project_root: Option<&Path>) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }

    project_root.map_or_else(|| path.to_path_buf(), |root| root.join(path))
}

fn discover_runtime(project_root: Option<&Path>) -> Result<Option<PathBuf>, CorsaResolveError> {
    let dev_paths = dev_paths_enabled();
    let cwd = std::env::current_dir().ok();
    let walk_starts = discovery_starts(project_root, cwd.as_deref());

    discover_in_walk_checked(&walk_starts, dev_paths).map_err(|directory| {
        CorsaResolveError::InvalidRuntime {
            directory: directory.join("node_modules"),
        }
    })
}

/// Whether developer-checkout paths may be probed.
fn dev_paths_enabled() -> bool {
    match std::env::var_os(CORSA_DEV_PATHS_ENV) {
        Some(value) => !value.is_empty() && value != "0",
        None => cfg!(debug_assertions),
    }
}

fn discovery_starts(project_root: Option<&Path>, cwd: Option<&Path>) -> Vec<PathBuf> {
    match project_root {
        Some(root) if root.is_absolute() => vec![root.to_path_buf()],
        Some(root) => vec![cwd.map_or_else(|| root.to_path_buf(), |cwd| cwd.join(root))],
        None => cwd.into_iter().map(Path::to_path_buf).collect(),
    }
}

enum Candidate {
    Native(PathBuf),
    Legacy(PathBuf),
    Wrapper(PathBuf),
}

/// Follow the selected dependency roots without replacing a nearer native
/// runtime with a different ancestor's version. Resolve native packages before
/// falling back to a local launcher.
fn discover_in_walk(walk_starts: &[PathBuf], dev_paths: bool) -> Option<PathBuf> {
    discover_in_walk_checked(walk_starts, dev_paths)
        .ok()
        .flatten()
}

/// Like [`discover_in_walk`], but an invalid declared runtime names the
/// directory whose `node_modules` declares it.
fn discover_in_walk_checked(
    walk_starts: &[PathBuf],
    dev_paths: bool,
) -> Result<Option<PathBuf>, PathBuf> {
    let mut wrapper_fallback: Option<PathBuf> = None;
    let mut visited: Vec<&Path> = Vec::new();

    for start in walk_starts {
        for dir in start.ancestors() {
            if visited.contains(&dir) {
                continue;
            }
            visited.push(dir);
            match probe_directory(dir, dev_paths) {
                Ok(Some(Candidate::Native(path) | Candidate::Legacy(path))) => {
                    return Ok(Some(path));
                }
                Ok(Some(Candidate::Wrapper(path))) => {
                    wrapper_fallback.get_or_insert(path);
                }
                Ok(None) => {}
                Err(_) => return Err(dir.to_path_buf()),
            }
        }
    }

    Ok(wrapper_fallback)
}

fn probe_directory(
    dir: &Path,
    dev_paths: bool,
) -> Result<Option<Candidate>, packages::InvalidNativePackage> {
    if let Some(path) = find_project_cache(dir, dev_paths) {
        return Ok(Some(Candidate::Native(path)));
    }
    if dev_paths
        && let Some(parent) = dir.parent()
        && let Some(path) = find_project_cache(&parent.join("corsa-bind"), dev_paths)
    {
        return Ok(Some(Candidate::Native(path)));
    }
    if let Some(path) = find_node_modules_native(dir)? {
        return Ok(Some(path));
    }
    Ok(find_node_modules_wrapper(dir).map(Candidate::Wrapper))
}

fn find_project_cache(dir: &Path, dev_paths: bool) -> Option<PathBuf> {
    if !dev_paths {
        return None;
    }
    let mut cache_dirs = vec![dir.join(".cache")];
    if dev_paths {
        let typescript_go = dir.join("ref").join("typescript-go");
        cache_dirs.push(typescript_go.join(".cache"));
        cache_dirs.push(typescript_go.join("built").join("local"));
    }

    for cache_dir in cache_dirs {
        for executable in CORSA_EXECUTABLE_NAMES {
            if let Some(found) = existing_executable(&cache_dir, executable) {
                return Some(found);
            }
        }
    }

    None
}

/// Probe `dir/{name}` and `dir/{name}.exe`. npm's Windows platform packages
/// ship `lib/tsgo.exe`; every other platform ships an extensionless binary.
fn existing_executable(dir: &Path, executable: &str) -> Option<PathBuf> {
    let plain = dir.join(executable);
    if plain.is_file() {
        return Some(plain);
    }
    let windows = dir.join(&*crate::cstr!("{executable}.exe"));
    if windows.is_file() {
        return Some(windows);
    }
    None
}

fn wrapper_project_root(path: &Path) -> Option<&Path> {
    let bin_dir = path.parent()?;
    if bin_dir.file_name()? != ".bin" {
        return None;
    }
    let node_modules = bin_dir.parent()?;
    if node_modules.file_name()? != "node_modules" {
        return None;
    }
    node_modules.parent()
}

#[cfg(test)]
mod tests;
